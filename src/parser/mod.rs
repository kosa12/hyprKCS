use anyhow::{Context, Result};
use dirs::config_dir;
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub mod input;

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: Rc<str>,
    pub clean_mods: Rc<str>,
    pub flags: Rc<str>,
    pub key: Rc<str>,
    pub dispatcher: Rc<str>,
    pub args: Rc<str>,
    pub description: Option<Rc<str>>,
    pub submap: Option<Rc<str>>,
    pub line_number: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Rc<str>,
    pub value: Rc<str>,
    pub line_number: usize,
    pub file_path: PathBuf,
}

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(env_path) = std::env::var("HYPRKCS_CONFIG") {
        let path = expand_tilde(&env_path);
        if path.is_dir() {
            return Ok(path.join(crate::config::constants::HYPRLAND_CONF));
        }
        return Ok(path);
    }

    // Check for alternative config path in settings
    let style_config = crate::config::StyleConfig::load();
    if let Some(alt_path) = style_config.alternative_config_path {
        if !alt_path.trim().is_empty() {
            let path = expand_tilde(&alt_path);
            // If user pointed to a folder, append default config name
            if path.is_dir() {
                return Ok(path.join(crate::config::constants::HYPRLAND_CONF));
            }
            return Ok(path);
        }
    }

    let mut path = config_dir().context("Could not find config directory")?;
    path.push(crate::config::constants::HYPR_DIR);
    path.push(crate::config::constants::HYPRLAND_CONF);
    Ok(path)
}

fn expand_tilde(path_str: &str) -> PathBuf {
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path_str)
}

fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

fn resolve_variables(
    input: &str,
    vars: &HashMap<String, String>,
    sorted_keys: &[String],
) -> String {
    if !input.contains('$') {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(idx) = remaining.find('$') {
        result.push_str(&remaining[..idx]);
        remaining = &remaining[idx..];

        let mut matched = false;
        // sorted_keys are sorted by length descending, so the first match is the longest
        for key in sorted_keys {
            if remaining.starts_with(key) {
                if let Some(val) = vars.get(key) {
                    result.push_str(val);
                    remaining = &remaining[key.len()..];
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            // No variable matched, keep the '$' and advance
            // We need to advance by at least 1 char to avoid infinite loop
            // Handle potentially multi-byte char if '$' wasn't 1 byte (it is, but good practice)
            let mut chars = remaining.chars();
            if let Some(c) = chars.next() {
                result.push(c);
                remaining = chars.as_str();
            } else {
                break;
            }
        }
    }
    result.push_str(remaining);
    result
}

fn split_comment(line: &str) -> (&str, &str) {
    if let Some(idx) = line.find(" #") {
        (&line[..idx], &line[idx..])
    } else if line.trim_start().starts_with('#') {
        ("", line)
    } else if let Some(idx) = line.find('#') {
        (&line[..idx], &line[idx..])
    } else {
        (line, "")
    }
}

fn expand_path(
    path_str: &str,
    current_file: &Path,
    vars: &HashMap<String, String>,
    sorted_keys: &[String],
) -> PathBuf {
    let resolved_path_str = resolve_variables(path_str, vars, sorted_keys);
    let path_str = resolved_path_str.trim();

    if path_str.starts_with('~') {
        return expand_tilde(path_str);
    }

    let p = PathBuf::from(path_str);
    if p.is_absolute() {
        p
    } else {
        current_file.parent().unwrap_or(&PathBuf::from(".")).join(p)
    }
}

/// Caches file contents and variables to avoid redundant I/O and processing
struct ParserContext {
    variables: HashMap<String, String>,
    sorted_keys: Vec<String>,
    visited: HashSet<PathBuf>,
    keys_dirty: bool,
}

impl ParserContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            sorted_keys: Vec::new(),
            visited: HashSet::new(),
            keys_dirty: false,
        }
    }

    fn mark_dirty(&mut self) {
        self.keys_dirty = true;
    }

    fn ensure_sorted(&mut self) {
        if self.keys_dirty {
            self.sorted_keys = self.variables.keys().cloned().collect();
            self.sorted_keys
                .sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
            self.keys_dirty = false;
        }
    }
}

struct ConfigData {
    variables: HashMap<String, String>,
    defined_variables: Vec<Variable>,
    file_cache: HashMap<PathBuf, Rc<String>>,
}

fn load_config_data() -> Result<ConfigData> {
    let main_path = get_config_path()?;
    let mut ctx = ParserContext::new();

    // Inject $hypr variable pointing to the config root
    // This supports the common convention of using $hypr to refer to the config dir,
    // which is especially useful when loading from a custom directory where the
    // user's actual environment variable might not match the test context.
    let active_root = main_path
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."));

    ctx.variables.insert(
        "$hypr".to_string(),
        active_root.to_string_lossy().to_string(),
    );
    ctx.mark_dirty();

    let mut file_cache: HashMap<PathBuf, Rc<String>> = HashMap::new();
    let mut path_cache: HashMap<PathBuf, Rc<PathBuf>> = HashMap::new();
    let mut defined_variables = Vec::new();

    fn collect_recursive(
        path: PathBuf,
        ctx: &mut ParserContext,
        file_cache: &mut HashMap<PathBuf, Rc<String>>,
        path_cache: &mut HashMap<PathBuf, Rc<PathBuf>>,
        defined_variables: &mut Vec<Variable>,
        system_root: &Path,
        active_root: &Path,
    ) -> Result<()> {
        if ctx.visited.contains(&path) {
            return Ok(());
        }
        // If path doesn't exist, we can't read it.
        // However, we might have reached here via a re-rooted path that DOES exist.
        // If it still doesn't exist, we skip.
        if !path.exists() {
            return Ok(());
        }

        ctx.visited.insert(path.clone());

        // Support for directory sourcing (Hyprland feature)
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
                paths.sort(); // Alphabetical order
                for sub_path in paths {
                    let _ = collect_recursive(
                        sub_path,
                        ctx,
                        file_cache,
                        path_cache,
                        defined_variables,
                        system_root,
                        active_root,
                    );
                }
            }
            return Ok(());
        }

        // Get or create shared path reference
        let shared_path = path_cache
            .entry(path.clone())
            .or_insert_with(|| Rc::new(path.clone()))
            .clone();

        let content = if let Some(cached) = file_cache.get(&path) {
            cached.clone()
        } else {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            let rc = Rc::new(s);
            file_cache.insert(path.clone(), rc.clone());
            rc
        };

        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Variable parsing: $name = value
            if line.starts_with('$') {
                if let Some((name_part, value_part)) = line.split_once('=') {
                    let name = name_part.trim().to_string();
                    let (raw_value_part, _) = split_comment(value_part);
                    let raw_value = raw_value_part.trim();

                    if !name.is_empty() {
                        // Store the definition for UI management (use shared path)
                        defined_variables.push(Variable {
                            name: Rc::from(name.as_str()),
                            value: Rc::from(raw_value), // Store raw value for editing
                            line_number: line_idx,
                            file_path: (*shared_path).clone(),
                        });

                        // Resolve for parser usage - only if value contains variables
                        let value = if raw_value.contains('$') {
                            ctx.ensure_sorted();
                            resolve_variables(raw_value, &ctx.variables, &ctx.sorted_keys)
                        } else {
                            raw_value.to_string()
                        };

                        ctx.variables.insert(name, value);
                        ctx.mark_dirty();
                    }
                }
            }
            // Source parsing: source = path
            else if let Some(rest) = line.strip_prefix("source") {
                let trimmed_rest = rest.trim_start();
                if let Some(path_part) = trimmed_rest.strip_prefix('=') {
                    let path_str = path_part
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"');

                    // Only get sorted keys if path contains variables
                    let mut sourced_path = if path_str.contains('$') {
                        ctx.ensure_sorted();
                        expand_path(path_str, &path, &ctx.variables, &ctx.sorted_keys)
                    } else {
                        expand_path(path_str, &path, &ctx.variables, &[])
                    };

                    // Fallback: If path ends in /.conf and doesn't exist, try /*.conf
                    // This handles potential typos or specific user patterns where .conf implies *.conf
                    if !sourced_path.exists() && sourced_path.ends_with(".conf") {
                        if let Some(parent) = sourced_path.parent() {
                            sourced_path = parent.join("*.conf");
                        }
                    }

                    // --- Re-rooting Logic ---
                    if system_root != active_root {
                        if let Ok(suffix) = sourced_path.strip_prefix(system_root) {
                            let remapped = active_root.join(suffix);
                            // Check if remapped path exists or is a glob pattern that matches files
                            let remapped_str = remapped.to_string_lossy();
                            let is_glob = is_glob_pattern(&remapped_str);

                            if remapped.exists()
                                || (is_glob
                                    && glob(&remapped_str)
                                        .map_or(false, |mut p| p.next().is_some()))
                            {
                                sourced_path = remapped;
                            }
                        }
                    }

                    // Use glob to expand wildcards
                    let pattern = sourced_path.to_string_lossy();
                    if !is_glob_pattern(&pattern) {
                        let _ = collect_recursive(
                            sourced_path,
                            ctx,
                            file_cache,
                            path_cache,
                            defined_variables,
                            system_root,
                            active_root,
                        );
                    } else if let Ok(paths) = glob(&pattern) {
                        for entry in paths {
                            if let Ok(p) = entry {
                                let _ = collect_recursive(
                                    p,
                                    ctx,
                                    file_cache,
                                    path_cache,
                                    defined_variables,
                                    system_root,
                                    active_root,
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let system_root = config_dir()
        .unwrap_or_default()
        .join(crate::config::constants::HYPR_DIR);
    let active_root = main_path.parent().unwrap_or(Path::new(".")).to_path_buf();

    collect_recursive(
        main_path,
        &mut ctx,
        &mut file_cache,
        &mut path_cache,
        &mut defined_variables,
        &system_root,
        &active_root,
    )?;
    Ok(ConfigData {
        variables: ctx.variables,
        defined_variables,
        file_cache,
    })
}

pub fn get_variables() -> Result<HashMap<String, String>> {
    let data = load_config_data()?;
    Ok(data.variables)
}

pub fn get_defined_variables() -> Result<Vec<Variable>> {
    let data = load_config_data()?;
    Ok(data.defined_variables)
}

pub fn get_loaded_files() -> Result<Vec<PathBuf>> {
    let data = load_config_data()?;
    Ok(data.file_cache.keys().cloned().collect())
}

fn replace_variable_in_content(content: &str, old_name: &str, new_name: &str) -> (String, bool) {
    let search_term = format!("${}", old_name);
    let replacement = format!("${}", new_name);

    let mut new_content = String::with_capacity(content.len());
    let mut last_idx = 0;
    let mut modified = false;

    let matches: Vec<_> = content.match_indices(&search_term).collect();

    for (idx, _) in matches {
        // Check word boundary
        let after_idx = idx + search_term.len();

        let is_boundary = if after_idx >= content.len() {
            true
        } else {
            let c = content[after_idx..].chars().next().unwrap();
            !c.is_alphanumeric() && c != '_'
        };

        if is_boundary {
            // Append everything up to match
            new_content.push_str(&content[last_idx..idx]);
            // Append replacement
            new_content.push_str(&replacement);
            last_idx = after_idx;
            modified = true;
        }
    }

    // Append remaining
    new_content.push_str(&content[last_idx..]);

    (new_content, modified)
}

pub fn rename_variable_references(old_name: &str, new_name: &str) -> Result<usize> {
    let files = get_loaded_files()?;
    let mut count = 0;

    for path in files {
        if !path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let (new_content, modified) = replace_variable_in_content(&content, old_name, new_name);

        if modified {
            std::fs::write(&path, new_content)?;
            count += 1;
        }
    }
    Ok(count)
}

pub fn count_variable_references(name: &str) -> Result<usize> {
    let files = get_loaded_files()?;
    let mut count = 0;

    let search_term = format!("${}", name.trim_start_matches('$'));

    for path in files {
        if !path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let matches: Vec<_> = content.match_indices(&search_term).collect();

        for (idx, _) in matches {
            let after_idx = idx + search_term.len();

            let is_boundary = if after_idx >= content.len() {
                true
            } else {
                let c = content[after_idx..].chars().next().unwrap();
                !c.is_alphanumeric() && c != '_'
            };

            let mut is_definition = false;
            if is_boundary {
                let mut check_idx = after_idx;
                while check_idx < content.len() {
                    let c = content[check_idx..].chars().next().unwrap();
                    if !c.is_whitespace() {
                        if c == '=' {
                            is_definition = true;
                        }
                        break;
                    }
                    check_idx += c.len_utf8();
                }
            }

            if is_boundary && !is_definition {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn inline_variable_references(name: &str, value: &str) -> Result<usize> {
    let files = get_loaded_files()?;
    let mut count = 0;

    // We are replacing $name with value
    let search_term = format!("${}", name.trim_start_matches('$'));
    let replacement = value; // No $ prefix for value (unless value has it, which we pass as is)

    for path in files {
        if !path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;

        // Custom replacement logic to avoid regex dependency
        let mut new_content = String::with_capacity(content.len());
        let mut last_idx = 0;
        let mut modified = false;

        let matches: Vec<_> = content.match_indices(&search_term).collect();

        for (idx, _) in matches {
            let after_idx = idx + search_term.len();

            let is_boundary = if after_idx >= content.len() {
                true
            } else {
                let c = content[after_idx..].chars().next().unwrap();
                !c.is_alphanumeric() && c != '_'
            };

            // Avoid replacing the definition itself: "$name ="
            // Quick heuristic: check if next non-whitespace char is '='
            let mut is_definition = false;
            if is_boundary {
                let mut check_idx = after_idx;
                while check_idx < content.len() {
                    let c = content[check_idx..].chars().next().unwrap();
                    if !c.is_whitespace() {
                        if c == '=' {
                            is_definition = true;
                        }
                        break;
                    }
                    check_idx += c.len_utf8();
                }
            }

            if is_boundary && !is_definition {
                new_content.push_str(&content[last_idx..idx]);
                new_content.push_str(replacement);
                last_idx = after_idx;
                modified = true;
            }
        }

        new_content.push_str(&content[last_idx..]);

        if modified {
            std::fs::write(&path, new_content)?;
            count += 1;
        }
    }
    Ok(count)
}

/// Finds occurrences of a literal value in keybind lines and replaces them with a variable reference.
/// Returns the total number of actual replacements made across all configuration files.
pub fn refactor_hardcoded_references(value: &str, variable_name: &str) -> Result<usize> {
    let files = get_loaded_files()?;
    let mut count = 0;

    // We want to replace `value` with `$variable_name`
    let replacement = format!("${}", variable_name.trim_start_matches('$'));
    let search_term = value;

    for path in files {
        if !path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let mut modified = false;

        // Iterate line by line to only target "bind" lines
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::with_capacity(lines.len());

        for line in lines {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("bind") {
                // Perform replacement on this line
                let mut new_line = String::with_capacity(line.len());
                let mut line_last_idx = 0;
                let matches: Vec<_> = line.match_indices(search_term).collect();

                let mut line_modified = false;

                for (idx, _) in matches {
                    let after_idx = idx + search_term.len();

                    // Word boundary check
                    let is_start_boundary = if idx == 0 {
                        true
                    } else {
                        let c = line[..idx].chars().last().unwrap();
                        !c.is_alphanumeric() && c != '_' && c != '-' && c != '$'
                    };

                    let is_end_boundary = if after_idx >= line.len() {
                        true
                    } else {
                        let c = line[after_idx..].chars().next().unwrap();
                        !c.is_alphanumeric() && c != '_' && c != '-'
                    };

                    if is_start_boundary && is_end_boundary {
                        new_line.push_str(&line[line_last_idx..idx]);
                        new_line.push_str(&replacement);
                        line_last_idx = after_idx;
                        line_modified = true;
                        count += 1;
                    }
                }
                new_line.push_str(&line[line_last_idx..]);

                if line_modified {
                    new_lines.push(new_line);
                    modified = true;
                } else {
                    new_lines.push(line.to_string());
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        if modified {
            write_lines(&path, &new_lines)?;
        }
    }
    Ok(count)
}

pub fn write_lines<P: AsRef<Path>>(path: P, lines: &[String]) -> Result<()> {
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::write(path, content).context("Failed to write to file")
}

pub fn add_variable(path: PathBuf, name: &str, value: &str) -> Result<()> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() {
        vec![]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    };

    let new_line = format!("${} = {}", name.trim_start_matches('$'), value);

    // Try to find a block of variables to append to
    let mut insert_idx = 0;
    let mut found_vars = false;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with('$') {
            found_vars = true;
            insert_idx = i + 1;
        } else if found_vars && !line.trim().is_empty() {
            // End of variable block?
            break;
        }
    }

    if found_vars {
        lines.insert(insert_idx, new_line);
    } else {
        // No variables found, insert at top or after comments
        let mut top_idx = 0;
        for (i, line) in lines.iter().enumerate() {
            if !line.trim().starts_with('#') {
                top_idx = i;
                break;
            }
        }
        lines.insert(top_idx, new_line);
    }

    write_lines(&path, &lines)
}

pub fn update_variable(
    path: PathBuf,
    line_number: usize,
    new_name: &str,
    new_value: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    // Preserve comments if any
    let original = &lines[line_number];
    let (_, comment_part) = split_comment(original);

    lines[line_number] = format!(
        "${} = {}{}{}",
        new_name.trim_start_matches('$'),
        new_value,
        if comment_part.is_empty() { "" } else { " " },
        comment_part
    );
    write_lines(&path, &lines)
}

pub fn delete_variable(path: PathBuf, line_number: usize) -> Result<()> {
    // Reuse existing delete logic since it's just line removal
    delete_keybind(path, line_number)
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    let main_path = get_config_path()?;
    let data = load_config_data()?;
    let variables = data.variables;
    let file_cache = data.file_cache;

    let mut sorted_keys: Vec<_> = variables.keys().cloned().collect();
    sorted_keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));

    let mut keybinds = Vec::new();
    let mut visited = HashSet::new();
    let mut current_submap: Option<Rc<str>> = None;

    fn parse_recursive(
        path: PathBuf,
        keybinds: &mut Vec<Keybind>,
        variables: &HashMap<String, String>,
        sorted_keys: &[String],
        visited: &mut HashSet<PathBuf>,
        current_submap: &mut Option<Rc<str>>,
        file_cache: &HashMap<PathBuf, Rc<String>>,
        system_root: &Path,
        active_root: &Path,
    ) -> Result<()> {
        if visited.contains(&path) {
            return Ok(());
        }
        // Skip if path doesn't exist, unless we find it via re-rooting logic which we do BEFORE calling this function recursively
        if !path.exists() {
            return Ok(());
        }

        visited.insert(path.clone());

        // Support for directory sourcing (Hyprland feature)
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
                paths.sort(); // Alphabetical order
                for sub_path in paths {
                    let _ = parse_recursive(
                        sub_path,
                        keybinds,
                        variables,
                        sorted_keys,
                        visited,
                        current_submap,
                        file_cache,
                        system_root,
                        active_root,
                    );
                }
            }
            return Ok(());
        }

        // Use cached content or read if missing (should be in cache from load_config_data, but fallback)
        let content = if let Some(cached) = file_cache.get(&path) {
            cached.clone()
        } else {
            Rc::new(std::fs::read_to_string(&path).unwrap_or_default())
        };

        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            // Check for submap
            if let Some(rest) = line_trimmed.strip_prefix("submap") {
                let rest_trimmed = rest.trim_start();
                if let Some(val) = rest_trimmed.strip_prefix('=') {
                    let name = val.split('#').next().unwrap_or("").trim();
                    if name == "reset" {
                        *current_submap = None;
                    } else {
                        *current_submap = Some(name.into());
                    }
                }
            }
            // Check for bind
            else if let Some(rest) = line_trimmed.strip_prefix("bind") {
                let rest = rest.trim_start(); // could check flags here like 'e', 'l', etc.

                // extract potential flags: take while alphanumeric
                let flags;
                let mut remaining = rest;

                // Simple manual "take_while" for flags
                // 'bind' is already stripped. "bindl =" -> "l ="
                if let Some(eq_idx) = remaining.find('=') {
                    let potential_flags = remaining[..eq_idx].trim();
                    if potential_flags.chars().all(|c| c.is_alphabetic()) {
                        flags = potential_flags.to_string();
                        remaining = &remaining[eq_idx + 1..]; // skip '='
                    } else {
                        // malformed or no equals?
                        continue;
                    }
                } else {
                    continue;
                }

                let raw_content = remaining.trim();
                let mut description = None;

                // Check inline
                if let Some(idx) = line.find('#') {
                    let comment = line[idx + 1..].trim();
                    if !comment.is_empty() {
                        description = Some(Rc::from(comment));
                    }
                }

                // Check preceding line if no inline description found
                if description.is_none() && index > 0 {
                    let prev_line = lines[index - 1].trim();
                    if prev_line.starts_with('#') {
                        let comment = prev_line.trim_start_matches('#').trim();
                        if !comment.is_empty() {
                            description = Some(Rc::from(comment));
                        }
                    }
                }

                let resolved_content = resolve_variables(raw_content, variables, sorted_keys);
                let content_clean = resolved_content.split('#').next().unwrap_or("").trim();

                // Custom splitter to respect quotes (e.g. for bash -c "...")
                let mut parts = Vec::with_capacity(5);
                let mut current_part = String::with_capacity(32);
                let mut in_quote = false;
                let mut parts_count = 0;

                let is_bindd = flags == "d";
                let limit = if is_bindd { 4 } else { 3 };

                for c in content_clean.chars() {
                    if parts_count < limit {
                        if c == '"' {
                            in_quote = !in_quote;
                            current_part.push(c);
                        } else if c == ',' && !in_quote {
                            parts.push(current_part.trim().to_string());
                            current_part.clear();
                            parts_count += 1;
                        } else {
                            current_part.push(c);
                        }
                    } else {
                        // For the last part (args), just take everything else
                        current_part.push(c);
                    }
                }
                if !current_part.trim().is_empty() || parts_count >= limit {
                    parts.push(current_part.trim().to_string());
                }

                if parts.len() >= 3 {
                    let mods: Rc<str>;
                    let key: Rc<str>;
                    let dispatcher: Rc<str>;
                    let args: Rc<str>;

                    if is_bindd {
                        mods = Rc::from(parts[0].as_str());
                        key = Rc::from(parts[1].as_str());
                        // parts[2] is description
                        if parts.len() > 2 {
                            let desc_str = parts[2].trim();
                            if !desc_str.is_empty() {
                                description = Some(Rc::from(desc_str));
                            }
                        }

                        if parts.len() > 3 {
                            dispatcher = Rc::from(parts[3].as_str());
                        } else {
                            dispatcher = Rc::from("");
                        }

                        if parts.len() > 4 {
                            args = Rc::from(parts[4].as_str());
                        } else {
                            args = Rc::from("");
                        }
                    } else {
                        mods = Rc::from(parts[0].as_str());
                        key = Rc::from(parts[1].as_str());
                        dispatcher = Rc::from(parts[2].as_str());
                        args = if parts.len() > 3 {
                            Rc::from(parts[3].as_str())
                        } else {
                            Rc::from("")
                        };
                    }

                    keybinds.push(Keybind {
                        mods: mods.clone(),
                        clean_mods: mods,
                        flags: Rc::from(flags.as_str()),
                        key,
                        dispatcher,
                        args,
                        description,
                        submap: current_submap.clone(),
                        line_number: index,
                        file_path: path.clone(),
                    });
                }
            }
            // Check for source
            else if let Some(rest) = line_trimmed.strip_prefix("source") {
                let trimmed_rest = rest.trim_start();
                if let Some(path_part) = trimmed_rest.strip_prefix('=') {
                    let path_str = path_part
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"');

                    let mut sourced_path = expand_path(path_str, &path, variables, sorted_keys);

                    // Fallback: If path ends in /.conf and doesn't exist, try /*.conf
                    if !sourced_path.exists() && sourced_path.ends_with(".conf") {
                        if let Some(parent) = sourced_path.parent() {
                            sourced_path = parent.join("*.conf");
                        }
                    }

                    // --- Re-rooting Logic ---
                    if system_root != active_root {
                        if let Ok(suffix) = sourced_path.strip_prefix(system_root) {
                            let remapped = active_root.join(suffix);
                            // Check if remapped path exists or is a glob pattern that matches files
                            let remapped_str = remapped.to_string_lossy();
                            let is_glob = is_glob_pattern(&remapped_str);

                            if remapped.exists()
                                || (is_glob
                                    && glob(&remapped_str)
                                        .map_or(false, |mut p| p.next().is_some()))
                            {
                                sourced_path = remapped;
                            }
                        }
                    }

                    // Use glob to expand wildcards
                    let pattern = sourced_path.to_string_lossy();
                    if !is_glob_pattern(&pattern) {
                        let _ = parse_recursive(
                            sourced_path,
                            keybinds,
                            variables,
                            sorted_keys,
                            visited,
                            current_submap,
                            file_cache,
                            system_root,
                            active_root,
                        );
                    } else if let Ok(paths) = glob(&pattern) {
                        for entry in paths {
                            if let Ok(p) = entry {
                                let _ = parse_recursive(
                                    p,
                                    keybinds,
                                    variables,
                                    sorted_keys,
                                    visited,
                                    current_submap,
                                    file_cache,
                                    system_root,
                                    active_root,
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let system_root = config_dir()
        .unwrap_or_default()
        .join(crate::config::constants::HYPR_DIR);
    let active_root = main_path
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();

    parse_recursive(
        main_path,
        &mut keybinds,
        &variables,
        &sorted_keys,
        &mut visited,
        &mut current_submap,
        &file_cache,
        &system_root,
        &active_root,
    )?;
    Ok(keybinds)
}

#[allow(clippy::too_many_arguments)]
pub fn update_line(
    path: PathBuf,
    line_number: usize,
    new_mods: &str,
    new_key: &str,
    new_dispatcher: &str,
    new_args: &str,
    description: Option<String>,
    new_flags: Option<&str>,
) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    let original_line = &lines[line_number];
    // Manual parsing for update_line logic
    // We want to preserve indentation and the 'bind' part
    // regex was: r"^(\s*)bind([a-zA-Z]*)(\s*=\s*)([^"]*)"

    // 1. Indent
    let indent_len = original_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    let indent = &original_line[..indent_len];
    let trimmed_start = &original_line[indent_len..];

    if let Some(after_bind) = trimmed_start.strip_prefix("bind") {
        if let Some(eq_idx) = after_bind.find('=') {
            let current_flags = after_bind[..eq_idx].trim();

            let flags = new_flags.unwrap_or(current_flags);
            let is_bindd = flags == "d";

            // preserve existing spacing around equals if possible, or just standard " = "
            // The original code reconstructed the line completely anyway.

            let mut new_line = if is_bindd {
                let desc_str = description.as_deref().unwrap_or("");
                if new_args.trim().is_empty() {
                    format!(
                        "{}bind{} = {}, {}, {}, {}",
                        indent, flags, new_mods, new_key, desc_str, new_dispatcher
                    )
                } else {
                    format!(
                        "{}bind{} = {}, {}, {}, {}, {}",
                        indent, flags, new_mods, new_key, desc_str, new_dispatcher, new_args
                    )
                }
            } else if new_args.trim().is_empty() {
                format!(
                    "{}bind{} = {}, {}, {}",
                    indent, flags, new_mods, new_key, new_dispatcher
                )
            } else {
                format!(
                    "{}bind{} = {}, {}, {}, {}",
                    indent, flags, new_mods, new_key, new_dispatcher, new_args
                )
            };

            if !is_bindd {
                if let Some(desc) = description {
                    if !desc.trim().is_empty() {
                        new_line = format!("{} # {}", new_line, desc.trim());
                    }
                } else {
                    // Preserve existing comment if no new description provided
                    if let Some(idx) = original_line.find('#') {
                        new_line = format!("{} {}", new_line, &original_line[idx..]);
                    }
                }
            }

            lines[line_number] = new_line;
            write_lines(&path, &lines)
        } else {
            Err(anyhow::anyhow!(
                "Could not parse original line structure (missing =)"
            ))
        }
    } else {
        Err(anyhow::anyhow!(
            "Could not parse original line structure (not a bind)"
        ))
    }
}

pub fn create_submap_block(
    path: PathBuf,
    name: &str,
    reset_key: Option<&str>,
    exit_target: &str,
) -> Result<()> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() {
        vec![]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    };

    // Ensure we are in global scope before adding new block
    let needs_reset = lines
        .iter()
        .rev()
        .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .map(|l| l.trim() != "submap = reset")
        .unwrap_or(false);

    if needs_reset {
        lines.push("submap = reset".to_string());
    }

    lines.push(String::new());
    lines.push(format!("submap = {}", name));

    if let Some(rk) = reset_key {
        if !rk.trim().is_empty() {
            lines.push(format!("bind = , {}, submap, {}", rk, exit_target));
        }
    }

    lines.push("submap = reset".to_string());

    write_lines(&path, &lines)
}

#[allow(clippy::too_many_arguments)]
pub fn add_keybind(
    path: PathBuf,
    mods: &str,
    key: &str,
    dispatcher: &str,
    args: &str,
    submap: Option<String>,
    description: Option<String>,
    flags: &str,
) -> Result<usize> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() {
        vec![]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    };

    let is_bindd = flags == "d";
    let bind_cmd = if flags.is_empty() {
        "bind".to_string()
    } else {
        format!("bind{}", flags)
    };

    let mut new_line = if is_bindd {
        let desc_str = description.as_deref().unwrap_or("");

        if args.trim().is_empty() {
            format!(
                "{} = {}, {}, {}, {}",
                bind_cmd, mods, key, desc_str, dispatcher
            )
        } else {
            format!(
                "{} = {}, {}, {}, {}, {}",
                bind_cmd, mods, key, desc_str, dispatcher, args
            )
        }
    } else if args.trim().is_empty() {
        format!("{} = {}, {}, {}", bind_cmd, mods, key, dispatcher)
    } else {
        format!("{} = {}, {}, {}, {}", bind_cmd, mods, key, dispatcher, args)
    };

    if !is_bindd {
        if let Some(desc) = description {
            if !desc.trim().is_empty() {
                new_line = format!("{} # {}", new_line, desc.trim());
            }
        }
    }

    if let Some(submap_name) = submap.filter(|s| !s.is_empty()) {
        let submap_decl = format!("submap = {}", submap_name);
        let mut found_submap = false;
        let mut insert_index = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == submap_decl {
                found_submap = true;
                // Look ahead for the end of this submap
                for (j, line_j) in lines.iter().enumerate().skip(i + 1) {
                    let next_trimmed = line_j.trim();
                    if next_trimmed.starts_with("submap =") {
                        // Found end of block (either reset or another submap start)
                        insert_index = Some(j);
                        break;
                    }
                }
                if insert_index.is_none() {
                    // Submap exists but no closing 'submap =' found, append to end
                    insert_index = Some(lines.len());
                }
                break;
            }
        }

        if let Some(idx) = insert_index {
            lines.insert(idx, new_line);
            write_lines(&path, &lines)?;
            Ok(idx)
        } else if found_submap {
            // Should have been handled above, but fallback
            lines.push(new_line);
            write_lines(&path, &lines)?;
            Ok(lines.len() - 1)
        } else {
            // Submap doesn't exist, create it
            lines.push(String::new()); // spacer
            lines.push(submap_decl);
            lines.push(new_line);
            lines.push("submap = reset".to_string());

            write_lines(&path, &lines)?;
            Ok(lines.len() - 2) // Index of the new bind
        }
    } else {
        // Global map
        lines.push(new_line);
        write_lines(&path, &lines)?;
        Ok(lines.len() - 1)
    }
}

pub fn delete_keybind(path: PathBuf, line_number: usize) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    lines.remove(line_number);
    write_lines(&path, &lines)
}

pub struct BatchUpdate {
    pub line_number: usize,
    pub new_mods: String,
    pub new_key: String,
    pub new_dispatcher: String,
    pub new_args: String,
    pub description: Option<String>,
}

pub fn update_multiple_lines(path: PathBuf, updates: Vec<BatchUpdate>) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    for update in updates {
        if update.line_number >= lines.len() {
            continue; // Skip out of bounds
        }

        let original_line = &lines[update.line_number];

        // 1. Indent
        let indent_len = original_line
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();
        let indent = &original_line[..indent_len];
        let trimmed_start = &original_line[indent_len..];

        if let Some(after_bind) = trimmed_start.strip_prefix("bind") {
            if let Some(eq_idx) = after_bind.find('=') {
                let flags = after_bind[..eq_idx].trim();
                let is_bindd = flags == "d";

                let mut new_line = if is_bindd {
                    let desc_str = update.description.as_deref().unwrap_or("");
                    if update.new_args.trim().is_empty() {
                        format!(
                            "{}bind{} = {}, {}, {}, {}",
                            indent,
                            flags,
                            update.new_mods,
                            update.new_key,
                            desc_str,
                            update.new_dispatcher
                        )
                    } else {
                        format!(
                            "{}bind{} = {}, {}, {}, {}, {}",
                            indent,
                            flags,
                            update.new_mods,
                            update.new_key,
                            desc_str,
                            update.new_dispatcher,
                            update.new_args
                        )
                    }
                } else if update.new_args.trim().is_empty() {
                    format!(
                        "{}bind{} = {}, {}, {}",
                        indent, flags, update.new_mods, update.new_key, update.new_dispatcher
                    )
                } else {
                    format!(
                        "{}bind{} = {}, {}, {}, {}",
                        indent,
                        flags,
                        update.new_mods,
                        update.new_key,
                        update.new_dispatcher,
                        update.new_args
                    )
                };

                if !is_bindd {
                    if let Some(desc) = update.description {
                        if !desc.trim().is_empty() {
                            new_line = format!("{} # {}", new_line, desc.trim());
                        }
                    } else {
                        // Preserve existing comment if no new description provided
                        if let Some(idx) = original_line.find('#') {
                            new_line = format!("{} {}", new_line, &original_line[idx..]);
                        }
                    }
                }

                lines[update.line_number] = new_line;
            }
        }
    }

    write_lines(&path, &lines)
}
