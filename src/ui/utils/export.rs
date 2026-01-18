use crate::keybind_object::KeybindObject;
use anyhow::Result;
use gtk4 as gtk;
use gtk::gio;
use gtk::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_keybinds_to_markdown(model: &gio::ListStore, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "# Hyprland Keybinds\n")?;
    writeln!(file, "| Modifiers | Key | Action | Arguments | Submap | Description |")?;
    writeln!(file, "|---|---|---|---|---|---|")?;

    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let mods = obj.property::<String>("clean-mods");
            let key = obj.property::<String>("key");
            let disp = obj.property::<String>("dispatcher");
            let args = obj.property::<String>("args");
            let submap = obj.property::<String>("submap");
            let desc = obj.property::<String>("description");

            writeln!(
                file,
                "| {} | {} | {} | {} | {} | {} |",
                mods, key, disp, args, submap, desc
            )?;
        }
    }

    Ok(())
}