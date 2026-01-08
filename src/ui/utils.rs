use gtk4 as gtk;
use gtk::{gio, prelude::*};
use crate::keybind_object::KeybindObject;

pub fn refresh_conflicts(model: &gio::ListStore) {
    let mut counts = std::collections::HashMap::new();
    
    // First pass: count occurrences
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let mods = obj.property::<String>("mods");
            let key = obj.property::<String>("key");
            
            // Normalizing: remove anything in brackets `[...]` and trim.
            let clean_mods = if let Some(idx) = mods.find(']') {
                if let Some(start) = mods.find('[') {
                     if start < idx {
                         mods[idx+1..].trim().to_string()
                     } else {
                         mods
                     }
                } else {
                    mods
                }
            } else {
                mods
            };
            
            let key_tuple = (clean_mods.to_lowercase(), key.to_lowercase());
            *counts.entry(key_tuple).or_insert(0) += 1;
        }
    }
    
    // Second pass: update is-conflicted
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let mods = obj.property::<String>("mods");
            let key = obj.property::<String>("key");
            
            let clean_mods = if let Some(idx) = mods.find(']') {
                if let Some(start) = mods.find('[') {
                     if start < idx {
                         mods[idx+1..].trim().to_string()
                     } else {
                         mods
                     }
                } else {
                    mods
                }
            } else {
                mods
            };
            
            let count = counts.get(&(clean_mods.to_lowercase(), key.to_lowercase())).unwrap_or(&0);
            obj.set_property("is-conflicted", *count > 1);
        }
    }
}
