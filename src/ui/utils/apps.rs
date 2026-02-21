use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
}

pub fn get_installed_apps() -> Vec<AppInfo> {
    let mut apps = HashMap::new();
    let data_dirs = get_xdg_data_dirs();

    for dir in data_dirs {
        let applications_dir = dir.join("applications");
        if applications_dir.exists() {
            scan_dir(&applications_dir, &mut apps);
        }
    }

    let mut result: Vec<AppInfo> = apps.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    println!("[hyprKCS] Found {} installed applications for autocomplete.", result.len());
    result
}

fn get_xdg_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User local directory
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share"));
    }

    // System directories from XDG_DATA_DIRS
    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for path in xdg_data_dirs.split(':') {
            if !path.is_empty() {
                dirs.push(PathBuf::from(path));
            }
        }
    } else {
        // Fallback defaults
        dirs.push(PathBuf::from("/usr/local/share"));
        dirs.push(PathBuf::from("/usr/share"));
    }

    dirs
}

fn scan_dir(dir: &Path, apps: &mut HashMap<String, AppInfo>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively scan subdirectories (e.g. kde4, etc.)
                scan_dir(&path, apps);
            } else if let Some(ext) = path.extension() {
                if ext == "desktop" {
                    if let Some(app) = parse_desktop_file(&path) {
                        // Use filename as unique key to avoid duplicates from different directories
                        // (User local overrides system)
                        if let Some(file_stem) = path.file_stem() {
                            let key = file_stem.to_string_lossy().to_string();
                            apps.entry(key).or_insert(app);
                        }
                    }
                }
            }
        }
    }
}

fn parse_desktop_file(path: &Path) -> Option<AppInfo> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut is_desktop_entry = false;
    let mut no_display = false;
    let mut is_application = true; // Default to true if Type is missing, though spec says it's required.

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            is_desktop_entry = true;
            continue;
        }

        if !is_desktop_entry {
            continue;
        }

        if line.starts_with('[') && line != "[Desktop Entry]" {
            break;
        }
        
        // Handle Type=Application
        if line.starts_with("Type=") {
             let type_val = line.trim_start_matches("Type=");
             if type_val != "Application" {
                 is_application = false;
             }
        }

        if line.starts_with("Name=") && name.is_none() {
            name = Some(line.trim_start_matches("Name=").to_string());
        } else if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(line.trim_start_matches("Exec=").to_string());
        } else if line.starts_with("Icon=") && icon.is_none() {
            icon = Some(line.trim_start_matches("Icon=").to_string());
        } else if line == "NoDisplay=true" {
            no_display = true;
        }
    }

    if no_display || !is_application {
        return None;
    }

    if let (Some(name), Some(exec)) = (name, exec) {
        // Clean up Exec command (remove field codes like %u, %F, etc.)
        let clean_exec = clean_exec_cmd(&exec);
        Some(AppInfo {
            name,
            exec: clean_exec,
            icon,
        })
    } else {
        None
    }
}

fn clean_exec_cmd(cmd: &str) -> String {
    // Remove field codes: %f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m
    // Also usually quoted like "firefox %u" -> "firefox"
    // Better strategy: Take the first part of the command if it's not a variable assignment?
    // No, commands can be "foo --bar".
    // We just want to remove the % placeholders.
    
    // Simple approach: split by space, keep parts that don't look like %X
    // But we need to handle quotes.
    // If we have "google-chrome-stable %U", splitting by space gives ["\"google-chrome-stable", "%U\""] if quoted?
    // Actually desktop files usually don't quote the %U.
    // Exec=/usr/bin/google-chrome-stable %U
    
    // Let's just remove any token starting with % and length 2.
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut clean_parts = Vec::new();

    for part in parts {
        if part.starts_with('%') && part.len() == 2 {
            continue;
        }
        clean_parts.push(part);
    }

    clean_parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanning() {
        let apps = get_installed_apps();
        println!("Found {} apps", apps.len());
        for app in apps.iter().take(10) {
            println!(" - {} -> {}", app.name, app.exec);
        }
        assert!(apps.len() > 0);
    }
}
