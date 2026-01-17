use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use crate::config::constants;
use crate::config::StyleConfig;
use crate::parser;

pub fn perform_backup(force: bool) -> Result<String> {
    let config = StyleConfig::load();
    
    if !force && !config.auto_backup {
        return Ok("Auto-backup disabled".to_string());
    }

    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    let backup_root = config_dir.join(constants::HYPR_DIR).join(constants::BACKUP_DIR);
    
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let backup_dir = backup_root.join(&timestamp);

    fs::create_dir_all(&backup_dir)?;

    let files = parser::get_all_config_files()?;
    let mut count = 0;

    for file_path in files {
        if let Some(name) = file_path.file_name() {
            let dest = backup_dir.join(name);
            if let Err(e) = fs::copy(&file_path, &dest) {
                eprintln!("Failed to backup {:?}: {}", file_path, e);
            } else {
                count += 1;
            }
        }
    }

    if config.max_backups_enabled {
        if let Err(e) = prune_backups(&backup_root, config.max_backups_count as usize) {
            eprintln!("Failed to prune backups: {}", e);
        }
    }

    Ok(format!("Backed up {} files to {}", count, timestamp))
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
