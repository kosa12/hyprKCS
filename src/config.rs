use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub font_size: Option<String>,
    pub border_size: Option<String>,
    pub border_radius: Option<String>,
    pub opacity: Option<f64>,
    pub width: i32,
    pub height: i32,
    pub show_submaps: bool,
    pub show_args: bool,
    pub monitor_margin: i32,
    pub row_padding: i32,
    
    // Internal use for UI feedback
    pub errors: Vec<String>,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            font_size: None,
            border_size: None,
            border_radius: None,
            opacity: None,
            width: 700,
            height: 500,
            show_submaps: true,
            show_args: true,
            monitor_margin: 12,
            row_padding: 2,
            errors: Vec::new(),
        }
    }
}

impl StyleConfig {
    pub fn load() -> Self {
        let mut config = StyleConfig::default();

        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("hyprkcs/hyprkcs.conf");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path) {
                    let vars = parse_ini_like(&content);
                    
                    if let Some(val) = vars.get("fontSize") {
                        config.font_size = Some(val.clone());
                    }
                    if let Some(val) = vars.get("borderSize") {
                        config.border_size = Some(val.clone());
                    }
                    if let Some(val) = vars.get("borderRadius") {
                        config.border_radius = Some(val.clone());
                    }
                    if let Some(val) = vars.get("opacity") {
                        match val.parse::<f64>() {
                            Ok(num) => {
                                if num < 0.0 || num > 1.0 {
                                    config.errors.push(format!("Opacity '{}' out of range (0.0 - 1.0). Using default.", val));
                                } else {
                                    config.opacity = Some(num);
                                }
                            }
                            Err(_) => config.errors.push(format!("Invalid opacity value '{}'. Using default.", val)),
                        }
                    }
                    if let Some(val) = vars.get("width") {
                        if let Some(num) = parse_pixels(val) {
                            if num < 100 {
                                config.errors.push(format!("Width '{}' is too small (min 100px). Using default.", val));
                            } else {
                                config.width = num;
                            }
                        } else {
                            config.errors.push(format!("Invalid width value '{}'.", val));
                        }
                    }
                    if let Some(val) = vars.get("height") {
                        if let Some(num) = parse_pixels(val) {
                            if num < 100 {
                                config.errors.push(format!("Height '{}' is too small (min 100px). Using default.", val));
                            } else {
                                config.height = num;
                            }
                        } else {
                            config.errors.push(format!("Invalid height value '{}'.", val));
                        }
                    }
                    if let Some(val) = vars.get("showSubmaps") {
                        config.show_submaps = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("showArgs") {
                        config.show_args = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("monitorMargin") {
                        if let Some(num) = parse_pixels(val) {
                             if num < 0 {
                                config.errors.push(format!("Monitor margin '{}' cannot be negative. Using default.", val));
                            } else {
                                config.monitor_margin = num;
                            }
                        } else {
                            config.errors.push(format!("Invalid monitorMargin '{}'.", val));
                        }
                    }
                     if let Some(val) = vars.get("rowPadding") {
                        if let Some(num) = parse_pixels(val) {
                             if num < 0 {
                                config.errors.push(format!("Row padding '{}' cannot be negative. Using default.", val));
                            } else {
                                config.row_padding = num;
                            }
                        } else {
                            config.errors.push(format!("Invalid rowPadding '{}'.", val));
                        }
                    }
                }
            }
        }
        config
    }
}

fn parse_pixels(val: &str) -> Option<i32> {
    let val = val.to_lowercase();
    let clean = val.trim_end_matches("px").trim();
    clean.parse::<i32>().ok()
}

fn parse_ini_like(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            map.insert(key, value);
        }
    }
    map
}
