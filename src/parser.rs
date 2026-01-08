use anyhow::{Context, Result};
use dirs::config_dir;
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: String,
    pub clean_mods: String,
    pub flags: String,
    pub key: String,
    pub dispatcher: String,
    pub args: String,
    pub line_number: usize,
    pub file_path: PathBuf,
}

pub fn get_config_path() -> Result<PathBuf> {
    let mut path = config_dir().context("Could not find config directory")?;
    path.push("hypr");
    path.push("hyprland.conf");
    Ok(path)
}

fn expand_path(path_str: &str, current_file: &PathBuf) -> PathBuf {
    let path_str = path_str.trim();
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

pub fn get_variables() -> Result<std::collections::HashMap<String, String>> {
    let main_path = get_config_path()?;
    let mut variables = std::collections::HashMap::new();
    let mut visited = std::collections::HashSet::new();
    
    fn collect_vars(path: PathBuf, vars: &mut std::collections::HashMap<String, String>, visited: &mut std::collections::HashSet<PathBuf>) -> Result<()> {
        if !path.exists() || visited.contains(&path) {
            return Ok(());
        }
        visited.insert(path.clone());
        
        let content = std::fs::read_to_string(&path)?;
        let var_re = Regex::new(r"^\s*\$([a-zA-Z0-0_-]+)\s*=\s*(.*)").unwrap();
        let source_re = Regex::new(r"^\s*source\s*=\s*(.*)").unwrap();

        for line in content.lines() {
            let line = line.trim();
            if let Some(caps) = var_re.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let value = caps.get(2).unwrap().as_str().trim().to_string();
                vars.insert(format!("${}", name), value);
            } else if let Some(caps) = source_re.captures(line) {
                let sourced_path = expand_path(caps.get(1).unwrap().as_str(), &path);
                let _ = collect_vars(sourced_path, vars, visited);
            }
        }
        Ok(())
    }

    collect_vars(main_path, &mut variables, &mut visited)?;
    Ok(variables)
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    let main_path = get_config_path()?;
    let mut keybinds = Vec::new();
    let variables = get_variables()?;
    let mut visited = std::collections::HashSet::new();

    fn parse_recursive(
        path: PathBuf, 
        keybinds: &mut Vec<Keybind>, 
        variables: &std::collections::HashMap<String, String>,
        visited: &mut std::collections::HashSet<PathBuf>
    ) -> Result<()> {
        if !path.exists() || visited.contains(&path) {
            return Ok(());
        }
        visited.insert(path.clone());

        let content = std::fs::read_to_string(&path)?;
        let bind_re = Regex::new(r"^\s*bind([a-z]*)\s*=\s*([^,]*)\s*,\s*([^,]+)\s*,\s*([^,]+)(?:\s*,\s*(.*))?").unwrap();
        let source_re = Regex::new(r"^\s*source\s*=\s*(.*)").unwrap();

        for (index, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            if let Some(caps) = bind_re.captures(line_trimmed) {
                let flags = caps.get(1).map_or("", |m| m.as_str()).trim();
                let raw_mods = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
                let key = caps.get(3).map_or("", |m| m.as_str()).trim().to_string();
                let dispatcher = caps.get(4).map_or("", |m| m.as_str()).trim().to_string();
                let args = caps.get(5).map_or("", |m| m.as_str()).trim().to_string();

                let mut resolved_mods = raw_mods.clone();
                for (var, val) in variables {
                    if resolved_mods.contains(var) {
                        resolved_mods = resolved_mods.replace(var, val);
                    }
                }

                let display_mods = if flags.is_empty() {
                    raw_mods.clone()
                } else {
                    format!("[{}] {}", flags, raw_mods)
                };

                let args = if let Some(idx) = args.find('#') {
                    args[..idx].trim().to_string()
                } else {
                    args
                };

                keybinds.push(Keybind {
                    mods: display_mods,
                    clean_mods: resolved_mods,
                    flags: flags.to_string(),
                    key,
                    dispatcher,
                    args,
                    line_number: index,
                    file_path: path.clone(),
                });
            } else if let Some(caps) = source_re.captures(line_trimmed) {
                let sourced_path = expand_path(caps.get(1).unwrap().as_str(), &path);
                let _ = parse_recursive(sourced_path, keybinds, variables, visited);
            }
        }
        Ok(())
    }

    parse_recursive(main_path, &mut keybinds, &variables, &mut visited)?;
    Ok(keybinds)
}

pub fn update_line(path: PathBuf, line_number: usize, new_mods: &str, new_key: &str, new_dispatcher: &str, new_args: &str) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    let original_line = &lines[line_number];
    let re = Regex::new(r"^(\s*bind)([a-z]*\s*=\s*)([^,]*),\s*([^,]+)\s*,\s*([^,]+)(.*)$").unwrap();
    
    if let Some(caps) = re.captures(original_line) {
         let prefix = caps.get(1).map_or("", |m| m.as_str());
         let flags_eq = caps.get(2).map_or("", |m| m.as_str());
         
         let new_line = if new_args.trim().is_empty() {
             format!("{}{}{}, {}, {}", prefix, flags_eq, new_mods, new_key, new_dispatcher)
         } else {
             format!("{}{}{}, {}, {}, {}", prefix, flags_eq, new_mods, new_key, new_dispatcher, new_args)
         };
         
         lines[line_number] = new_line;
         std::fs::write(&path, lines.join("\n"))?;
         Ok(())
    } else {
         Err(anyhow::anyhow!("Could not parse original line structure"))
    }
}

pub fn add_keybind(path: PathBuf, mods: &str, key: &str, dispatcher: &str, args: &str) -> Result<usize> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() { vec![] } else { content.lines().map(|s| s.to_string()).collect() };

    let new_line = if args.trim().is_empty() {
        format!("bind = {}, {}, {}", mods, key, dispatcher)
    } else {
        format!("bind = {}, {}, {}, {}", mods, key, dispatcher, args)
    };

    lines.push(new_line);
    std::fs::write(&path, lines.join("\n"))?;
    
    Ok(lines.len() - 1)
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