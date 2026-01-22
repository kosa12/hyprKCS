use crate::keybind_object::KeybindObject;
use anyhow::Result;
use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_keybinds_to_markdown(model: &gio::ListStore, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "# Hyprland Keybinds\n")?;
    writeln!(file, "(Exported with HyprKCS)\n")?;
    writeln!(
        file,
        "| Modifiers | Key | Action | Arguments | Submap | Description |"
    )?;
    writeln!(file, "|---|---|---|---|---|---|")?;

    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            obj.with_data(|d| -> std::io::Result<()> {
                writeln!(
                    file,
                    "| {} | {} | {} | {} | {} | {} |",
                    d.clean_mods,
                    d.key,
                    d.dispatcher,
                    d.args.as_deref().unwrap_or(""),
                    d.submap.as_deref().unwrap_or(""),
                    d.description.as_deref().unwrap_or("")
                )
            })?;
        }
    }

    Ok(())
}
