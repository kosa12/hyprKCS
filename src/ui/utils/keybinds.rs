use crate::config::favorites::{is_favorite, load_favorites};
use crate::keybind_object::KeybindObject;
use crate::ui::utils::execution::command_exists;
use gtk::gio;
use gtk4 as gtk;
use std::rc::Rc;

pub fn normalize(mods: &str, key: &str) -> (String, String) {
    let mut mods_list: Vec<&str> = mods
        .split(|c: char| c == '+' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    mods_list.sort_unstable();
    mods_list.dedup();

    (
        mods_list.join(" ").to_uppercase(),
        key.trim().to_lowercase(),
    )
}

fn detect_conflicts(keybinds: &[crate::parser::Keybind]) -> Vec<Option<String>> {
    let mut collision_map: std::collections::HashMap<(String, String, Rc<str>), Vec<usize>> =
        std::collections::HashMap::new();

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
                // Find all OTHER binds in this group to describe what it conflicts with
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

pub fn reload_keybinds(model: &gio::ListStore) {
    model.remove_all();

    let keybinds = crate::parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    let conflicts = detect_conflicts(&keybinds);
    let broken = detect_broken(&keybinds);
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
        model.append(&KeybindObject::new(kb, conflict, is_broken, is_fav));
    }
}
