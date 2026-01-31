use crate::config::StyleConfig;
use crate::ui::utils::reload_keybinds;
use gtk::{gio, glib};
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_general_page(
    config: Rc<RefCell<StyleConfig>>,
    window: &adw::ApplicationWindow,
    model: &gio::ListStore,
    on_show_toast: Rc<dyn Fn(String)>,
    on_restore_clicked: Rc<dyn Fn()>,
) -> adw::PreferencesPage {
    let page_general = adw::PreferencesPage::builder().build();

    // Configuration Group
    let group_config = adw::PreferencesGroup::builder()
        .title("Configuration")
        .build();

    let config_path_row = adw::ActionRow::builder()
        .title("Alternative Config Path")
        .subtitle(
            config
                .borrow()
                .alternative_config_path
                .as_deref()
                .unwrap_or("Default (System)"),
        )
        .build();

    let browse_btn = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .valign(gtk::Align::Center)
        .tooltip_text("Browse for configuration folder")
        .build();

    let clear_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .valign(gtk::Align::Center)
        .tooltip_text("Reset to default")
        .visible(config.borrow().alternative_config_path.is_some())
        .build();

    let window_weak = window.downgrade();
    let config_c = config.clone();
    let row_c = config_path_row.clone();
    let clear_btn_c = clear_btn.clone();
    let toast_cb = on_show_toast.clone();
    let model_browse = model.clone();

    browse_btn.connect_clicked(move |_| {
        let dialog = gtk::FileDialog::builder()
            .title("Select Configuration Folder")
            .modal(true)
            .build();

        let window = window_weak.upgrade();
        let c = config_c.clone();
        let r = row_c.clone();
        let cb = clear_btn_c.clone();
        let t = toast_cb.clone();
        let m = model_browse.clone();

        dialog.select_folder(
            window.as_ref(),
            None::<&gtk::gio::Cancellable>,
            move |res| match res {
                Ok(file) => {
                    if let Some(path) = file.path() {
                        let path_str = path.to_string_lossy().to_string();
                        c.borrow_mut().alternative_config_path = Some(path_str.clone());
                        let _ = c.borrow().save();
                        r.set_subtitle(&path_str);
                        cb.set_visible(true);
                        t(format!("Config path set to: {}", path_str));

                        // Refetch binds from new path
                        reload_keybinds(&m);
                    }
                }
                Err(e) => {
                    println!("Folder selection cancelled/error: {}", e);
                }
            },
        );
    });

    let config_c = config.clone();
    let row_c = config_path_row.clone();
    // let clear_btn_c = clear_btn.clone(); // Unused here
    let toast_cb = on_show_toast.clone();
    let model_clear = model.clone();

    clear_btn.connect_clicked(move |btn| {
        config_c.borrow_mut().alternative_config_path = None;
        let _ = config_c.borrow().save();
        row_c.set_subtitle("Default (System)");
        btn.set_visible(false);
        toast_cb("Reset config path to default".to_string());

        // Refetch binds from default path
        reload_keybinds(&model_clear);
    });

    config_path_row.add_suffix(&browse_btn);
    config_path_row.add_suffix(&clear_btn);
    group_config.add(&config_path_row);
    page_general.add(&group_config);

    let group_backup = adw::PreferencesGroup::builder()
        .title("Backup and Restore")
        .build();

    // Auto-Backup
    let auto_backup_switch = gtk::Switch::builder()
        .active(config.borrow().auto_backup)
        .valign(gtk::Align::Center)
        .build();
    let auto_backup_row = adw::ActionRow::builder()
        .title("Auto-Backup")
        .subtitle("Backup config on every save")
        .activatable_widget(&auto_backup_switch)
        .build();
    auto_backup_row.add_suffix(&auto_backup_switch);
    let c = config.clone();
    auto_backup_switch.connect_state_set(move |_, s| {
        c.borrow_mut().auto_backup = s;
        let _ = c.borrow().save();
        glib::Propagation::Proceed
    });
    group_backup.add(&auto_backup_row);

    // Max Backups Enabled
    let max_backups_switch = gtk::Switch::builder()
        .active(config.borrow().max_backups_enabled)
        .valign(gtk::Align::Center)
        .build();
    let max_backups_row = adw::ActionRow::builder()
        .title("Limit Backups")
        .subtitle("Delete old backups")
        .activatable_widget(&max_backups_switch)
        .build();
    max_backups_row.add_suffix(&max_backups_switch);

    // Backup Count
    let count_adj = gtk::Adjustment::new(
        config.borrow().max_backups_count as f64,
        1.0,
        1000.0,
        1.0,
        10.0,
        0.0,
    );
    let count_spin = gtk::SpinButton::builder()
        .adjustment(&count_adj)
        .valign(gtk::Align::Center)
        .build();
    let count_row = adw::ActionRow::builder()
        .title("Max Backups")
        .subtitle("Number of backups to keep")
        .build();
    count_row.add_suffix(&count_spin);
    let c = config.clone();
    count_spin.connect_value_changed(move |s| {
        c.borrow_mut().max_backups_count = s.value() as i32;
        let _ = c.borrow().save();
    });

    // Initial state
    count_row.set_sensitive(config.borrow().max_backups_enabled);

    let count_row_ref = count_row.clone();
    let c = config.clone();
    max_backups_switch.connect_state_set(move |_, s| {
        c.borrow_mut().max_backups_enabled = s;
        let _ = c.borrow().save();
        count_row_ref.set_sensitive(s);
        glib::Propagation::Proceed
    });
    group_backup.add(&max_backups_row);
    group_backup.add(&count_row);

    let restore_row = adw::ActionRow::builder()
        .title("Restore Backup")
        .subtitle("Restore configuration from a previous backup")
        .activatable(true)
        .build();

    let restore_icon = gtk::Image::from_icon_name("document-revert-symbolic");
    restore_row.add_prefix(&restore_icon);

    let restore_suffix = gtk::Image::from_icon_name("go-next-symbolic");
    restore_row.add_suffix(&restore_suffix);

    let on_restore = on_restore_clicked.clone();
    restore_row.connect_activated(move |_| {
        on_restore();
    });
    group_backup.add(&restore_row);

    page_general.add(&group_backup);

    // Export Group
    let group_export = adw::PreferencesGroup::builder().title("Export").build();

    let export_row = adw::ActionRow::builder()
        .title("Export Keybinds")
        .subtitle("Save all keybinds to a Markdown file")
        .activatable(true)
        .build();
    let export_icon = gtk::Image::from_icon_name("document-save-as-symbolic");
    export_row.add_prefix(&export_icon);

    let suffix = gtk::Image::from_icon_name("go-next-symbolic");
    export_row.add_suffix(&suffix);

    let window_c = window.clone();
    let model_c = model.clone();
    let toast_cb = on_show_toast.clone();

    export_row.connect_activated(move |_| {
        let file_dialog = gtk::FileDialog::builder()
            .title("Export Keybinds")
            .accept_label("Export")
            .initial_name("keybinds.md")
            .build();

        let m = model_c.clone();
        let t_cb = toast_cb.clone();
        file_dialog.save(
            Some(&window_c),
            None::<&gtk::gio::Cancellable>,
            move |res| match res {
                Ok(file) => {
                    if let Some(path) = file.path() {
                        match crate::ui::utils::export_keybinds_to_markdown(&m, &path) {
                            Ok(_) => t_cb(format!("Successfully exported to {:?}", path)),
                            Err(e) => t_cb(format!("Export failed: {}", e)),
                        }
                    }
                }
                Err(e) => {
                    println!("Export cancelled/error: {}", e);
                }
            },
        );
    });
    group_export.add(&export_row);
    page_general.add(&group_export);

    page_general
}
