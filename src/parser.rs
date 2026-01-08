use anyhow::{Context, Result};
use dirs::config_dir;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: String,       // Display string (e.g. "[l] SUPER")
    pub clean_mods: String, // Raw mods (e.g. "SUPER")
    pub flags: String,      // Flags (e.g. "l")
    pub key: String,
    pub dispatcher: String,
    pub args: String,
    pub line_number: usize,
}

pub fn get_config_path() -> Result<PathBuf> {
    let mut path = config_dir().context("Could not find config directory")?;
    path.push("hypr");
    path.push("hyprland.conf");
    Ok(path)
}

pub fn parse_config() -> Result<Vec<Keybind>> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let file = File::open(&path).context(format!("Failed to open config file: {:?}", path))?;
    let reader = BufReader::new(file);
    let mut keybinds = Vec::new();

    // Regex to match: bind[FLAGS] = MODS, KEY, DISPATCHER, ARGS
    // Group 1: Flags (e, l, r, m, etc.)
    // Group 2: Mods (can be empty)
    // Group 3: Key
    // Group 4: Dispatcher
    // Group 5: Args (optional)
    // FUCK REGEXXXXXXXXXXXXXXXXXXX
    let re = Regex::new(r"^\s*bind([a-z]*)\s*=\s*([^,]*)\s*,\s*([^,]+)\s*,\s*([^,]+)(?:\s*,\s*(.*))?").unwrap();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let line_trimmed = line.trim();
        
        if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
            continue;
        }

        if let Some(caps) = re.captures(line_trimmed) {
            let flags = caps.get(1).map_or("", |m| m.as_str()).trim();
            let mods = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
            let key = caps.get(3).map_or("", |m| m.as_str()).trim().to_string();
            let dispatcher = caps.get(4).map_or("", |m| m.as_str()).trim().to_string();
            let args = caps.get(5).map_or("", |m| m.as_str()).trim().to_string();

            let display_mods = if flags.is_empty() {
                mods.clone()
            } else {
                format!("[{}] {}", flags, mods)
            };

            let args = if let Some(idx) = args.find('#') {
                args[..idx].trim().to_string()
            } else {
                args
            };

            keybinds.push(Keybind {
                mods: display_mods,
                clean_mods: mods,
                flags: flags.to_string(),
                key,
                dispatcher,
                args,
                line_number: index,
            });
        }
    }

    Ok(keybinds)
}

pub fn update_line(line_number: usize, new_mods: &str, new_key: &str, new_dispatcher: &str, new_args: &str) -> Result<()> {
    let path = get_config_path()?;
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    let original_line = &lines[line_number];
    
    // Regex to break down the line:
    // Group 1: Prefix (indent + "bind")
    // Group 2: Flags + " = " (e.g. "e = ")
    // Group 3: Old Mods (ignored)
    // Group 4: Old Key (ignored)
    // Group 5: Old Dispatcher (ignored)
    // Group 6: Old Args (ignored, but we check if it existed)
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
         
         std::fs::write(path, lines.join("\n"))?;
         Ok(())
    } else {
         Err(anyhow::anyhow!("Could not parse original line structure"))
    }
}

pub fn add_keybind(mods: &str, key: &str, dispatcher: &str, args: &str) -> Result<usize> {
    let path = get_config_path()?;
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let new_line = if args.trim().is_empty() {
        format!("bind = {}, {}, {}", mods, key, dispatcher)
    } else {
        format!("bind = {}, {}, {}, {}", mods, key, dispatcher, args)
    };

    lines.push(new_line);
    std::fs::write(&path, lines.join("\n"))?;
    
    Ok(lines.len() - 1)
}

pub fn delete_keybind(line_number: usize) -> Result<()> {
    let path = get_config_path()?;
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if line_number >= lines.len() {
        return Err(anyhow::anyhow!("Line number out of bounds"));
    }

    lines.remove(line_number);
    std::fs::write(&path, lines.join("\n"))?;
    
    Ok(())
}

