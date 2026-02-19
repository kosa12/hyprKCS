use crate::config::favorites::{is_favorite, load_favorites};
use crate::keybind_object::KeybindObject;
use crate::ui::utils::execution::command_exists;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple interner to share Arc<str> pointers across all keybind objects
struct StringPool(HashSet<Arc<str>>);

impl StringPool {
    fn new() -> Self {
        Self(HashSet::with_capacity(512))
    }

    fn intern(&mut self, s: Arc<str>) -> Arc<str> {
        if let Some(existing) = self.0.get(&s) {
            existing.clone()
        } else {
            self.0.insert(s.clone());
            s
        }
    }
}

pub fn normalize(mods: &str, key: &str) -> (String, String) {
    let mut mods_list: Vec<String> = mods
        .split(|c: char| c == '+' || c.is_whitespace())
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .map(|s| match s.as_str() {
            "MOD4" | "SUPER" | "LOGO" | "WIN" => "SUPER".to_string(),
            "MOD1" | "ALT" => "ALT".to_string(),
            "CONTROL" | "CTRL" => "CTRL".to_string(),
            "SHIFT" => "SHIFT".to_string(),
            _ => s,
        })
        .collect();

    mods_list.sort_unstable();
    mods_list.dedup();

    let mut clean_key = key.trim().to_lowercase();

    // Map common aliases to canonical keysyms used by xkbcommon/Hyprland
    clean_key = match clean_key.as_str() {
        " " => "space".to_string(),
        "\\" => "backslash".to_string(),
        "|" => "bar".to_string(),
        "[" => "bracketleft".to_string(),
        "]" => "bracketright".to_string(),
        "{" => "braceleft".to_string(),
        "}" => "braceright".to_string(),
        ";" => "semicolon".to_string(),
        ":" => "colon".to_string(),
        "'" => "apostrophe".to_string(),
        "\"" => "quotedbl".to_string(),
        "," => "comma".to_string(),
        "<" => "less".to_string(),
        "." => "period".to_string(),
        ">" => "greater".to_string(),
        "/" => "slash".to_string(),
        "?" => "question".to_string(),
        "-" => "minus".to_string(),
        "_" => "underscore".to_string(),
        "=" => "equal".to_string(),
        "+" => "plus".to_string(),
        "`" => "grave".to_string(),
        "~" => "asciitilde" .to_string(),
        _ => clean_key,
    };

    (
        mods_list.join(" "),
        clean_key,
    )
}

pub fn detect_conflicts(keybinds: &[crate::parser::Keybind]) -> Vec<Option<String>> {
    let mut collision_map: HashMap<(String, String, Arc<str>), Vec<usize>> = HashMap::new();

    for (i, kb) in keybinds.iter().enumerate() {
        let (sorted_mods, clean_key) = normalize(&kb.clean_mods, &kb.key);
        let submap = kb.submap.clone().unwrap_or_else(|| "".into());

        let key = (sorted_mods, clean_key, submap);
        collision_map.entry(key).or_default().push(i);
    }

    let mut results = vec![None; keybinds.len()];

    for (_, indices) in collision_map {
        if indices.len() > 1 {
            for &current_idx in &indices {
                let others: Vec<String> = indices
                    .iter()
                    .filter(|&&other_idx| other_idx != current_idx)
                    .map(|&other_idx| {
                        let kb = &keybinds[other_idx];
                        if kb.args.trim().is_empty() {
                            kb.dispatcher.to_string()
                        } else {
                            format!("{} {}", kb.dispatcher, kb.args)
                        }
                    })
                    .collect();

                if !others.is_empty() {
                    let reason = format!("Conflicts with: {}", others.join(", "));
                    results[current_idx] = Some(reason);
                }
            }
        }
    }

    results
}

fn detect_broken(keybinds: &[crate::parser::Keybind]) -> Vec<Option<String>> {
    keybinds
        .iter()
        .map(|kb| {
            let disp = kb.dispatcher.to_lowercase();
            if disp == "exec" || disp == "execr" {
                let cmd = kb.args.trim();
                if !cmd.is_empty() && !command_exists(cmd) {
                    return Some(format!(
                        "Executable not found: {}",
                        cmd.split_whitespace().next().unwrap_or("")
                    ));
                }
            }
            None
        })
        .collect()
}

static RELOAD_GENERATION: AtomicU64 = AtomicU64::new(0);

type ReloadData = (
    Vec<crate::parser::Keybind>,
    Vec<Option<String>>,
    Vec<Option<String>>,
    u64, // Generation ID
);

pub fn reload_keybinds(model: &gio::ListStore) {
    // Revert to polling loop because MainContext::channel is not available in current re-export
    let (tx, rx) = std::sync::mpsc::channel::<ReloadData>();
    let gen = RELOAD_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    std::thread::spawn(move || {
        // Invalidate command cache to reflect system changes (newly installed apps, etc)
        crate::ui::utils::execution::invalidate_command_cache();

        let keybinds = crate::parser::parse_config().unwrap_or_else(|err| {
            eprintln!("Error parsing config: {}", err);
            vec![]
        });

        let conflicts = detect_conflicts(&keybinds);
        let broken = detect_broken(&keybinds);

        let _ = tx.send((keybinds, conflicts, broken, gen));
    });

    let model = model.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
        match rx.try_recv() {
            Ok((keybinds, conflicts, broken, result_gen)) => {
                // Check if this is the latest requested generation
                if result_gen < RELOAD_GENERATION.load(Ordering::SeqCst) {
                    return glib::ControlFlow::Break;
                }

                let n_items = model.n_items();

                // Match check
                if n_items as usize == keybinds.len() {
                    let mut all_match = true;
                    for (i, kb) in keybinds.iter().enumerate() {
                        if let Some(obj) = model.item(i as u32).and_downcast::<KeybindObject>() {
                            let matches = obj.with_data(|d| {
                                d.mods.as_ref() == kb.mods.as_ref()
                                    && d.key.as_ref() == kb.key.as_ref()
                                    && d.dispatcher.as_ref() == kb.dispatcher.as_ref()
                                    && d.args.as_deref().unwrap_or("") == kb.args.as_ref()
                                    && d.submap.as_deref() == kb.submap.as_deref()
                                    && d.description.as_deref() == kb.description.as_deref()
                                    && d.flags.as_ref() == kb.flags.as_ref()
                            });
                            if !matches {
                                all_match = false;
                                break;
                            }
                        } else {
                            all_match = false;
                            break;
                        }
                    }
                    if all_match {
                        return glib::ControlFlow::Break;
                    }
                }

                let mut pool = StringPool::new();
                let mut new_objects = Vec::with_capacity(keybinds.len());
                let favs = load_favorites();

                for ((kb, conflict), is_broken) in keybinds
                    .into_iter()
                    .zip(conflicts.into_iter())
                    .zip(broken.into_iter())
                {
                    let is_fav = is_favorite(
                        &favs,
                        &kb.clean_mods,
                        &kb.key,
                        kb.submap.as_deref().unwrap_or(""),
                        &kb.dispatcher,
                        &kb.args,
                    );

                    let kb_flags = kb.flags.clone();

                    let mods = pool.intern(kb.mods);
                    let clean_mods = pool.intern(kb.clean_mods);
                    let key = pool.intern(kb.key);
                    let dispatcher = pool.intern(kb.dispatcher);
                    let args = pool.intern(kb.args);
                    let submap = kb.submap.map(|s| pool.intern(s));
                    let description = kb.description.map(|s| pool.intern(s));

                    let mods_lower = if mods.chars().any(|c: char| c.is_uppercase()) {
                        pool.intern(mods.to_lowercase().into())
                    } else {
                        mods.clone()
                    };

                    let clean_mods_lower = if clean_mods.chars().any(|c: char| c.is_uppercase()) {
                        pool.intern(clean_mods.to_lowercase().into())
                    } else {
                        clean_mods.clone()
                    };

                    let key_lower = if key.chars().any(|c: char| c.is_uppercase()) {
                        pool.intern(key.to_lowercase().into())
                    } else {
                        key.clone()
                    };

                    let dispatcher_lower = if dispatcher.chars().any(|c: char| c.is_uppercase()) {
                        pool.intern(dispatcher.to_lowercase().into())
                    } else {
                        dispatcher.clone()
                    };

                    let args_lower = if args.chars().any(|c: char| c.is_uppercase()) {
                        if args.is_empty() {
                            None
                        } else {
                            Some(pool.intern(args.to_lowercase().into()))
                        }
                    } else if args.is_empty() {
                        None
                    } else {
                        Some(args.clone())
                    };

                    let description_lower = description.as_ref().map(|desc: &Arc<str>| {
                        if desc.chars().any(|c| c.is_uppercase()) {
                            pool.intern(desc.to_lowercase().into())
                        } else {
                            desc.clone()
                        }
                    });

                    new_objects.push(KeybindObject::new(
                        crate::parser::Keybind {
                            mods: mods.clone(),
                            clean_mods: clean_mods.clone(),
                            key: key.clone(),
                            dispatcher: dispatcher.clone(),
                            args: args.clone(),
                            submap: submap.clone(),
                            description: description.clone(),
                            flags: kb_flags.clone(),
                            line_number: kb.line_number,
                            file_path: kb.file_path,
                        },
                        conflict,
                        is_broken,
                        is_fav,
                        mods_lower,
                        clean_mods_lower,
                        key_lower,
                        dispatcher_lower,
                        args_lower,
                        description_lower,
                        kb_flags,
                    ));
                }

                model.splice(0, n_items, &new_objects);
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}
