use crate::ui::utils::{
    create_destructive_button, create_page_header, create_pill_button, generate_diff, list_backups,
    restore_backup,
};
use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;
use std::path::Path;

pub fn create_restore_view(
    stack: &gtk::Stack,
    model: &gtk::gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    restore_container: &gtk::Box,
) -> gtk::Widget {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .vexpand(true)
        .build();

    let outer_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&container)
        .build();

    // Header
    let stack_c = stack.clone();
    let header_box = create_page_header(
        "Restore Backup",
        Some("Select a backup to restore your configuration. This will overwrite current files."),
        "Back to Settings",
        move || {
            stack_c.set_visible_child_name("settings");
        },
    );

    let warning = gtk::Label::builder()
        .label("WARNING: YOU COULD LOSE EVERYTHING CHANGED SINCE THE LAST BACKUP.")
        .css_classes(["error", "caption"])
        .halign(gtk::Align::Start)
        .margin_bottom(12)
        .wrap(true)
        .build();

    container.append(&header_box);
    container.append(&warning);
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let list_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .build();
    scroll.set_child(Some(&list_box));
    container.append(&scroll);

    let backups = list_backups().unwrap_or_default();

    if backups.is_empty() {
        let no_backups = adw::StatusPage::builder()
            .title("No Backups Found")
            .description("You haven't created any backups yet.")
            .icon_name("document-open-recent-symbolic")
            .vexpand(true)
            .build();
        list_box.append(&no_backups);
    } else {
        for path in backups {
            let row = create_backup_row(&path, stack, model, toast_overlay, restore_container);
            list_box.append(&row);
        }
    }

    outer_scroll.upcast()
}

fn format_timestamp(ts_str: &str) -> String {
    let ts_str = ts_str.trim();

    // If it's a raw unix timestamp (only digits), convert it
    if !ts_str.is_empty() && ts_str.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(secs) = ts_str.parse::<u64>() {
            let d = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
            let datetime: chrono::DateTime<chrono::Local> = d.into();
            return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        }
    }

    // Otherwise, assume it's already ISO-like (e.g. from Chrono)
    // Cleanup: 2024-01-22_12-34-56 -> 2024-01-22 12:34:56
    ts_str
        .replace('_', " ")
        .replace('-', ":")
        .replacen(':', "-", 2)
}

fn create_backup_row(
    path: &Path,
    stack: &gtk::Stack,
    model: &gtk::gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    restore_container: &gtk::Box,
) -> gtk::Widget {
    let raw_timestamp = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let timestamp = format_timestamp(&raw_timestamp);

    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();

    let diff_btn = create_pill_button("View Diff", None);
    let restore_btn = create_destructive_button("Restore", None);

    actions_box.append(&diff_btn);
    actions_box.append(&restore_btn);

    let row = crate::ui::utils::create_card_row(&timestamp, Some(&raw_timestamp), &actions_box);

    let path_c = path.to_path_buf();
    let toast_c = toast_overlay.clone();
    let stack_c = stack.clone();
    let model_c = model.clone();

    restore_btn.connect_clicked(move |_| {
        let p = path_c.clone();
        let t = toast_c.clone();
        let s = stack_c.clone();
        let m = model_c.clone();

        match restore_backup(&p) {
            Ok(msg) => {
                let toast = adw::Toast::builder()
                    .title(format!("Restore successful: {}", msg))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                t.add_toast(toast);
                crate::ui::utils::reload_keybinds(&m);
                crate::ui::style::reload_style();
                s.set_visible_child_name("home");
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(format!("Restore failed: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                t.add_toast(toast);
            }
        }
    });

    let path_diff = path.to_path_buf();
    let restore_container_c = restore_container.clone();
    let stack_diff = stack.clone();
    let model_diff = model.clone();
    let toast_diff = toast_overlay.clone();

    diff_btn.connect_clicked(move |_| {
        while let Some(child) = restore_container_c.first_child() {
            restore_container_c.remove(&child);
        }
        let diff_view = create_diff_view(
            &path_diff,
            &stack_diff,
            &model_diff,
            &toast_diff,
            &restore_container_c,
        );
        restore_container_c.append(&diff_view);
    });

    row.upcast()
}

fn create_diff_view(
    path: &Path,
    stack: &gtk::Stack,
    model: &gtk::gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    restore_container: &gtk::Box,
) -> gtk::Widget {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .vexpand(true)
        .build();

    let outer_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&container)
        .build();

    let raw_timestamp = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let timestamp = format_timestamp(&raw_timestamp);

    let restore_container_c = restore_container.clone();
    let stack_c = stack.clone();
    let model_c = model.clone();
    let toast_c = toast_overlay.clone();

    let header = create_page_header(&format!("Diff: {}", timestamp), None, "Back", move || {
        while let Some(child) = restore_container_c.first_child() {
            restore_container_c.remove(&child);
        }
        let restore_view = create_restore_view(&stack_c, &model_c, &toast_c, &restore_container_c);
        restore_container_c.append(&restore_view);
    });

    container.append(&header);

    let scroll = gtk::ScrolledWindow::builder().vexpand(true).build();

    let text_view = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::Char)
        .build();

    let buffer = text_view.buffer();
    let tag_table = buffer.tag_table();

    let tag_add = gtk::TextTag::builder()
        .name("add")
        .foreground("#26a269")
        .build();
    let tag_del = gtk::TextTag::builder()
        .name("del")
        .foreground("#c01c28")
        .build();
    let tag_header = gtk::TextTag::builder()
        .name("header")
        .foreground("#1c71d8")
        .weight(gtk::pango::Weight::Bold.into_glib())
        .build();

    tag_table.add(&tag_add);
    tag_table.add(&tag_del);
    tag_table.add(&tag_header);

    match generate_diff(path) {
        Ok(diff_text) => {
            let mut iter = buffer.start_iter();
            for line in diff_text.lines() {
                let tag_name: Option<&str> = if line.starts_with('+') && !line.starts_with("+++") {
                    Some("add")
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Some("del")
                } else if line.starts_with("---") || line.starts_with("+++") {
                    Some("header")
                } else {
                    None
                };

                let line_with_newline = format!("{}\n", line);
                if let Some(tag) = tag_name {
                    buffer.insert_with_tags_by_name(&mut iter, &line_with_newline, &[tag]);
                } else {
                    buffer.insert(&mut iter, &line_with_newline);
                }
            }
        }
        Err(e) => {
            buffer.set_text(&format!("Error generating diff: {}", e));
        }
    }

    scroll.set_child(Some(&text_view));
    container.append(&scroll);

    outer_scroll.upcast()
}
