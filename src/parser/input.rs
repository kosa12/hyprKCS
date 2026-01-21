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

#[derive(Debug, Clone)]
pub struct GesturesConfig {
    pub workspace_swipe: bool,
    pub workspace_swipe_fingers: i32,
}

impl Default for GesturesConfig {
    fn default() -> Self {
        Self {
            workspace_swipe: false,
            workspace_swipe_fingers: 3,
        }
    }
}

pub fn load_input_config() -> Result<(InputConfig, GesturesConfig)> {
    let path = super::get_config_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let mut input_config = InputConfig {
        kb_layout: String::new(),
        kb_variant: String::new(),
        kb_options: String::new(),
        follow_mouse: 1,
        sensitivity: 0.0,
        repeat_rate: 25,
        repeat_delay: 600,
    };

    let mut gestures_config = GesturesConfig::default();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_block = "";
    let mut block_depth = 0;

    let re_kv = Regex::new(r"^\s*([a-zA-Z0-9_]+)\s*=\s*(.*)").unwrap();
    let re_gesture = Regex::new(r"^\s*gesture\s*=\s*(\d+)\s*,\s*horizontal\s*,\s*workspace").unwrap();

    for line in lines {
        let trimmed = line.trim();
        
        // Detect block start
        if trimmed.starts_with("input {") || (trimmed.starts_with("input") && trimmed.ends_with("{")) {
            current_block = "input";
            block_depth = 1;
            continue;
        }

        // Global scope check for gesture
        if block_depth == 0 {
            if let Some(caps) = re_gesture.captures(trimmed) {
                gestures_config.workspace_swipe = true;
                if let Ok(n) = caps.get(1).unwrap().as_str().parse() {
                    gestures_config.workspace_swipe_fingers = n;
                }
            }
        }

        if block_depth > 0 {
            if trimmed == "}" {
                block_depth -= 1;
                if block_depth == 0 {
                    current_block = "";
                }
                continue;
            } else if trimmed.ends_with("{") {
                block_depth += 1;
            }

            if let Some(caps) = re_kv.captures(trimmed) {
                let key = caps.get(1).unwrap().as_str().trim();
                let val = caps
                    .get(2)
                    .unwrap()
                    .as_str()
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim();

                if current_block == "input" {
                    match key {
                        "kb_layout" => input_config.kb_layout = val.to_string(),
                        "kb_variant" => input_config.kb_variant = val.to_string(),
                        "kb_options" => input_config.kb_options = val.to_string(),
                        "follow_mouse" => {
                            if let Ok(n) = val.parse() {
                                input_config.follow_mouse = n;
                            }
                        }
                        "sensitivity" => {
                            if let Ok(n) = val.parse() {
                                input_config.sensitivity = n;
                            }
                        }
                        "repeat_rate" => {
                            if let Ok(n) = val.parse() {
                                input_config.repeat_rate = n;
                            }
                        }
                        "repeat_delay" => {
                            if let Ok(n) = val.parse() {
                                input_config.repeat_delay = n;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok((input_config, gestures_config))
}

pub fn save_input_config(input_config: &InputConfig, gestures_config: &GesturesConfig) -> Result<()> {
    let path = super::get_config_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let input_updates = [
        ("kb_layout", input_config.kb_layout.as_str()),
        ("kb_variant", input_config.kb_variant.as_str()),
        ("kb_options", input_config.kb_options.as_str()),
        ("follow_mouse", &input_config.follow_mouse.to_string()),
        ("sensitivity", &input_config.sensitivity.to_string()),
        ("repeat_rate", &input_config.repeat_rate.to_string()),
        ("repeat_delay", &input_config.repeat_delay.to_string()),
    ];

    let legacy_gesture_keys = [
        "workspace_swipe", "workspace_swipe_fingers", "workspace_swipe_distance",
        "workspace_swipe_invert", "workspace_swipe_min_speed_to_force",
        "workspace_swipe_cancel_ratio", "workspace_swipe_create_new",
        "workspace_swipe_direction_lock", "workspace_swipe_direction_lock_threshold",
        "workspace_swipe_forever"
    ];

    // 1. Identify lines to remove (Gestures block and legacy keys in input)
    let mut to_remove = std::collections::HashSet::new();
    {
        let mut depth = 0;
        let mut current_top_block: Option<String> = None;
        let re_kv = Regex::new(r"^\s*([a-zA-Z0-9_]+)\s*=\s*(.*)").unwrap();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            if trimmed.ends_with('{') {
                if depth == 0 {
                    let name = trimmed.trim_end_matches('{').trim();
                    current_top_block = Some(name.to_string());
                }
                depth += 1;
                if let Some(ref name) = current_top_block {
                    if name == "gestures" {
                        to_remove.insert(i);
                    }
                }
                continue;
            }

            if trimmed.starts_with('}') {
                if depth > 0 {
                    if let Some(ref name) = current_top_block {
                        if name == "gestures" {
                            to_remove.insert(i);
                        }
                    }
                    depth -= 1;
                    if depth == 0 {
                        current_top_block = None;
                    }
                }
                continue;
            }

            if depth > 0 {
                if let Some(ref name) = current_top_block {
                    if name == "gestures" {
                        to_remove.insert(i);
                    } else if name == "input" {
                        if let Some(caps) = re_kv.captures(trimmed) {
                            let key = caps.get(1).unwrap().as_str().trim();
                            if legacy_gesture_keys.contains(&key) {
                                to_remove.insert(i);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Perform Removal
    let mut cleaned_lines = Vec::new();
    for (i, line) in lines.into_iter().enumerate() {
        if !to_remove.contains(&i) {
            cleaned_lines.push(line);
        }
    }
    let mut lines = cleaned_lines;

    // 3. Update Input Block
    {
        let block_name = "input";
        let mut inside_block = false;
        let mut start_idx = None;
        let mut end_idx = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("{} {{", block_name))
                || (trimmed.starts_with(block_name) && trimmed.ends_with("{"))
            {
                if !inside_block {
                    inside_block = true;
                    start_idx = Some(i);
                }
            }
            if inside_block && trimmed == "}" {
                end_idx = Some(i);
                break;
            }
        }

        if let (Some(start), Some(end)) = (start_idx, end_idx) {
            let mut updated_keys = std::collections::HashSet::new();
            let re_kv = Regex::new(r"^(\s*)([a-zA-Z0-9_]+)(\s*=\s*)(.*)").unwrap();
            let mut changes = Vec::new();

            for i in start + 1..end {
                let line = &lines[i];
                if let Some(caps) = re_kv.captures(line) {
                    let indent = caps.get(1).map_or("", |m| m.as_str()).to_string();
                    let key = caps.get(2).unwrap().as_str().trim().to_string();
                    let sep = caps.get(3).unwrap().as_str().to_string();

                    let comment = if let Some(idx) = line.find('#') {
                        line[idx..].to_string()
                    } else {
                        String::new()
                    };

                    for (u_key, u_val) in &input_updates {
                        if key == *u_key {
                            let new_line = format!(
                                "{}{}{}{}{}",
                                indent,
                                key,
                                sep,
                                u_val,
                                if comment.is_empty() { "" } else { " " }
                            );
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
            for (key, val) in &input_updates {
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
            lines.push(format!("{} {{", block_name));
            for (key, val) in &input_updates {
                if (key == &"kb_variant" || key == &"kb_options") && val.is_empty() {
                    continue;
                }
                lines.push(format!("    {} = {}", key, val));
            }
            lines.push("}".to_string());
        }
    }

    // 4. Update Global 'gesture' Line
    {
        let re_gesture = Regex::new(r"^\s*gesture\s*=\s*(\d+)\s*,\s*horizontal\s*,\s*workspace").unwrap();
        let mut gesture_line_idx = None;

        for (i, line) in lines.iter().enumerate() {
            if re_gesture.is_match(line) {
                gesture_line_idx = Some(i);
                break;
            }
        }

        if gestures_config.workspace_swipe {
            let new_line = format!("gesture = {}, horizontal, workspace", gestures_config.workspace_swipe_fingers);
            if let Some(idx) = gesture_line_idx {
                lines[idx] = new_line;
            } else {
                lines.push(new_line);
            }
        } else {
            if let Some(idx) = gesture_line_idx {
                lines.remove(idx);
            }
        }
    }

    std::fs::write(&path, lines.join("\n"))?;
    Ok(())
}
