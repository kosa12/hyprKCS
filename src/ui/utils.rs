use gtk4 as gtk;
use gtk::{gio, prelude::*};
use crate::keybind_object::KeybindObject;

fn normalize(mods: &str, key: &str) -> (std::collections::BTreeSet<String>, String) {
    let mod_set: std::collections::BTreeSet<String> = mods.split_whitespace()
        .map(|s| s.to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
        
    let clean_key = key.trim().to_string();
    
    (mod_set, clean_key)
}

pub fn refresh_conflicts(model: &gio::ListStore) {
    let n = model.n_items();
    
    for i in 0..n {
        let obj1 = model.item(i).and_downcast::<KeybindObject>().unwrap();
        let mods1_str = obj1.property::<String>("clean-mods");
        let key1_str = obj1.property::<String>("key");
        let (mods1, key1) = normalize(&mods1_str, &key1_str);
        
        let mut reason = String::new();
        let mut is_conflicted = false;

        for j in 0..n {
            if i == j { continue; }
            let obj2 = model.item(j).and_downcast::<KeybindObject>().unwrap();
            let mods2_str = obj2.property::<String>("clean-mods");
            let key2_str = obj2.property::<String>("key");
            let (mods2, key2) = normalize(&mods2_str, &key2_str);
            
            if mods1 == mods2 && key1 == key2 {
                 is_conflicted = true;
                 reason = format!("Conflicts with: {} {}", mods2_str, key2_str);
                 break;
            }
        }
        
        obj1.set_property("is-conflicted", is_conflicted);
        obj1.set_property("conflict-reason", reason);
    }
}
