use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;

pub fn start_config_watcher(sender: Sender<()>) {
    thread::spawn(move || {
        let config_path_res = crate::parser::get_config_path();
        if let Err(e) = config_path_res {
            eprintln!("Failed to get config path for watcher: {}", e);
            return;
        }
        let config_file = config_path_res.unwrap();
        let watch_path = config_file.parent().unwrap_or(Path::new(".")).to_path_buf();

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
            eprintln!(
                "Failed to start recursive watcher on {:?}: {}",
                watch_path, e
            );
            return;
        }

        for res in rx {
            match res {
                Ok(event) => {
                    let relevant = event
                        .paths
                        .iter()
                        .any(|p| p.extension().map_or(false, |ext| ext == "conf"));

                    if relevant {
                        let _ = sender.send(());
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }
    });
}
