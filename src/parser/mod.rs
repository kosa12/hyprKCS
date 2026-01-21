use anyhow::{Context, Result};
use dirs::config_dir;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub mod input;

static VAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(\$[a-zA-Z0-9_-]+)\s*=\s*(.*)").unwrap());
static SOURCE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*source\s*=\s*(.*)").unwrap());
static BIND_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*bind([a-zA-Z]*)\s*=\s*(.*)$").unwrap());
static SUBMAP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*submap\s*=\s*(.*)$").unwrap());

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: String,
    pub clean_mods: String,
    pub flags: String,
    pub key: String,
    pub dispatcher: String,
    pub args: String,
    pub description: Option<String>,
    pub submap: Option<String>,
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
            self.sorted_keys.sort_by_key(|b| std::cmp::Reverse(b.len()));
        }
    }
}

pub fn get_variables() -> Result<HashMap<String, String>> {
    let main_path = get_config_path()?;
    let mut ctx = ParserContext::new();

    fn collect_recursive(path: PathBuf, ctx: &mut ParserContext) -> Result<()> {
        if !path.exists() || ctx.visited.contains(&path) {
            return Ok(());
        }
        ctx.visited.insert(path.clone());

        let content = std::fs::read_to_string(&path).unwrap_or_default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(caps) = VAR_RE.captures(line) {
                let name = caps.get(1).unwrap().as_str().to_string();
                let raw_value_full = caps.get(2).unwrap().as_str();
                let raw_value = raw_value_full.split('#').next().unwrap_or("").trim();

                ctx.update_sorted_keys();
                let value = resolve_variables(raw_value, &ctx.variables, &ctx.sorted_keys);
                ctx.variables.insert(name, value);
            } else if let Some(caps) = SOURCE_RE.captures(line) {
                let path_str = caps
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim();

                ctx.update_sorted_keys();
                let sourced_path = expand_path(path_str, &path, &ctx.variables, &ctx.sorted_keys);
                let _ = collect_recursive(sourced_path, ctx);
            }
        }
        Ok(())
    }

    collect_recursive(main_path, &mut ctx)?;
    Ok(ctx.variables)
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    let main_path = get_config_path()?;
    let variables = get_variables()?;
    let mut sorted_keys: Vec<_> = variables.keys().cloned().collect();
    sorted_keys.sort_by_key(|b| std::cmp::Reverse(b.len()));

    let mut keybinds = Vec::new();
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
        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            if let Some(caps) = SUBMAP_RE.captures(line_trimmed) {
                let name = caps.get(1).map_or("", |m| m.as_str()).trim();
                if name == "reset" {
                    *current_submap = None;
                } else {
                    *current_submap = Some(name.to_string());
                }
            } else if let Some(caps) = BIND_RE.captures(line_trimmed) {
                let flags = caps.get(1).map_or("", |m| m.as_str()).trim();
                let raw_content = caps.get(2).map_or("", |m| m.as_str()).trim();

                let mut description = None;

                // Check inline
                if let Some(idx) = line.find('#') {
                    let comment = line[idx + 1..].trim();
                    if !comment.is_empty() {
                        description = Some(comment.to_string());
                    }
                }

                // Check preceding line if no inline description found
                if description.is_none() && index > 0 {
                    let prev_line = lines[index - 1].trim();
                    if prev_line.starts_with('#') {
                        let comment = prev_line.trim_start_matches('#').trim();
                        if !comment.is_empty() {
                            description = Some(comment.to_string());
                        }
                    }
                }

                let resolved_content = resolve_variables(raw_content, variables, sorted_keys);
                let content_clean = resolved_content.split('#').next().unwrap_or("").trim();
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
                        mods: mods.clone(),
                        clean_mods: mods,
                        flags: flags.to_string(),
                        key,
                        dispatcher,
                        args,
                        description,
                        submap: current_submap.clone(),
                        line_number: index,
                        file_path: path.clone(),
                    });
                }
            } else if let Some(caps) = SOURCE_RE.captures(line_trimmed) {
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
    description: Option<String>,
) -> Result<()> {
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    let original_line = &lines[line_number];
    let re = Regex::new(r"^(\s*)bind([a-zA-Z]*)(\s*=\s*)([^#]*)").unwrap();

    if let Some(caps) = re.captures(original_line) {
        let indent = caps.get(1).map_or("", |m| m.as_str());
        let flags = caps.get(2).map_or("", |m| m.as_str());

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
    description: Option<String>,
) -> Result<usize> {
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = if content.is_empty() {
        vec![]
    } else {
        content.lines().map(|s| s.to_string()).collect()
    };

    let mut new_line = if args.trim().is_empty() {
        format!("bind = {}, {}, {}", mods, key, dispatcher)
    } else {
        format!("bind = {}, {}, {}, {}", mods, key, dispatcher, args)
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

#[cfg(test)]
mod tests;
