use gtk4 as gtk;
use gtk::{gio, glib, prelude::*};
use crate::keybind_object::KeybindObject;

fn normalize(mods: &str, key: &str) -> (std::collections::BTreeSet<String>, String) {
    let mod_set: std::collections::BTreeSet<String> = mods.split_whitespace()
        .map(|s| s.to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
        
    let clean_key = key.trim().to_lowercase();
    
    (mod_set, clean_key)
}

fn detect_conflicts(keybinds: &[crate::parser::Keybind]) -> Vec<Option<String>> {
    let mut collision_map: std::collections::HashMap<(Vec<String>, String, String), Vec<usize>> = std::collections::HashMap::new();

    for (i, kb) in keybinds.iter().enumerate() {
        let (mod_set, clean_key) = normalize(&kb.clean_mods, &kb.key);
        // Convert BTreeSet to sorted Vec for hashing/key
        let sorted_mods: Vec<String> = mod_set.into_iter().collect();
        let submap = kb.submap.clone().unwrap_or_default();
        
        let key = (sorted_mods, clean_key, submap);
        collision_map.entry(key).or_default().push(i);
    }

    let mut results = vec![None; keybinds.len()];

    for (key, indices) in collision_map {
        if indices.len() > 1 {
            let (mods, key_char, _) = key;
            let mods_disp = mods.join(" ");
            let reason = format!("Conflicts with: {} {}", mods_disp, key_char);
            
            for idx in indices {
                results[idx] = Some(reason.clone());
            }
        }
    }
    
    results
}

pub fn reload_keybinds(model: &gio::ListStore) {
    model.remove_all();
    
    let keybinds = crate::parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    let conflicts = detect_conflicts(&keybinds);

    for (kb, conflict) in keybinds.into_iter().zip(conflicts.into_iter()) {
        model.append(&KeybindObject::new(kb, conflict));
    }
}

pub fn execute_keybind(dispatcher: &str, args: &str) {
    let variables = crate::parser::get_variables().unwrap_or_default();
    
    let mut resolved_dispatcher = dispatcher.to_string();
    let mut resolved_args = args.to_string();
    
    let mut sorted_vars: Vec<_> = variables.keys().collect();
    sorted_vars.sort_by(|a, b| b.len().cmp(&a.len()));

    for key in sorted_vars {
        if resolved_dispatcher.contains(key) {
            resolved_dispatcher = resolved_dispatcher.replace(key, &variables[key]);
        }
        if resolved_args.contains(key) {
            resolved_args = resolved_args.replace(key, &variables[key]);
        }
    }

    let mut command = std::process::Command::new("hyprctl");
    command.arg("dispatch").arg(&resolved_dispatcher);
    
    if !resolved_args.trim().is_empty() {
        command.arg(&resolved_args);
    }
    
    let _ = command.spawn();
}

pub fn execute_hyprctl(args: &[&str]) {
    use std::io::Write;
    
    let output = std::process::Command::new("hyprctl")
        .args(args)
        .output();

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hyprkcs-debug.log") 
    {
        let _ = writeln!(file, "Executing: hyprctl {:?}", args);
        match &output {
            Ok(out) => {
                let _ = writeln!(file, "Status: {}", out.status);
                let _ = writeln!(file, "Stdout: {}", String::from_utf8_lossy(&out.stdout));
                let _ = writeln!(file, "Stderr: {}", String::from_utf8_lossy(&out.stderr));
            },
            Err(e) => {
                let _ = writeln!(file, "Failed to execute: {}", e);
            }
        }
    }

    if let Err(e) = output {
        eprintln!("Failed to execute hyprctl: {}", e);
    }
}

#[allow(deprecated)]
pub fn setup_dispatcher_completion(entry: &gtk::Entry) {
    let dispatchers = [
        "exec", "execr", "pass", "killactive", "closewindow", "workspace",
        "movetoworkspace", "movetoworkspacesilent", "togglefloating",
        "fullscreen", "fakefullscreen", "dpms", "pin", "movefocus",
        "movewindow", "centerwindow", "resizeactive", "moveactive",
        "cyclenext", "swapnext", "focuswindow", "focusmonitor",
        "splitratio", "toggleopaque", "movecursortocorner", "workspaceopt",
        "exit", "forcerendererreload", "movecurrentworkspacetomonitor",
        "focusworkspaceoncurrentmonitor", "togglespecialworkspace",
        "focusurgentorlast", "togglegroup", "changegroupactive",
        "swapprev", "focuscurrentorlast", "lockgroups", "lockactivegroup",
        "moveintogroup", "moveoutofgroup", "movewindoworgroup",
        "movegroupwindow", "denywindowfromgroup", "setignoregrouplock",
        "alterzorder", "tag", "layoutmsg", "sendshortcut", "sendkeystate",
    ];

    let list_store = gtk::ListStore::new(&[glib::Type::STRING]);
    for dispatcher in dispatchers {
        list_store.set(&list_store.append(), &[(0, &dispatcher)]);
    }

    let completion = gtk::EntryCompletion::builder()
        .model(&list_store)
        .text_column(0)
        .inline_completion(true)
        .popup_completion(false)
        .build();

    entry.set_completion(Some(&completion));
}
