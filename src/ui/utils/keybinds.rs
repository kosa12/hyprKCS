use crate::config::favorites::{is_favorite, load_favorites};
use crate::keybind_object::KeybindObject;
use gtk::gio;
use gtk4 as gtk;

pub fn normalize(mods: &str, key: &str) -> (Vec<String>, String) {
    let mut mod_vec: Vec<String> = mods
        .replace("+", " ") // Handle Super+Shift style
        .split_whitespace()
        .map(|s| s.to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    mod_vec.sort();
    mod_vec.dedup();

    let clean_key = key.trim().to_lowercase();

    (mod_vec, clean_key)
}

fn detect_conflicts(keybinds: &[crate::parser::Keybind]) -> Vec<Option<String>> {
    let mut collision_map: std::collections::HashMap<(Vec<String>, String, String), Vec<usize>> =
        std::collections::HashMap::new();

    for (i, kb) in keybinds.iter().enumerate() {
        let (sorted_mods, clean_key) = normalize(&kb.clean_mods, &kb.key);
        let submap = kb.submap.clone().unwrap_or_default();

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
                            kb.dispatcher.clone()
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

pub fn reload_keybinds(model: &gio::ListStore) {
    model.remove_all();

    let keybinds = crate::parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    let conflicts = detect_conflicts(&keybinds);
    let favs = load_favorites();

    for (kb, conflict) in keybinds.into_iter().zip(conflicts.into_iter()) {
        let is_fav = is_favorite(
            &favs,
            &kb.clean_mods,
            &kb.key,
            kb.submap.as_deref().unwrap_or(""),
            &kb.dispatcher,
            &kb.args,
        );
        model.append(&KeybindObject::new(kb, conflict, is_fav));
    }
}
