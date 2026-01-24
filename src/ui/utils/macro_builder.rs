use crate::ui::utils::setup_dispatcher_completion;
use gtk::prelude::*;
use gtk4 as gtk;

pub fn create_macro_row(
    dispatcher_val: Option<&str>,
    args_val: Option<&str>,
) -> (gtk::Box, gtk::Entry, gtk::Entry, gtk::Button) {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let disp_entry = gtk::Entry::builder()
        .placeholder_text("Dispatcher")
        .hexpand(true)
        .text(dispatcher_val.unwrap_or(""))
        .build();
    setup_dispatcher_completion(&disp_entry);

    let args_entry = gtk::Entry::builder()
        .placeholder_text("Arguments")
        .hexpand(true)
        .text(args_val.unwrap_or(""))
        .build();

    let delete_btn = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(["flat", "destructive-action"])
        .tooltip_text("Remove Action")
        .build();

    row.append(&disp_entry);
    row.append(&args_entry);
    row.append(&delete_btn);

    (row, disp_entry, args_entry, delete_btn)
}

pub fn compile_macro(container: &gtk::Box) -> Option<(String, String)> {
    let mut commands = Vec::new();

    let mut child = container.first_child();
    while let Some(widget) = child {
        if let Some(box_widget) = widget.downcast_ref::<gtk::Box>() {
            let mut iter = box_widget.first_child();
            let disp_widget = iter.and_then(|w| w.downcast::<gtk::Entry>().ok());
            iter = box_widget.first_child().and_then(|w| w.next_sibling());
            let args_widget = iter.and_then(|w| w.downcast::<gtk::Entry>().ok());

            if let (Some(d), Some(a)) = (disp_widget, args_widget) {
                let d_text = d.text().to_string();
                let a_text = a.text().to_string();

                if !d_text.trim().is_empty() {
                    let cmd = if a_text.trim().is_empty() {
                        format!("hyprctl dispatch {}", d_text.trim())
                    } else {
                        let escaped_args = a_text.replace('"', "\\\"");
                        format!("hyprctl dispatch {} \"{}\"", d_text.trim(), escaped_args)
                    };
                    commands.push(cmd);
                }
            }
        }
        child = widget.next_sibling();
    }

    if commands.is_empty() {
        return None;
    }

    let script = commands.join("; ");
    Some(("exec".to_string(), format!("bash -c \"{}\"", script)))
}

/// Tries to parse a `bash -c "hyprctl ..."` string back into a list of actions.
pub fn parse_macro(dispatcher: &str, args: &str) -> Option<Vec<(String, String)>> {
    if dispatcher != "exec" && dispatcher != "execr" {
        return None;
    }

    let trimmed_args = args.trim();
    if !trimmed_args.starts_with("bash -c \"") || !trimmed_args.ends_with('"') {
        return None;
    }

    // Extract inner content
    let inner = &trimmed_args[9..trimmed_args.len() - 1]; // strip `bash -c "` and `"`

    let mut actions = Vec::new();
    for part in inner.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some(rest) = part.strip_prefix("hyprctl dispatch ") {
            let rest = rest.trim();
            let (disp, arg) = rest.split_once(' ').unwrap_or((rest, ""));
            let arg = arg.trim();

            // Unescape quotes if we added them: "arg" -> arg
            let clean_arg = if arg.starts_with('"') && arg.ends_with('"') {
                &arg[1..arg.len() - 1]
            } else {
                arg
            };

            actions.push((disp.to_string(), clean_arg.replace("\\\"", "\"")));
        } else {
            // Found something that isn't hyprctl dispatch -> abort parsing
            return None;
        }
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}
