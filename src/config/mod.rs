pub mod favorites;
pub mod constants;

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
    pub show_favorites: bool,
    pub alternating_row_colors: bool,
    pub default_sort: String,
    pub shadow_size: String,
    pub monitor_margin: i32,
    pub row_padding: i32,
    
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
            show_submaps: false,
            show_args: true,
            show_favorites: true,
            alternating_row_colors: true,
            default_sort: "key".to_string(),
            shadow_size: "0 4px 24px rgba(0,0,0,0.4)".to_string(),
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
            let config_path = config_dir.join(constants::HYPRKCS_DIR).join(constants::HYPRKCS_CONF);
            
            if !config_path.exists() {
                // Create default config
                if let Some(parent) = config_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                
                let default_content = r#"# Window dimensions
width = 700px
height = 500px

# Appearance
fontSize = 0.9rem
borderSize = 1px
borderRadius = 12px
opacity = 1.0

# UI Elements
showSubmaps = false
showArgs = true
showFavorites = true
alternatingRowColors = true
defaultSort = key
shadowSize = 0 4px 24px rgba(0,0,0,0.4)

# Spacing
monitorMargin = 12px
rowPadding = 2px
"#;
                if let Err(e) = fs::write(&config_path, default_content) {
                    eprintln!("Failed to write default config: {}", e);
                }
            }

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
                            }
                            else {
                                config.width = num;
                            }
                        }
                        else {
                            config.errors.push(format!("Invalid width value '{}'.", val));
                        }
                    }
                    if let Some(val) = vars.get("height") {
                        if let Some(num) = parse_pixels(val) {
                            if num < 100 {
                                config.errors.push(format!("Height '{}' is too small (min 100px). Using default.", val));
                            }
                            else {
                                config.height = num;
                            }
                        }
                        else {
                            config.errors.push(format!("Invalid height value '{}'.", val));
                        }
                    }
                    if let Some(val) = vars.get("showSubmaps") {
                        config.show_submaps = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("showArgs") {
                        config.show_args = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("showFavorites") {
                        config.show_favorites = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("alternatingRowColors") {
                        config.alternating_row_colors = val.to_lowercase() == "true";
                    }
                    if let Some(val) = vars.get("defaultSort") {
                        config.default_sort = val.to_lowercase();
                    }
                    if let Some(val) = vars.get("shadowSize") {
                        config.shadow_size = val.clone();
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
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join(constants::HYPRKCS_DIR).join(constants::HYPRKCS_CONF);
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let content = format!(
                r#"# Window dimensions
width = {}px
height = {}px

# Appearance
fontSize = {}
borderSize = {}
borderRadius = {}
opacity = {}

# UI Elements
showSubmaps = {}
showArgs = {}
showFavorites = {}
alternatingRowColors = {}
defaultSort = {}
shadowSize = {}

# Spacing
monitorMargin = {}px
rowPadding = {}px
"#,
                self.width,
                self.height,
                self.font_size.as_deref().unwrap_or("0.9rem"),
                self.border_size.as_deref().unwrap_or("1px"),
                self.border_radius.as_deref().unwrap_or("12px"),
                self.opacity.unwrap_or(1.0),
                self.show_submaps,
                self.show_args,
                self.show_favorites,
                self.alternating_row_colors,
                self.default_sort,
                self.shadow_size,
                self.monitor_margin,
                self.row_padding
            );

            fs::write(config_path, content)?;
        }
        Ok(())
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