pub mod about;
pub mod appearance;
pub mod feedback;
pub mod general;
pub mod gestures;
pub mod input;
pub mod ui_elements;
pub mod window;

use crate::config::StyleConfig;
use crate::parser::input::load_input_config;
use crate::ui::utils::create_page_header;
use gtk::gio;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(clippy::too_many_arguments)]
pub fn create_settings_view(
    window: &adw::ApplicationWindow,
    stack: &gtk::Stack,
    model: &gio::ListStore,
    on_desc_toggle: Rc<dyn Fn(bool)>,
    on_fav_toggle: Rc<dyn Fn(bool)>,
    on_args_toggle: Rc<dyn Fn(bool)>,
    on_submap_toggle: Rc<dyn Fn(bool)>,
    on_sort_change: Rc<dyn Fn(String)>,
    on_show_toast: Rc<dyn Fn(String)>,
    on_restore_clicked: Rc<dyn Fn()>,
) -> gtk::Widget {
    let config = Rc::new(RefCell::new(StyleConfig::load()));

    // Load input config early to share between input and gestures pages
    let (input_config, gestures_config) = match load_input_config() {
        Ok((i, g)) => (Rc::new(RefCell::new(i)), Rc::new(RefCell::new(g))),
        Err(e) => {
            eprintln!("Failed to load input/gestures config: {}", e);
            (
                Rc::new(RefCell::new(crate::parser::input::InputConfig::default())),
                Rc::new(RefCell::new(crate::parser::input::GesturesConfig::default())),
            )
        }
    };

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // --- Header ---
    let stack_c = stack.clone();
    let header = create_page_header(
        "Settings",
        Some("Configure your preferences"),
        "Back",
        move || {
            stack_c.set_visible_child_name("home");
        },
    );
    header.set_margin_top(12);
    header.set_margin_bottom(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

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

    // ================== PAGE 1: GENERAL ==================
    let page_general = general::create_general_page(
        config.clone(),
        window,
        model,
        on_show_toast.clone(),
        on_restore_clicked,
    );
    settings_stack.add_titled(&page_general, Some("general"), "General");

    // ================== PAGE 2: WINDOW ==================
    let page_window = window::create_window_page(config.clone(), window);
    settings_stack.add_titled(&page_window, Some("window"), "Window");

    // ================== PAGE 2.5: APPEARANCE ==================
    let page_appearance = appearance::create_appearance_page(config.clone());
    settings_stack.add_titled(&page_appearance, Some("appearance"), "Appearance");

    // ================== PAGE 2.6: INPUT ==================
    let page_input = input::create_input_page(
        input_config.clone(),
        gestures_config.clone(),
        on_show_toast.clone(),
    );
    settings_stack.add_titled(&page_input, Some("input"), "Input");

    // ================== PAGE 2.7: GESTURES ==================
    let page_gestures = gestures::create_gestures_page(
        input_config.clone(),
        gestures_config.clone(),
        on_show_toast.clone(),
    );
    settings_stack.add_titled(&page_gestures, Some("gestures"), "Gestures");

    // ================== PAGE 3: UI ELEMENTS ==================
    let page_ui = ui_elements::create_ui_elements_page(
        config.clone(),
        on_desc_toggle,
        on_fav_toggle,
        on_args_toggle,
        on_submap_toggle,
        on_sort_change,
    );
    settings_stack.add_titled(&page_ui, Some("ui"), "UI Elements");

    // ================== PAGE 4: FEEDBACK ==================
    let page_feedback = feedback::create_feedback_page(window);
    settings_stack.add_titled(&page_feedback, Some("feedback"), "Feedback");

    // ================== PAGE 5: ABOUT ==================
    let page_about = about::create_about_page(window);
    settings_stack.add_titled(&page_about, Some("about"), "About");

    main_box.upcast::<gtk::Widget>()
}
