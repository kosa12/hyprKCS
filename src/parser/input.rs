use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone, Default)]
pub struct InputConfig {
    pub kb_layout: String,
    pub kb_variant: String,
    pub kb_options: String,
    pub follow_mouse: i32,
    pub sensitivity: f64,
    pub repeat_rate: i32,
    pub repeat_delay: i32,
}

pub fn load_input_config() -> Result<InputConfig> {
    let path = super::get_config_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    
    let mut config = InputConfig {
        kb_layout: String::new(),
        kb_variant: String::new(),
        kb_options: String::new(),
        follow_mouse: 1,
        sensitivity: 0.0,
        repeat_rate: 25,
        repeat_delay: 600,
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut inside_input = false;
    let mut input_depth = 0;

    let re_kv = Regex::new(r"^\s*([a-zA-Z0-9_]+)\s*=\s*(.*)").unwrap();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("input {") || (trimmed.starts_with("input") && trimmed.ends_with("{")) {
            inside_input = true;
            input_depth = 1;
            continue;
        }

        if inside_input {
            if trimmed == "}" {
                input_depth -= 1;
                if input_depth == 0 {
                    break;
                }
            } else if trimmed.ends_with("{") {
                input_depth += 1;
            }

            if let Some(caps) = re_kv.captures(trimmed) {
                let key = caps.get(1).unwrap().as_str().trim();
                let val = caps.get(2).unwrap().as_str().split('#').next().unwrap_or("").trim();

                match key {
                    "kb_layout" => config.kb_layout = val.to_string(),
                    "kb_variant" => config.kb_variant = val.to_string(),
                    "kb_options" => config.kb_options = val.to_string(),
                    "follow_mouse" => if let Ok(n) = val.parse() { config.follow_mouse = n; },
                    "sensitivity" => if let Ok(n) = val.parse() { config.sensitivity = n; },
                    "repeat_rate" => if let Ok(n) = val.parse() { config.repeat_rate = n; },
                    "repeat_delay" => if let Ok(n) = val.parse() { config.repeat_delay = n; },
                    _ => {}
                }
            }
        }
    }

    Ok(config)
}

pub fn save_input_config(config: &InputConfig) -> Result<()> {
    let path = super::get_config_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    
    let mut inside_input = false;
    let mut input_start_idx = None;
    let mut input_end_idx = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("input {") || (trimmed.starts_with("input") && trimmed.ends_with("{")) {
            if !inside_input {
                inside_input = true;
                input_start_idx = Some(i);
            }
        }
        if inside_input && trimmed == "}" {
            input_end_idx = Some(i);
            break; 
        }
    }

    let updates = [
        ("kb_layout", &config.kb_layout),
        ("kb_variant", &config.kb_variant),
        ("kb_options", &config.kb_options),
        ("follow_mouse", &config.follow_mouse.to_string()),
        ("sensitivity", &config.sensitivity.to_string()),
        ("repeat_rate", &config.repeat_rate.to_string()),
        ("repeat_delay", &config.repeat_delay.to_string()),
    ];

    if let (Some(start), Some(end)) = (input_start_idx, input_end_idx) {
        let mut updated_keys = std::collections::HashSet::new();
        let re_kv = Regex::new(r"^(\s*)([a-zA-Z0-9_]+)(\s*=\s*)(.*)").unwrap();

        let mut changes = Vec::new();

        for i in start + 1..end {
            let line = &lines[i];
            if let Some(caps) = re_kv.captures(line) {
                let indent = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let key = caps.get(2).unwrap().as_str().trim().to_string();
                let sep = caps.get(3).unwrap().as_str().to_string(); // " = "
                
                let comment = if let Some(idx) = line.find('#') {
                    line[idx..].to_string()
                } else {
                    String::new()
                };

                for (u_key, u_val) in &updates {
                    if key == *u_key {
                        let new_line = format!("{}{}{}{}{}", indent, key, sep, u_val, if comment.is_empty() { "" } else { " " });
                        changes.push((i, format!("{}{}", new_line, comment)));
                        updated_keys.insert(key.clone());
                    }
                }
            }
        }

        for (idx, new_content) in changes {
            lines[idx] = new_content;
        }

        let mut insert_pos = end;
        for (key, val) in &updates {
            if !updated_keys.contains(*key) && !val.is_empty() {
                if (key == &"kb_variant" || key == &"kb_options") && val.is_empty() {
                    continue;
                }
                
                lines.insert(insert_pos, format!("    {} = {}", key, val));
                insert_pos += 1;
            }
        }

    } else {
        lines.push(String::new());
        lines.push("input {".to_string());
        for (key, val) in &updates {
             if (key == &"kb_variant" || key == &"kb_options") && val.is_empty() {
                continue;
            }
            lines.push(format!("    {} = {}", key, val));
        }
        lines.push("}".to_string());
    }

    std::fs::write(&path, lines.join("\n"))?;
    Ok(())
}
