use anyhow::{Context, Result};
use dirs::config_dir;
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod input;

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: Arc<str>,
    pub clean_mods: Arc<str>,
    pub flags: Arc<str>,
    pub key: Arc<str>,
    pub dispatcher: Arc<str>,
    pub args: Arc<str>,
    pub description: Option<Arc<str>>,
    pub submap: Option<Arc<str>>,
    pub line_number: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Arc<str>,
    pub value: Arc<str>,
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
    file_cache: HashMap<PathBuf, Arc<String>>,
    mtimes: HashMap<PathBuf, std::time::SystemTime>,
    sizes: HashMap<PathBuf, u64>,
}

fn load_config_data() -> Result<ConfigData> {
    let main_path = get_config_path()?;
    let mut ctx = ParserContext::new();

    // Inject $hypr variable pointing to the config root
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

    let mut file_cache: HashMap<PathBuf, Arc<String>> = HashMap::new();
    let mut path_cache: HashMap<PathBuf, Arc<PathBuf>> = HashMap::new();
    let mut defined_variables = Vec::new();
    let mut mtimes: HashMap<PathBuf, std::time::SystemTime> = HashMap::new();
    let mut sizes: HashMap<PathBuf, u64> = HashMap::new();

    fn collect_recursive(
        path: PathBuf,
        ctx: &mut ParserContext,
        file_cache: &mut HashMap<PathBuf, Arc<String>>,
        path_cache: &mut HashMap<PathBuf, Arc<PathBuf>>,
        defined_variables: &mut Vec<Variable>,
        mtimes: &mut HashMap<PathBuf, std::time::SystemTime>,
        sizes: &mut HashMap<PathBuf, u64>,
        system_root: &Path,
        active_root: &Path,
    ) -> Result<()> {
        if ctx.visited.contains(&path) {
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }

        ctx.visited.insert(path.clone());

        if path.is_dir() {
            // Track directory mtime/size to detect additions/removals
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mtime = metadata
                    .modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now());
                mtimes.insert(path.clone(), mtime);
                sizes.insert(path.clone(), metadata.len());
            }

            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
                paths.sort();
                for sub_path in paths {
                    let _ = collect_recursive(
                        sub_path,
                        ctx,
                        file_cache,
                        path_cache,
                        defined_variables,
                        mtimes,
                        sizes,
                        system_root,
                        active_root,
                    );
                }
            }
            return Ok(());
        }

        let shared_path = path_cache
            .entry(path.clone())
            .or_insert_with(|| Arc::new(path.clone()))
            .clone();

        let content = if let Some(cached) = file_cache.get(&path) {
            cached.clone()
        } else {
            let metadata = std::fs::metadata(&path)?;
            let mtime = metadata
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::now());
            mtimes.insert(path.clone(), mtime);
            sizes.insert(path.clone(), metadata.len());

            let s = std::fs::read_to_string(&path).unwrap_or_default();
            let rc = Arc::new(s);
            file_cache.insert(path.clone(), rc.clone());
            rc
        };

        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('$') {
                if let Some((name_part, value_part)) = line.split_once('=') {
                    let name = name_part.trim().to_string();
                    let (raw_value_part, _) = split_comment(value_part);
                    let raw_value = raw_value_part.trim();

                    if !name.is_empty() {
                        defined_variables.push(Variable {
                            name: Arc::from(name.as_str()),
                            value: Arc::from(raw_value),
                            line_number: line_idx,
                            file_path: (*shared_path).clone(),
                        });

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
            } else if let Some(rest) = line.strip_prefix("source") {
                let trimmed_rest = rest.trim_start();
                if let Some(path_part) = trimmed_rest.strip_prefix('=') {
                    let path_str = path_part
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"');

                    let mut sourced_path = if path_str.contains('$') {
                        ctx.ensure_sorted();
                        expand_path(path_str, &path, &ctx.variables, &ctx.sorted_keys)
                    } else {
                        expand_path(path_str, &path, &ctx.variables, &[])
                    };

                    if !sourced_path.exists() && sourced_path.ends_with(".conf") {
                        if let Some(parent) = sourced_path.parent() {
                            sourced_path = parent.join("*.conf");
                        }
                    }

                    if system_root != active_root {
                        if let Ok(suffix) = sourced_path.strip_prefix(system_root) {
                            let remapped = active_root.join(suffix);
                            let remapped_str = remapped.to_string_lossy();
                            let is_glob = is_glob_pattern(&remapped_str);

                            if remapped.exists()
                                || (is_glob
                                    && glob(&remapped_str).is_ok_and(|mut p| p.next().is_some()))
                            {
                                sourced_path = remapped;
                            }
                        }
                    }

                    let pattern = sourced_path.to_string_lossy();
                    if !is_glob_pattern(&pattern) {
                        let _ = collect_recursive(
                            sourced_path,
                            ctx,
                            file_cache,
                            path_cache,
                            defined_variables,
                            mtimes,
                            sizes,
                            system_root,
                            active_root,
                        );
                    } else if let Ok(paths) = glob(&pattern) {
                        // Track the parent directory so new files matching
                        // the glob pattern will invalidate the cache.
                        if let Some(parent) = sourced_path.parent() {
                            if parent.is_dir() {
                                if let Ok(dir_meta) = std::fs::metadata(parent) {
                                    let dir_mtime = dir_meta
                                        .modified()
                                        .unwrap_or_else(|_| std::time::SystemTime::now());
                                    mtimes.insert(parent.to_path_buf(), dir_mtime);
                                    sizes.insert(parent.to_path_buf(), dir_meta.len());
                                }
                            }
                        }
                        for p in paths.flatten() {
                            let _ = collect_recursive(
                                p,
                                ctx,
                                file_cache,
                                path_cache,
                                defined_variables,
                                mtimes,
                                sizes,
                                system_root,
                                active_root,
                            );
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
        &mut mtimes,
        &mut sizes,
        &system_root,
        &active_root,
    )?;
    Ok(ConfigData {
        variables: ctx.variables,
        defined_variables,
        file_cache,
        mtimes,
        sizes,
    })
}

#[derive(Clone)]
struct CacheState {
    keybinds: Vec<Keybind>,
    variables: HashMap<String, String>,
    defined_variables: Vec<Variable>,
    loaded_files: Vec<PathBuf>,
    mtimes: HashMap<PathBuf, std::time::SystemTime>,
    sizes: HashMap<PathBuf, u64>,
    main_path: PathBuf,
}

static GLOBAL_CACHE: Mutex<Option<CacheState>> = Mutex::new(None);

pub fn invalidate_parser_cache() {
    if let Ok(mut cache) = GLOBAL_CACHE.lock() {
        *cache = None;
    }
}

fn get_valid_cache() -> Result<Option<CacheState>> {
    let main_path = get_config_path()?;

    // 1. Get snapshot of validation data under lock to minimize lock contention
    let (mtimes, sizes, cache) = {
        if let Ok(guard) = GLOBAL_CACHE.lock() {
            if let Some(cache) = guard.as_ref() {
                if cache.main_path != main_path {
                    return Ok(None);
                }
                (cache.mtimes.clone(), cache.sizes.clone(), cache.clone())
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }
    };

    // 2. Perform slow I/O without holding the lock
    let mut valid = true;
    for (path, last_mtime) in &mtimes {
        let last_size = sizes.get(path).cloned().unwrap_or(0);
        match std::fs::metadata(path) {
            Ok(m) => {
                let mtime = m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if mtime != *last_mtime || m.len() != last_size {
                    valid = false;
                    break;
                }
            }
            _ => {
                valid = false;
                break;
            }
        }
    }

    if valid {
        Ok(Some(cache))
    } else {
        Ok(None)
    }
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    if let Some(cache) = get_valid_cache()? {
        return Ok(cache.keybinds);
    }

    let main_path = get_config_path()?;
    let data = load_config_data()?;
    let variables = data.variables;
    let file_cache = data.file_cache;
    let mtimes = data.mtimes;
    let sizes = data.sizes;
    let defined_variables = data.defined_variables;

    let mut sorted_keys: Vec<_> = variables.keys().cloned().collect();
    sorted_keys.sort_by_key(|b: &String| std::cmp::Reverse(b.len()));

    let mut keybinds = Vec::new();
    let mut visited = HashSet::new();
    let mut current_submap: Option<Arc<str>> = None;

    fn parse_recursive(
        path: PathBuf,
        keybinds: &mut Vec<Keybind>,
        ctx: &RecursiveParseContext,
        visited: &mut HashSet<PathBuf>,
        current_submap: &mut Option<Arc<str>>,
    ) -> Result<()> {
        if visited.contains(&path) {
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }
        visited.insert(path.clone());

        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
                paths.sort();
                for sub_path in paths {
                    let _ = parse_recursive(sub_path, keybinds, ctx, visited, current_submap);
                }
            }
            return Ok(());
        }

        let content = if let Some(cached) = ctx.file_cache.get(&path) {
            cached.clone()
        } else {
            Arc::new(std::fs::read_to_string(&path).unwrap_or_default())
        };

        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            if let Some(rest) = line_trimmed.strip_prefix("submap") {
                let rest_trimmed = rest.trim_start();
                if let Some(val) = rest_trimmed.strip_prefix('=') {
                    let name = val.split('#').next().unwrap_or("").trim();
                    if name == "reset" {
                        *current_submap = None;
                    } else {
                        *current_submap = Some(Arc::from(name));
                    }
                }
            } else if let Some(rest) = line_trimmed.strip_prefix("bind") {
                let rest = rest.trim_start();
                let flags;
                let mut remaining = rest;

                if let Some(eq_idx) = remaining.find('=') {
                    let potential_flags = remaining[..eq_idx].trim();
                    if potential_flags.chars().all(|c| c.is_alphabetic()) {
                        flags = potential_flags.to_string();
                        remaining = &remaining[eq_idx + 1..];
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }

                let raw_content = remaining.trim();
                let mut description = None;

                if let Some(idx) = line.find('#') {
                    let comment = line[idx + 1..].trim();
                    if !comment.is_empty() {
                        description = Some(Arc::from(comment));
                    }
                }

                if description.is_none() && index > 0 {
                    let prev_line = lines[index - 1].trim();
                    if prev_line.starts_with('#') {
                        let comment = prev_line.trim_start_matches('#').trim();
                        if !comment.is_empty() {
                            description = Some(Arc::from(comment));
                        }
                    }
                }

                let resolved_content =
                    resolve_variables(raw_content, ctx.variables, ctx.sorted_keys);
                let content_clean = resolved_content.split('#').next().unwrap_or("").trim();

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
                        current_part.push(c);
                    }
                }

                if !current_part.trim().is_empty() || parts_count >= limit {
                    parts.push(current_part.trim().to_string());
                }

                if parts.len() >= 3 {
                    let mods: Arc<str>;
                    let key: Arc<str>;
                    let dispatcher: Arc<str>;
                    let args: Arc<str>;

                    if is_bindd {
                        mods = Arc::from(parts[0].as_str());
                        key = Arc::from(parts[1].as_str());
                        if parts.len() > 2 {
                            let desc_str = parts[2].trim();
                            if !desc_str.is_empty() {
                                description = Some(Arc::from(desc_str));
                            }
                        }
                        dispatcher = if parts.len() > 3 {
                            Arc::from(parts[3].as_str())
                        } else {
                            Arc::from("")
                        };
                        args = if parts.len() > 4 {
                            Arc::from(parts[4].as_str())
                        } else {
                            Arc::from("")
                        };
                    } else {
                        mods = Arc::from(parts[0].as_str());
                        key = Arc::from(parts[1].as_str());
                        dispatcher = Arc::from(parts[2].as_str());
                        args = if parts.len() > 3 {
                            Arc::from(parts[3].as_str())
                        } else {
                            Arc::from("")
                        };
                    }

                    keybinds.push(Keybind {
                        mods: mods.clone(),
                        clean_mods: mods,
                        flags: Arc::from(flags.as_str()),
                        key,
                        dispatcher,
                        args,
                        description,
                        submap: current_submap.clone(),
                        line_number: index,
                        file_path: path.clone(),
                    });
                }
            } else if let Some(rest) = line_trimmed.strip_prefix("source") {
                let trimmed_rest = rest.trim_start();
                if let Some(path_part) = trimmed_rest.strip_prefix('=') {
                    let path_str = path_part
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"');

                    let mut sourced_path =
                        expand_path(path_str, &path, ctx.variables, ctx.sorted_keys);

                    if !sourced_path.exists() && sourced_path.ends_with(".conf") {
                        if let Some(parent) = sourced_path.parent() {
                            sourced_path = parent.join("*.conf");
                        }
                    }

                    if ctx.system_root != ctx.active_root {
                        if let Ok(suffix) = sourced_path.strip_prefix(ctx.system_root) {
                            let remapped = ctx.active_root.join(suffix);
                            let remapped_str = remapped.to_string_lossy();
                            let is_glob = is_glob_pattern(&remapped_str);

                            if remapped.exists()
                                || (is_glob
                                    && glob(&remapped_str).is_ok_and(|mut p| p.next().is_some()))
                            {
                                sourced_path = remapped;
                            }
                        }
                    }

                    let pattern = sourced_path.to_string_lossy();
                    if !is_glob_pattern(&pattern) {
                        let _ =
                            parse_recursive(sourced_path, keybinds, ctx, visited, current_submap);
                    } else if let Ok(paths) = glob(&pattern) {
                        for p in paths.flatten() {
                            let _ = parse_recursive(p, keybinds, ctx, visited, current_submap);
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

    let ctx = RecursiveParseContext {
        variables: &variables,
        sorted_keys: &sorted_keys,
        file_cache: &file_cache,
        system_root: &system_root,
        active_root: &active_root,
    };

    parse_recursive(
        main_path.clone(),
        &mut keybinds,
        &ctx,
        &mut visited,
        &mut current_submap,
    )?;

    let loaded_files = file_cache.keys().cloned().collect();
    let cache_state = CacheState {
        keybinds: keybinds.clone(),
        variables: variables.clone(),
        defined_variables,
        loaded_files,
        mtimes,
        sizes,
        main_path,
    };

    if let Ok(mut guard) = GLOBAL_CACHE.lock() {
        *guard = Some(cache_state);
    }

    Ok(keybinds)
}

pub fn get_variables() -> Result<HashMap<String, String>> {
    if let Some(cache) = get_valid_cache()? {
        return Ok(cache.variables);
    }
    let data = load_config_data()?;
    Ok(data.variables)
}

pub fn get_defined_variables() -> Result<Vec<Variable>> {
    if let Some(cache) = get_valid_cache()? {
        return Ok(cache.defined_variables);
    }
    let data = load_config_data()?;
    Ok(data.defined_variables)
}

pub fn get_loaded_files() -> Result<Vec<PathBuf>> {
    if let Some(cache) = get_valid_cache()? {
        return Ok(cache.loaded_files);
    }
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
            invalidate_parser_cache();
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
            invalidate_parser_cache();
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
    std::fs::write(path, content).context("Failed to write to file")?;
    invalidate_parser_cache();
    Ok(())
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

struct RecursiveParseContext<'a> {
    variables: &'a HashMap<String, String>,
    sorted_keys: &'a [String],
    file_cache: &'a HashMap<PathBuf, Arc<String>>,
    system_root: &'a Path,
    active_root: &'a Path,
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
    let indent_len = original_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    let indent = &original_line[..indent_len];
    let trimmed_start = &original_line[indent_len..];

    if let Some(after_bind) = trimmed_start.strip_prefix("bind") {
        if let Some(_eq_idx) = after_bind.find('=') {
            let current_flags = after_bind[.._eq_idx].trim();
            let flags = new_flags.unwrap_or(current_flags);
            let is_bindd = flags == "d";

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
                    if let Some(idx) = original_line.find('#') {
                        new_line = format!("{} {}", new_line, &original_line[idx..]);
                    }
                }
            }

            lines[line_number] = new_line;
            write_lines(&path, &lines)
        } else {
            Err(anyhow::anyhow!("Could not parse original line structure"))
        }
    } else {
        Err(anyhow::anyhow!("Not a bind line"))
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
                for (j, line_j) in lines.iter().enumerate().skip(i + 1) {
                    let next_trimmed = line_j.trim();
                    if next_trimmed.starts_with("submap =") {
                        insert_index = Some(j);
                        break;
                    }
                }
                if insert_index.is_none() {
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
            lines.push(new_line);
            write_lines(&path, &lines)?;
            Ok(lines.len() - 1)
        } else {
            lines.push(String::new());
            lines.push(submap_decl);
            lines.push(new_line);
            lines.push("submap = reset".to_string());
            write_lines(&path, &lines)?;
            Ok(lines.len() - 2)
        }
    } else {
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
            continue;
        }
        let original_line = &lines[update.line_number];
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
                    } else if let Some(idx) = original_line.find('#') {
                        new_line = format!("{} {}", new_line, &original_line[idx..]);
                    }
                }
                lines[update.line_number] = new_line;
            }
        }
    }
    write_lines(&path, &lines)
}
