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
    result
}

fn get_xdg_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share"));
    }

    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for path in xdg_data_dirs.split(':') {
            if !path.is_empty() {
                dirs.push(PathBuf::from(path));
            }
        }
    } else {
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
                scan_dir(&path, apps);
            } else if let Some(ext) = path.extension() {
                if ext == "desktop" {
                    if let Some(app) = parse_desktop_file(&path) {
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

    if no_display {
        return None;
    }

    if let (Some(name), Some(exec)) = (name, exec) {
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
