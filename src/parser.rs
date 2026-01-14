use anyhow::{Context, Result};
use dirs::config_dir;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: String,
    pub clean_mods: String,
    pub flags: String,
    pub key: String,
    pub dispatcher: String,
    pub args: String,
    pub submap: Option<String>,
    pub line_number: usize,
    pub file_path: PathBuf,
}

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(env_path) = std::env::var("HYPRKCS_CONFIG") {
        return Ok(PathBuf::from(env_path));
    }

    let mut path = config_dir().context("Could not find config directory")?;
    path.push("hypr");
    path.push("hyprland.conf");
    Ok(path)
}

fn resolve_variables(
    input: &str,
    vars: &HashMap<String, String>,
    sorted_keys: &[String],
) -> String {
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
            return home.join(&path_str[2..]);
        }
    }

    let p = PathBuf::from(path_str);
    if p.is_absolute() {
        p
    } else {
        current_file.parent().unwrap_or(&PathBuf::from(".")).join(p)
    }
}

pub fn get_variables() -> Result<HashMap<String, String>> {
    let main_path = get_config_path()?;
    let mut variables = HashMap::new();
    let mut visited = HashSet::new();

    fn collect_recursive(
        path: PathBuf,
        vars: &mut HashMap<String, String>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        if !path.exists() || visited.contains(&path) {
            return Ok(());
        }
        visited.insert(path.clone());

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let var_re = Regex::new(r"^\s*(\$[a-zA-Z0-9_-]+)\s*=\s*(.*)").unwrap();
        let source_re = Regex::new(r"^\s*source\s*=\s*(.*)").unwrap();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Prepare sorted keys from current variables for resolution
            // This happens per-line but only for variable/source lines, which is acceptable
            let mut sorted_keys: Vec<_> = vars.keys().cloned().collect();
            sorted_keys.sort_by(|a, b| b.len().cmp(&a.len()));

            if let Some(caps) = var_re.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let raw_value_full = caps.get(2).unwrap().as_str();
                let raw_value = raw_value_full.split('#').next().unwrap_or("").trim();
                let value = resolve_variables(raw_value, vars, &sorted_keys);
                vars.insert(name, value);
            } else if let Some(caps) = source_re.captures(line) {
                let path_str = caps
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim();
                let sourced_path = expand_path(path_str, &path, vars, &sorted_keys);
                let _ = collect_recursive(sourced_path, vars, visited);
            }
        }
        Ok(())
    }

    collect_recursive(main_path, &mut variables, &mut visited)?;
    Ok(variables)
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    let main_path = get_config_path()?;
    let mut keybinds = Vec::new();
    let variables = get_variables()?;

    // Sort keys ONCE for the entire parsing process
    let mut sorted_keys: Vec<_> = variables.keys().cloned().collect();
    sorted_keys.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut visited = HashSet::new();
    let mut current_submap: Option<String> = None;

    fn parse_recursive(
        path: PathBuf,
        keybinds: &mut Vec<Keybind>,
        variables: &HashMap<String, String>,
        sorted_keys: &[String],
        visited: &mut HashSet<PathBuf>,
        current_submap: &mut Option<String>,
    ) -> Result<()> {
        if !path.exists() || visited.contains(&path) {
            return Ok(());
        }
        visited.insert(path.clone());

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        // Regex to match "bind" or "bindl", "binde" etc, and capture the flags + the rest of the line
        let bind_re = Regex::new(r"^\s*bind([a-zA-Z]*)\s*=\s*(.*)$").unwrap();
        let source_re = Regex::new(r"^\s*source\s*=\s*(.*)$").unwrap();
        let submap_re = Regex::new(r"^\s*submap\s*=\s*(.*)$").unwrap();

        for (index, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            if let Some(caps) = submap_re.captures(line_trimmed) {
                let name = caps.get(1).map_or("", |m| m.as_str()).trim();
                if name == "reset" {
                    *current_submap = None;
                } else {
                    *current_submap = Some(name.to_string());
                }
            } else if let Some(caps) = bind_re.captures(line_trimmed) {
                let flags = caps.get(1).map_or("", |m| m.as_str()).trim();
                let raw_content = caps.get(2).map_or("", |m| m.as_str()).trim();

                // 1. Resolve variables in the content string using PRE-SORTED keys
                let resolved_content = resolve_variables(raw_content, variables, sorted_keys);

                // 2. Strip comments
                let content_clean = resolved_content.split('#').next().unwrap_or("").trim();

                // 3. Split by comma to get arguments
                // Limit 4 because: Mod, Key, Dispatcher, Args
                let parts: Vec<&str> = content_clean.splitn(4, ',').map(|s| s.trim()).collect();

                if parts.len() >= 3 {
                    let mods = parts[0].to_string();
                    let key = parts[1].to_string();
                    let dispatcher = parts[2].to_string();
                    let args = if parts.len() > 3 {
                        parts[3].to_string()
                    } else {
                        String::new()
                    };

                    keybinds.push(Keybind {
                        mods: mods.clone(), // We display the resolved mods
                        clean_mods: mods,   // And store resolved mods
                        flags: flags.to_string(),
                        key,
                        dispatcher,
                        args,
                        submap: current_submap.clone(),
                        line_number: index,
                        file_path: path.clone(),
                    });
                }
            } else if let Some(caps) = source_re.captures(line_trimmed) {
                let path_str = caps
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim();
                let sourced_path = expand_path(path_str, &path, variables, sorted_keys);
                let _ = parse_recursive(
                    sourced_path,
                    keybinds,
                    variables,
                    sorted_keys,
                    visited,
                    current_submap,
                );
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
) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    let original_line = &lines[line_number];
    let re = Regex::new(r"^(\s*bind)([a-zA-Z]*\s*=\s*)(.*)$").unwrap();

    if let Some(caps) = re.captures(original_line) {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let flags_eq = caps.get(2).map_or("", |m| m.as_str());

        let new_line = if new_args.trim().is_empty() {
            format!(
                "{}{} {}, {}, {}",
                prefix, flags_eq, new_mods, new_key, new_dispatcher
            )
        } else {
            format!(
                "{}{} {}, {}, {}, {}",
                prefix, flags_eq, new_mods, new_key, new_dispatcher, new_args
            )
        };

        lines[line_number] = new_line;
        std::fs::write(&path, lines.join("\n"))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Could not parse original line structure"))
    }
}

pub fn add_keybind(
    path: PathBuf,
    mods: &str,
    key: &str,
    dispatcher: &str,
    args: &str,
    submap: Option<String>,
) -> Result<usize> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() {
        vec![]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    };

    let new_line = if args.trim().is_empty() {
        format!("bind = {}, {}, {}", mods, key, dispatcher)
    } else {
        format!("bind = {}, {}, {}, {}", mods, key, dispatcher, args)
    };

    if let Some(submap_name) = submap.filter(|s| !s.is_empty()) {
        let submap_decl = format!("submap = {}", submap_name);
        let mut found_submap = false;
        let mut insert_index = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == submap_decl {
                found_submap = true;
                // Look ahead for the end of this submap
                for j in (i + 1)..lines.len() {
                    let next_trimmed = lines[j].trim();
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
