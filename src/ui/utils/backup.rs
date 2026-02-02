use crate::config::constants;
use crate::config::StyleConfig;
use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

fn expand_tilde(path_str: &str) -> PathBuf {
    if let Some(stripped) = path_str.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path_str)
}

fn get_backup_root() -> Result<PathBuf> {
    // 1. Check CLI/Env override
    if let Ok(env_path) = std::env::var("HYPRKCS_BACKUP_PATH") {
        return Ok(expand_tilde(&env_path));
    }

    // 2. Check Config setting
    let config = StyleConfig::load();
    if let Some(alt_path) = config.alternative_backup_path {
        if !alt_path.trim().is_empty() {
            return Ok(expand_tilde(&alt_path));
        }
    }

    // 3. Default
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    Ok(config_dir
        .join(constants::HYPR_DIR)
        .join(constants::BACKUP_DIR))
}

pub fn perform_backup(force: bool) -> Result<String> {
    let config = StyleConfig::load();

    if !force && !config.auto_backup {
        return Ok("Auto-backup disabled".to_string());
    }

    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    let hypr_dir = config_dir.join(constants::HYPR_DIR);
    let backup_root = get_backup_root()?;

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let current_backup_dir = backup_root.join(&timestamp);

    fs::create_dir_all(&current_backup_dir)?;

    let mut count = 0;
    let mut errors = Vec::new();

    fn backup_recursive(
        current_dir: &Path,
        hypr_root: &Path,
        backup_root: &Path,
        count: &mut i32,
        errors: &mut Vec<String>,
    ) -> Result<()> {
        if !current_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip backup directory and hidden files/directories (like .git)
            if file_name == constants::BACKUP_DIR || file_name_str.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                // Recursively backup subdirectories
                backup_recursive(&path, hypr_root, backup_root, count, errors)?;
            } else {
                // Backup file
                if let Ok(rel_path) = path.strip_prefix(hypr_root) {
                    let dest = backup_root.join(rel_path);

                    if let Some(parent) = dest.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            errors
                                .push(format!("Failed to create parent dir for {:?}: {}", dest, e));
                            continue;
                        }
                    }

                    if let Err(e) = fs::copy(&path, &dest) {
                        errors.push(format!("Failed to backup {:?}: {}", path, e));
                    } else {
                        *count += 1;
                    }
                }
            }
        }
        Ok(())
    }

    if let Err(e) = backup_recursive(
        &hypr_dir,
        &hypr_dir,
        &current_backup_dir,
        &mut count,
        &mut errors,
    ) {
        eprintln!("Backup process encountered error: {}", e);
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("{}", err);
        }
    }

    if config.max_backups_enabled {
        if let Err(e) = prune_backups(&backup_root, config.max_backups_count as usize) {
            eprintln!("Failed to prune backups: {}", e);
        }
    }

    Ok(format!("Backed up {} files to {}", count, timestamp))
}

pub fn restore_backup(backup_path: &Path) -> Result<String> {
    if !backup_path.exists() || !backup_path.is_dir() {
        return Err(anyhow::anyhow!("Invalid backup path"));
    }

    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    let hypr_dir = config_dir.join(constants::HYPR_DIR);

    let mut restored_count = 0;
    let mut errors = Vec::new();

    // Helper to recursively walk and restore
    fn restore_recursive(
        current_dir: &Path,
        backup_root: &Path,
        target_root: &Path,
        count: &mut i32,
        errors: &mut Vec<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip hidden files/directories
            if file_name_str.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                restore_recursive(&path, backup_root, target_root, count, errors)?;
            } else if let Ok(rel_path) = path.strip_prefix(backup_root) {
                let dest = target_root.join(rel_path);

                if let Some(parent) = dest.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        errors.push(format!("Failed to create dir {:?}: {}", parent, e));
                        continue;
                    }
                }

                if let Err(e) = fs::copy(&path, &dest) {
                    errors.push(format!("Failed to copy {:?} to {:?}: {}", path, dest, e));
                } else {
                    *count += 1;
                }
            }
        }
        Ok(())
    }

    if let Err(e) = restore_recursive(
        backup_path,
        backup_path,
        &hypr_dir,
        &mut restored_count,
        &mut errors,
    ) {
        return Err(anyhow::anyhow!("Restore process encountered error: {}", e));
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("{}", err);
        }
        return Ok(format!(
            "Restored {} files with {} errors",
            restored_count,
            errors.len()
        ));
    }

    Ok(format!("Restored {} files successfully", restored_count))
}

use similar::{ChangeTag, TextDiff};

pub fn generate_diff(backup_path: &Path) -> Result<String> {
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    let hypr_dir = config_dir.join(constants::HYPR_DIR);

    let mut diff_output = String::new();
    let mut errors = Vec::new();

    fn diff_recursive(
        current_backup_dir: &Path,
        backup_root: &Path,
        hypr_root: &Path,
        output: &mut String,
        _errors: &mut Vec<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(current_backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                diff_recursive(&path, backup_root, hypr_root, output, _errors)?;
            } else if let Ok(rel_path) = path.strip_prefix(backup_root) {
                let current_file = hypr_root.join(rel_path);
                let backup_content = fs::read_to_string(&path).unwrap_or_default();
                let current_content = if current_file.exists() {
                    fs::read_to_string(&current_file).unwrap_or_default()
                } else {
                    String::new()
                };

                if backup_content != current_content {
                    output.push_str(&format!("--- {:?}\n", rel_path));
                    output.push_str(&format!("+++ {:?}\n", rel_path));

                    let diff = TextDiff::from_lines(&current_content, &backup_content);
                    for change in diff.iter_all_changes() {
                        let sign = match change.tag() {
                            ChangeTag::Delete => "-",
                            ChangeTag::Insert => "+",
                            ChangeTag::Equal => " ",
                        };
                        output.push_str(&format!("{}{}", sign, change));
                    }
                    output.push('\n');
                }
            }
        }
        Ok(())
    }

    diff_recursive(
        backup_path,
        backup_path,
        &hypr_dir,
        &mut diff_output,
        &mut errors,
    )?;

    if diff_output.is_empty() {
        return Ok("No differences found.".to_string());
    }

    Ok(diff_output)
}

pub fn list_backups() -> Result<Vec<PathBuf>> {
    let backup_root = get_backup_root()?;

    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(backup_root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    // Sort newest first (descending)
    entries.sort_by(|a, b| b.cmp(a));

    Ok(entries)
}

fn prune_backups(backup_root: &Path, max_count: usize) -> Result<()> {
    if !backup_root.exists() {
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(backup_root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    // Sort by name (which is timestamp, so lexicographical sort works for ISO-like dates)
    // Oldest first.
    entries.sort();

    if entries.len() > max_count {
        let to_remove = entries.len() - max_count;
        for path in entries.iter().take(to_remove) {
            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!("Failed to remove old backup {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}
