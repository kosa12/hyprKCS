use crate::parser;
use crate::ui::utils::components::{get_flag_from_index, get_mouse_code_from_index};
use crate::ui::utils::macro_builder::compile_macro;
use crate::ui::utils::{create_pill_button, reload_keybinds};
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::path::PathBuf;

#[allow(deprecated)]
pub struct CloneContext {
    pub file_path: PathBuf,
    pub model: gio::ListStore,
    pub toast_overlay: adw::ToastOverlay,
    pub stack: gtk::Stack,

    pub entry_mods: gtk::Entry,
    pub entry_key: gtk::Entry,
    pub entry_dispatcher: gtk::Entry,
    pub entry_args: gtk::Entry,
    pub entry_desc: gtk::Entry,
    pub entry_submap: gtk::ComboBoxText,

    pub macro_switch: gtk::Switch,
    pub macro_list: gtk::Box,
    pub flags_dropdown: gtk::DropDown,
    pub mouse_switch: gtk::Switch,
    pub mouse_dropdown: gtk::DropDown,

    pub mods_had_prefix: bool,
    pub args_had_prefix: bool,
}

pub fn create_clone_button(ctx: CloneContext) -> gtk::Button {
    let btn = create_pill_button("Clone", Some("edit-copy-symbolic"));
    btn.set_tooltip_text(Some("Save as a new keybind"));

    // We need to move the context into the closure.
    // Since we can't move fields out of the struct easily without destructuring,
    // we'll clone what we need. Since GTK widgets are ref-counted (internally), cloning them is cheap.

    let file_path = ctx.file_path.clone();
    let model = ctx.model.clone();
    let toast_overlay = ctx.toast_overlay.clone();
    let stack = ctx.stack.clone();
    let entry_mods = ctx.entry_mods.clone();
    let entry_key = ctx.entry_key.clone();
    let entry_dispatcher = ctx.entry_dispatcher.clone();
    let entry_args = ctx.entry_args.clone();
    let entry_desc = ctx.entry_desc.clone();
    let entry_submap = ctx.entry_submap.clone();
    let macro_switch = ctx.macro_switch.clone();
    let macro_list = ctx.macro_list.clone();
    let flags_dropdown = ctx.flags_dropdown.clone();
    let mouse_switch = ctx.mouse_switch.clone();
    let mouse_dropdown = ctx.mouse_dropdown.clone();
    let mods_had_prefix = ctx.mods_had_prefix;
    let args_had_prefix = ctx.args_had_prefix;

    btn.connect_clicked(move |_| {
        let input_mods = entry_mods.text().to_string();
        let new_mods = if mods_had_prefix {
            format!("${}", input_mods)
        } else {
            input_mods
        };

        let new_key = if mouse_switch.is_active() {
            get_mouse_code_from_index(mouse_dropdown.selected()).to_string()
        } else {
            entry_key.text().to_string()
        };

        let desc = entry_desc.text().to_string();
        let new_flag = get_flag_from_index(flags_dropdown.selected());

        #[allow(deprecated)]
        let submap_id = entry_submap.active_id();
        let new_submap = if let Some(id) = submap_id {
            if id.is_empty() {
                None
            } else {
                Some(id.to_string())
            }
        } else {
            #[allow(deprecated)]
            if let Some(text) = entry_submap.active_text() {
                let t = text.as_str().trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            } else {
                None
            }
        };

        let (new_dispatcher, new_args) = if macro_switch.is_active() {
            match compile_macro(&macro_list) {
                Some(res) => res,
                None => {
                    let toast = adw::Toast::builder()
                        .title("Macro is empty or invalid")
                        .timeout(crate::config::constants::TOAST_TIMEOUT)
                        .build();
                    toast_overlay.add_toast(toast);
                    return;
                }
            }
        } else {
            let d = entry_dispatcher.text().to_string();
            let input_args = entry_args.text().to_string();
            let a = if args_had_prefix {
                format!("${}", input_args)
            } else {
                input_args
            };
            (d, a)
        };

        match parser::add_keybind(
            file_path.clone(),
            &new_mods,
            &new_key,
            &new_dispatcher,
            &new_args,
            new_submap,
            if desc.is_empty() { None } else { Some(desc) },
            new_flag,
        ) {
            Ok(_) => {
                reload_keybinds(&model);
                let toast = adw::Toast::builder()
                    .title("Keybind cloned successfully")
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay.add_toast(toast);
                stack.set_visible_child_name("home");
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(format!("Error cloning: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay.add_toast(toast);
            }
        }
    });

    btn
}
