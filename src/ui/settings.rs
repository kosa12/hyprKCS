use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::config::StyleConfig;

pub fn create_settings_view(window: &adw::ApplicationWindow, stack: &gtk::Stack) -> gtk::Widget {
    let config = Rc::new(RefCell::new(StyleConfig::load()));

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // --- Header ---
    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let back_btn = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text("Back")
        .build();
    
    let stack_c = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("home");
    });

    let title = gtk::Label::builder()
        .label("Settings")
        .css_classes(["title-2"])
        .build();

    header.append(&back_btn);
    header.append(&title);
    main_box.append(&header);
    main_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // --- Sidebar Layout ---
    let sidebar_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .build();
    
    let settings_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::None)
        .vexpand(true)
        .hexpand(true)
        .build();

    let sidebar = gtk::StackSidebar::builder()
        .stack(&settings_stack)
        .vexpand(true)
        .width_request(200) // Sidebar width
        .build();
    
    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&sidebar)
        .build();

    sidebar_box.append(&sidebar_scroll);
    sidebar_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    sidebar_box.append(&settings_stack);
    
    main_box.append(&sidebar_box);


    // ================== PAGE 1: WINDOW ==================
    let page_window = adw::PreferencesPage::builder().build();
    let group_dims = adw::PreferencesGroup::builder().title("Dimensions").build();
    
    // Width
    let width_adj = gtk::Adjustment::new(config.borrow().width as f64, 100.0, 3840.0, 10.0, 100.0, 0.0);
    let width_spin = gtk::SpinButton::builder().adjustment(&width_adj).valign(gtk::Align::Center).build();
    let width_row = adw::ActionRow::builder().title("Window Width").subtitle("px").build();
    width_row.add_suffix(&width_spin);
    let c = config.clone();
    let window_c = window.clone();
    width_spin.connect_value_changed(move |s| { 
        let val = s.value() as i32;
        {
            c.borrow_mut().width = val; 
            let _ = c.borrow().save();
        }
        let h = c.borrow().height;
        window_c.set_default_size(val, h);
        window_c.set_size_request(val, h);
        crate::ui::style::reload_style();
    });
    group_dims.add(&width_row);

    // Height
    let height_adj = gtk::Adjustment::new(config.borrow().height as f64, 100.0, 2160.0, 10.0, 100.0, 0.0);
    let height_spin = gtk::SpinButton::builder().adjustment(&height_adj).valign(gtk::Align::Center).build();
    let height_row = adw::ActionRow::builder().title("Window Height").subtitle("px").build();
    height_row.add_suffix(&height_spin);
    let c = config.clone();
    let window_c = window.clone();
    height_spin.connect_value_changed(move |s| { 
        let val = s.value() as i32;
        {
            c.borrow_mut().height = val; 
            let _ = c.borrow().save();
        }
        let w = c.borrow().width;
        window_c.set_default_size(w, val);
        window_c.set_size_request(w, val);
        crate::ui::style::reload_style();
    });
    group_dims.add(&height_row);

    // Monitor Margin
    let margin_adj = gtk::Adjustment::new(config.borrow().monitor_margin as f64, 0.0, 500.0, 1.0, 10.0, 0.0);
    let margin_spin = gtk::SpinButton::builder().adjustment(&margin_adj).valign(gtk::Align::Center).build();
    let margin_row = adw::ActionRow::builder().title("Monitor Margin").subtitle("Spacing from screen edges (px)").build();
    margin_row.add_suffix(&margin_spin);
    let c = config.clone();
    margin_spin.connect_value_changed(move |s| { 
        c.borrow_mut().monitor_margin = s.value() as i32; 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&margin_row);

     // Row Padding
    let pad_adj = gtk::Adjustment::new(config.borrow().row_padding as f64, 0.0, 50.0, 1.0, 5.0, 0.0);
    let pad_spin = gtk::SpinButton::builder().adjustment(&pad_adj).valign(gtk::Align::Center).build();
    let pad_row = adw::ActionRow::builder().title("Row Padding").subtitle("Spacing between list rows (px)").build();
    pad_row.add_suffix(&pad_spin);
    let c = config.clone();
    pad_spin.connect_value_changed(move |s| { 
        c.borrow_mut().row_padding = s.value() as i32; 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&pad_row);

    let group_style = adw::PreferencesGroup::builder().title("Window Style").build();
    
    // Opacity
    let op_adj = gtk::Adjustment::new(config.borrow().opacity.unwrap_or(1.0), 0.1, 1.0, 0.05, 0.1, 0.0);
    let op_spin = gtk::SpinButton::builder().adjustment(&op_adj).digits(2).valign(gtk::Align::Center).build();
    let op_row = adw::ActionRow::builder().title("Opacity").subtitle("0.0 - 1.0").build();
    op_row.add_suffix(&op_spin);
    let c = config.clone();
    op_spin.connect_value_changed(move |s| { 
        c.borrow_mut().opacity = Some(s.value()); 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&op_row);

    // Shadow Size
    let shadow_entry = gtk::Entry::builder().text(&config.borrow().shadow_size).valign(gtk::Align::Center).width_chars(25).build();
    let shadow_row = adw::ActionRow::builder().title("Shadow Size").subtitle("CSS box-shadow format").build();
    shadow_row.add_suffix(&shadow_entry);
    let c = config.clone();
    shadow_entry.connect_changed(move |e| { 
        c.borrow_mut().shadow_size = e.text().to_string(); 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&shadow_row);

    page_window.add(&group_dims);
    page_window.add(&group_style);
    settings_stack.add_titled(&page_window, Some("window"), "Window");


    // ================== PAGE 2: APPEARANCE ==================
    let page_app = adw::PreferencesPage::builder().build();
    let group_font = adw::PreferencesGroup::builder().title("Typography and Borders").build();

    // Font Size
    let font_entry = gtk::Entry::builder().text(config.borrow().font_size.as_deref().unwrap_or("0.9rem")).valign(gtk::Align::Center).width_chars(10).build();
    let font_row = adw::ActionRow::builder().title("Font Size").subtitle("CSS value (e.g. 12px, 1rem)").build();
    font_row.add_suffix(&font_entry);
    let c = config.clone();
    font_entry.connect_changed(move |e| { 
        c.borrow_mut().font_size = Some(e.text().to_string()); 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&font_row);

    // Border Size
    let b_size_entry = gtk::Entry::builder().text(config.borrow().border_size.as_deref().unwrap_or("1px")).valign(gtk::Align::Center).width_chars(10).build();
    let b_size_row = adw::ActionRow::builder().title("Border Size").subtitle("CSS value (e.g. 2px)").build();
    b_size_row.add_suffix(&b_size_entry);
    let c = config.clone();
    b_size_entry.connect_changed(move |e| { 
        c.borrow_mut().border_size = Some(e.text().to_string()); 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_size_row);

    // Border Radius
    let b_rad_entry = gtk::Entry::builder().text(config.borrow().border_radius.as_deref().unwrap_or("12px")).valign(gtk::Align::Center).width_chars(10).build();
    let b_rad_row = adw::ActionRow::builder().title("Border Radius").subtitle("CSS value (e.g. 10px)").build();
    b_rad_row.add_suffix(&b_rad_entry);
    let c = config.clone();
    b_rad_entry.connect_changed(move |e| { 
        c.borrow_mut().border_radius = Some(e.text().to_string()); 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_rad_row);

    // Alternating Colors
    let alt_switch = gtk::Switch::builder().active(config.borrow().alternating_row_colors).valign(gtk::Align::Center).build();
    let alt_row = adw::ActionRow::builder().title("Alternating Row Colors").activatable_widget(&alt_switch).build();
    alt_row.add_suffix(&alt_switch);
    let c = config.clone();
    alt_switch.connect_state_set(move |_, s| { 
        c.borrow_mut().alternating_row_colors = s; 
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
        glib::Propagation::Proceed 
    });
    group_font.add(&alt_row);

    page_app.add(&group_font);
    settings_stack.add_titled(&page_app, Some("appearance"), "Appearance");


    // ================== PAGE 3: UI ELEMENTS ==================
    let page_ui = adw::PreferencesPage::builder().build();
    let group_cols = adw::PreferencesGroup::builder().title("Table Columns").build();

    // Submaps
    let sub_switch = gtk::Switch::builder().active(config.borrow().show_submaps).valign(gtk::Align::Center).build();
    let sub_row = adw::ActionRow::builder().title("Show Submaps").activatable_widget(&sub_switch).build();
    sub_row.add_suffix(&sub_switch);
    let c = config.clone();
    sub_switch.connect_state_set(move |_, s| { 
        c.borrow_mut().show_submaps = s; 
        let _ = c.borrow().save();
        // crate::ui::style::reload_style(); // Not impacting style
        glib::Propagation::Proceed 
    });
    group_cols.add(&sub_row);

    // Args
    let args_switch = gtk::Switch::builder().active(config.borrow().show_args).valign(gtk::Align::Center).build();
    let args_row = adw::ActionRow::builder().title("Show Arguments").activatable_widget(&args_switch).build();
    args_row.add_suffix(&args_switch);
    let c = config.clone();
    args_switch.connect_state_set(move |_, s| { 
        c.borrow_mut().show_args = s; 
        let _ = c.borrow().save();
        // crate::ui::style::reload_style();
        glib::Propagation::Proceed 
    });
    group_cols.add(&args_row);

    // Favorites
    let fav_switch = gtk::Switch::builder().active(config.borrow().show_favorites).valign(gtk::Align::Center).build();
    let fav_row = adw::ActionRow::builder().title("Show Favorites").activatable_widget(&fav_switch).build();
    fav_row.add_suffix(&fav_switch);
    let c = config.clone();
    fav_switch.connect_state_set(move |_, s| { 
        c.borrow_mut().show_favorites = s; 
        let _ = c.borrow().save();
        // crate::ui::style::reload_style();
        glib::Propagation::Proceed 
    });
    group_cols.add(&fav_row);

    let group_sort = adw::PreferencesGroup::builder().title("Sorting").build();
    
    // Default Sort
    let sort_opts = ["Key", "Modifiers", "Action", "Arguments", "Submap"];
    let sort_list = gtk::StringList::new(&sort_opts);
    
    // Map current string to index
    let current_sort = config.borrow().default_sort.to_lowercase();
    let selected_idx = if current_sort.contains("mod") { 1 }
        else if current_sort.contains("disp") || current_sort.contains("action") { 2 }
        else if current_sort.contains("arg") { 3 }
        else if current_sort.contains("sub") { 4 }
        else { 0 }; // Default Key

    let sort_drop = gtk::DropDown::builder().model(&sort_list).selected(selected_idx).valign(gtk::Align::Center).build();
    let sort_row = adw::ActionRow::builder().title("Default Sort Column").build();
    sort_row.add_suffix(&sort_drop);
    
    let c = config.clone();
    sort_drop.connect_selected_notify(move |d| {
        let val = match d.selected() {
            1 => "mods",
            2 => "dispatcher",
            3 => "args",
            4 => "submap",
            _ => "key",
        };
        c.borrow_mut().default_sort = val.to_string();
        let _ = c.borrow().save();
    });
    group_sort.add(&sort_row);

    page_ui.add(&group_cols);
    page_ui.add(&group_sort);
    settings_stack.add_titled(&page_ui, Some("ui"), "UI Elements");


    // ================== PAGE 4: FEEDBACK ==================
    let page_feedback = adw::PreferencesPage::builder().build();
    let group_community = adw::PreferencesGroup::builder().title("Community").build();

    let create_link = |title: &str, subtitle: &str, icon: &str, url: &str| {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .activatable(true)
            .build();
        
        let img = gtk::Image::from_icon_name(icon);
        row.add_prefix(&img);
        
        let suffix = gtk::Image::from_icon_name("external-link-symbolic");
        row.add_suffix(&suffix);

        let u = url.to_string();
        let w = window.clone();
        row.connect_activated(move |_| {
            let launcher = gtk::UriLauncher::new(&u);
            launcher.launch(Some(&w), None::<&gtk::gio::Cancellable>, |res| {
                if let Err(e) = res {
                    eprintln!("Failed to launch URL: {}", e);
                }
            });
        });
        row
    };

    group_community.add(&create_link(
        "GitHub Repository", 
        "Star the project on GitHub!", 
        "starred-symbolic", 
        "https://github.com/kosa12/hyprKCS"
    ));
    group_community.add(&create_link(
        "Report a Bug or Suggest a Feature", 
        "Found an issue? Have a suggestion? Let me know.", 
        "bug-symbolic", 
        "https://github.com/kosa12/hyprKCS/issues"
    ));
    group_community.add(&create_link(
        "Donate", 
        "Support the project on Ko-fi", 
        "favorite-symbolic", 
        "https://ko-fi.com/kosa12m"
    ));
    group_community.add(&create_link(
        "Donate", 
        "Support the project on Github Sponsors", 
        "favorite-symbolic", 
        "https://github.com/sponsors/kosa12"
    ));

    page_feedback.add(&group_community);
    settings_stack.add_titled(&page_feedback, Some("feedback"), "Feedback");


    main_box.upcast::<gtk::Widget>()
}
