use crate::ui::utils::{generate_diff, list_backups, restore_backup};
use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

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
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let back_btn = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text("Back to Settings")
        .build();

    let stack_c = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("settings");
    });

    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let title = gtk::Label::builder()
        .label("Restore Backup")
        .css_classes(["title-1"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    let subtitle = gtk::Label::builder()
        .label("Select a backup to restore your configuration. This will overwrite current files.")
        .css_classes(["dim-label"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();

    let warning = gtk::Label::builder()
        .label("WARNING: YOU COULD LOSE EVERYTHING CHANGED SINCE THE LAST BACKUP.")
        .css_classes(["error", "caption"])
        .halign(gtk::Align::Start)
        .margin_bottom(12)
        .wrap(true)
        .build();

    title_box.append(&title);
    title_box.append(&subtitle);
    title_box.append(&warning);

    header_box.append(&back_btn);
    header_box.append(&title_box);
    container.append(&header_box);

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

fn create_backup_row(
    path: &std::path::PathBuf,
    stack: &gtk::Stack,
    model: &gtk::gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    restore_container: &gtk::Box,
) -> gtk::Widget {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .css_classes(["card"])
        .margin_start(4)
        .margin_end(4)
        .build();

    let info_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .build();

    let timestamp = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let title = gtk::Label::builder()
        .label(&timestamp)
        .halign(gtk::Align::Start)
        .css_classes(["heading"])
        .build();

    let path_label = gtk::Label::builder()
        .label(path.to_string_lossy().to_string())
        .halign(gtk::Align::Start)
        .css_classes(["caption", "dim-label"])
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .max_width_chars(40)
        .wrap(true)
        .build();

    info_box.append(&title);
    info_box.append(&path_label);
    row.append(&info_box);

    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .valign(gtk::Align::Center)
        .margin_end(12)
        .build();

    let diff_btn = gtk::Button::builder()
        .label("View Diff")
        .css_classes(["pill"])
        .build();

    let restore_btn = gtk::Button::builder()
        .label("Restore")
        .css_classes(["destructive-action", "pill"])
        .build();

    actions_box.append(&diff_btn);
    actions_box.append(&restore_btn);
    row.append(&actions_box);

    let path_c = path.clone();
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
                let toast = adw::Toast::new(&format!("Restore successful: {}", msg));
                t.add_toast(toast);
                crate::ui::utils::reload_keybinds(&m);
                crate::ui::style::reload_style();
                s.set_visible_child_name("home");
            }
            Err(e) => {
                let toast = adw::Toast::new(&format!("Restore failed: {}", e));
                t.add_toast(toast);
            }
        }
    });

    let path_diff = path.clone();
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
    path: &std::path::PathBuf,
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

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let back_btn = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular"])
        .build();

    let restore_container_c = restore_container.clone();
    let stack_c = stack.clone();
    let model_c = model.clone();
    let toast_c = toast_overlay.clone();
    back_btn.connect_clicked(move |_| {
        while let Some(child) = restore_container_c.first_child() {
            restore_container_c.remove(&child);
        }
        let restore_view = create_restore_view(&stack_c, &model_c, &toast_c, &restore_container_c);
        restore_container_c.append(&restore_view);
    });

    let timestamp = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let title = gtk::Label::builder()
        .label(&format!("Diff: {}", timestamp))
        .css_classes(["title-2"])
        .halign(gtk::Align::Start)
        .build();

    header.append(&back_btn);
    header.append(&title);
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
                let tag_name = if line.starts_with('+') && !line.starts_with("+++") {
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
