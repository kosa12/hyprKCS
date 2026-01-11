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

pub fn refresh_conflicts(model: &gio::ListStore) {
    let n = model.n_items();
    
    for i in 0..n {
        let obj1 = model.item(i).and_downcast::<KeybindObject>().unwrap();
        let mods1_str = obj1.property::<String>("clean-mods");
        let key1_str = obj1.property::<String>("key");
        let submap1 = obj1.property::<String>("submap");
        let (mods1, key1) = normalize(&mods1_str, &key1_str);
        
        let mut reason = String::new();
        let mut is_conflicted = false;

        for j in 0..n {
            if i == j { continue; }
            let obj2 = model.item(j).and_downcast::<KeybindObject>().unwrap();
            let mods2_str = obj2.property::<String>("clean-mods");
            let key2_str = obj2.property::<String>("key");
            let submap2 = obj2.property::<String>("submap");
            let (mods2, key2) = normalize(&mods2_str, &key2_str);
            
            if mods1 == mods2 && key1 == key2 && submap1 == submap2 {
                 is_conflicted = true;
                 reason = format!("Conflicts with: {} {}", mods2_str, key2_str);
                 break;
            }
        }
        
        obj1.set_property("is-conflicted", is_conflicted);
        obj1.set_property("conflict-reason", reason);
    }
}

pub fn reload_keybinds(model: &gio::ListStore) {
    model.remove_all();
    
    let keybinds = crate::parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    for kb in keybinds {
        model.append(&KeybindObject::new(kb, None));
    }
    
    refresh_conflicts(model);
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
