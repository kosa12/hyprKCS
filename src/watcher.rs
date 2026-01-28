use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn create_config_watcher(sender: Sender<()>) -> Option<RecommendedWatcher> {
    let files = match crate::parser::get_loaded_files() {
        Ok(f) if !f.is_empty() => f,
        _ => crate::parser::get_config_path()
            .map(|p| vec![p])
            .unwrap_or_default(),
    };

    let mut dirs_to_watch: Vec<PathBuf> = files
        .iter()
        .filter_map(|p| {
            let parent = p.parent();
            match parent {
                Some(path) if path.as_os_str().is_empty() => std::env::current_dir().ok(),
                Some(path) => {
                    if path.is_absolute() {
                        Some(path.to_path_buf())
                    } else {
                        // Try to make absolute via CWD
                        std::env::current_dir().map(|cwd| cwd.join(path)).ok()
                    }
                }
                None => None,
            }
        })
        .map(|p| p.canonicalize().unwrap_or(p)) // Best effort canonicalization
        .collect();

    dirs_to_watch.sort();
    dirs_to_watch.dedup();

    let mut final_dirs: Vec<PathBuf> = Vec::new();
    for dir in dirs_to_watch {
        if !final_dirs.iter().any(|parent| dir.starts_with(parent)) {
            final_dirs.push(dir);
        }
    }

    let files_to_check = files;
    let sender = sender.clone();

    let mut watcher = match RecommendedWatcher::new(
        move |res: Result<notify::Event, _>| match res {
            Ok(event) => {
                let relevant = event
                    .paths
                    .iter()
                    .any(|p| files_to_check.iter().any(|f| f == p));

                if relevant {
                    let _ = sender.send(());
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create file watcher: {}", e);
            return None;
        }
    };

    for dir in final_dirs {
        if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
            eprintln!("Failed to start recursive watcher on {:?}: {}", dir, e);
        }
    }

    Some(watcher)
}
