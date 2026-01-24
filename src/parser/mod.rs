use anyhow::{Context, Result};
use dirs::config_dir;
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

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(env_path) = std::env::var("HYPRKCS_CONFIG") {
        return Ok(PathBuf::from(env_path));
    }

    let mut path = config_dir().context("Could not find config directory")?;
    path.push(crate::config::constants::HYPR_DIR);
    path.push(crate::config::constants::HYPRLAND_CONF);
    Ok(path)
}

fn resolve_variables(
    input: &str,
    vars: &HashMap<String, String>,
    sorted_keys: &[String],
) -> String {
    if !input.contains('$') {
        return input.to_string();
    }
    let mut result = input.to_string();
    for key in sorted_keys {
        if result.contains(key) {
            result = result.replace(key, &vars[key]);
        }
    }
    result
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
        if let Some(home) = dirs::home_dir() {
            let s: &str = &path_str[2..];
            return home.join(s);
        }
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
}

impl ParserContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            sorted_keys: Vec::new(),
            visited: HashSet::new(),
        }
    }

    fn update_sorted_keys(&mut self) {
        // Only update if variable count changed to save work
        if self.sorted_keys.len() != self.variables.len() {
            self.sorted_keys = self.variables.keys().cloned().collect();
            self.sorted_keys
                .sort_by_key(|b: &String| std::cmp::Reverse(b.len()));
        }
    }
}

struct ConfigData {
    variables: HashMap<String, String>,
    file_cache: HashMap<PathBuf, Rc<String>>,
}

fn load_config_data() -> Result<ConfigData> {
    let main_path = get_config_path()?;
    let mut ctx = ParserContext::new();
    let mut file_cache = HashMap::new();

    fn collect_recursive(
        path: PathBuf,
        ctx: &mut ParserContext,
        file_cache: &mut HashMap<PathBuf, Rc<String>>,
    ) -> Result<()> {
        if !path.exists() || ctx.visited.contains(&path) {
            return Ok(());
        }
        ctx.visited.insert(path.clone());

        let content = if let Some(cached) = file_cache.get(&path) {
            cached.clone()
        } else {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            let rc = Rc::new(s);
            file_cache.insert(path.clone(), rc.clone());
            rc
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Variable parsing: $name = value
            if line.starts_with('$') {
                if let Some((name_part, value_part)) = line.split_once('=') {
                    let name = name_part.trim().to_string();
                    let raw_value = value_part.split('#').next().unwrap_or("").trim();

                    if !name.is_empty() {
                        ctx.update_sorted_keys();
                        let value = resolve_variables(raw_value, &ctx.variables, &ctx.sorted_keys);
                        ctx.variables.insert(name, value);
                    }
                }
            }
            // Source parsing: source = path
            else if let Some(rest) = line.strip_prefix("source") {
                let trimmed_rest = rest.trim_start();
                if let Some(path_part) = trimmed_rest.strip_prefix('=') {
                    let path_str = path_part.split('#').next().unwrap_or("").trim();

                    ctx.update_sorted_keys();
                    let sourced_path =
                        expand_path(path_str, &path, &ctx.variables, &ctx.sorted_keys);
                    let _ = collect_recursive(sourced_path, ctx, file_cache);
                }
            }
        }
        Ok(())
    }

    collect_recursive(main_path, &mut ctx, &mut file_cache)?;
    Ok(ConfigData {
        variables: ctx.variables,
        file_cache,
    })
}

pub fn get_variables() -> Result<HashMap<String, String>> {
    let data = load_config_data()?;
    Ok(data.variables)
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
    ) -> Result<()> {
        if !path.exists() || visited.contains(&path) {
            return Ok(());
        }
        visited.insert(path.clone());

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
                let mut parts = Vec::new();
                let mut current_part = String::new();
                let mut in_quote = false;
                let mut parts_count = 0;

                let chars: Vec<char> = content_clean.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let c = chars[i];

                    if parts_count < 3 {
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
                        // For the 4th part (args), just take everything else
                        current_part.push(c);
                    }
                    i += 1;
                }
                if !current_part.trim().is_empty() || parts_count >= 3 {
                    parts.push(current_part.trim().to_string());
                }

                if parts.len() >= 3 {
                    let mods: Rc<str> = Rc::from(parts[0].as_str());
                    let key: Rc<str> = Rc::from(parts[1].as_str());
                    let dispatcher: Rc<str> = Rc::from(parts[2].as_str());
                    let args: Rc<str> = if parts.len() > 3 {
                        Rc::from(parts[3].as_str())
                    } else {
                        Rc::from("")
                    };

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
                    let path_str = path_part.split('#').next().unwrap_or("").trim();

                    let sourced_path = expand_path(path_str, &path, variables, sorted_keys);
                    let _ = parse_recursive(
                        sourced_path,
                        keybinds,
                        variables,
                        sorted_keys,
                        visited,
                        current_submap,
                        file_cache,
                    );
                }
            }
        }
        Ok(())
    }

    parse_recursive(
        main_path,
        &mut keybinds,
        &variables,
        &sorted_keys,
        &mut visited,
        &mut current_submap,
        &file_cache,
    )?;
    Ok(keybinds)
}

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
    // regex was: r"^(\s*)bind([a-zA-Z]*)(\s*=\s*)([^#]*)"

    // 1. Indent
    let indent_len = original_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    let indent = &original_line[..indent_len];
    let trimmed_start = &original_line[indent_len..];

    if trimmed_start.starts_with("bind") {
        let after_bind = &trimmed_start[4..];
        if let Some(eq_idx) = after_bind.find('=') {
            let current_flags = after_bind[..eq_idx].trim();
            
            let flags = new_flags.unwrap_or(current_flags);

            // preserve existing spacing around equals if possible, or just standard " = "
            // The original code reconstructed the line completely anyway.

            let mut new_line = if new_args.trim().is_empty() {
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

            lines[line_number] = new_line;
            std::fs::write(&path, lines.join("\n"))?;
            Ok(())
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

    let bind_cmd = if flags.is_empty() {
        "bind".to_string()
    } else {
        format!("bind{}", flags)
    };

    let mut new_line = if args.trim().is_empty() {
        format!("{} = {}, {}, {}", bind_cmd, mods, key, dispatcher)
    } else {
        format!("{} = {}, {}, {}, {}", bind_cmd, mods, key, dispatcher, args)
    };

    if let Some(desc) = description {
        if !desc.trim().is_empty() {
            new_line = format!("{} # {}", new_line, desc.trim());
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
            std::fs::write(&path, lines.join("\n"))?;
            Ok(idx)
        } else if found_submap {
            // Should have been handled above, but fallback
            lines.push(new_line);
            std::fs::write(&path, lines.join("\n"))?;
            Ok(lines.len() - 1)
        } else {
            // Submap doesn't exist, create it
            lines.push(String::new()); // spacer
            lines.push(submap_decl);
            lines.push(new_line);
            lines.push("submap = reset".to_string());

            std::fs::write(&path, lines.join("\n"))?;
            Ok(lines.len() - 2) // Index of the new bind
        }
    } else {
        // Global map
        lines.push(new_line);
        std::fs::write(&path, lines.join("\n"))?;
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
    std::fs::write(&path, lines.join("\n"))?;

    Ok(())
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

        if trimmed_start.starts_with("bind") {
            let after_bind = &trimmed_start[4..];
            if let Some(eq_idx) = after_bind.find('=') {
                let flags = after_bind[..eq_idx].trim();

                let mut new_line = if update.new_args.trim().is_empty() {
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

                lines[update.line_number] = new_line;
            }
        }
    }

    std::fs::write(&path, lines.join("\n"))?;
    Ok(())
}
