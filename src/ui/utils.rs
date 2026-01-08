use gtk4 as gtk;
use gtk::{gio, prelude::*};
use crate::keybind_object::KeybindObject;

pub fn refresh_conflicts(model: &gio::ListStore) {
    let mut counts = std::collections::HashMap::new();
    
    // First pass: count occurrences using resolved modifiers
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let clean_mods = obj.property::<String>("clean-mods");
            let key = obj.property::<String>("key");
            
            let key_tuple = (clean_mods.to_lowercase(), key.to_lowercase());
            *counts.entry(key_tuple).or_insert(0) += 1;
        }
    }
    
    // Second pass: update is-conflicted
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let clean_mods = obj.property::<String>("clean-mods");
            let key = obj.property::<String>("key");
            
            let count = counts.get(&(clean_mods.to_lowercase(), key.to_lowercase())).unwrap_or(&0);
            obj.set_property("is-conflicted", *count > 1);
        }
    }
}
