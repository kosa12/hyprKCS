pub mod about;
pub mod appearance;
pub mod general;
pub mod gestures;
pub mod hud;
pub mod input;
pub mod submaps;
pub mod ui_elements;
pub mod variables;
pub mod window;

use crate::config::StyleConfig;
use crate::parser::input::load_input_config;
use crate::ui::utils::create_page_header;
use gtk::gio;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Tracks which pages have been initialized for lazy loading
struct LazyPageState {
    variables: Cell<bool>,
    window: Cell<bool>,
    appearance: Cell<bool>,
    hud: Cell<bool>,
    input: Cell<bool>,
    gestures: Cell<bool>,
    submaps: Cell<bool>,
    ui_elements: Cell<bool>,
    about: Cell<bool>,
}

impl Default for LazyPageState {
    fn default() -> Self {
        Self {
            variables: Cell::new(false),
            window: Cell::new(false),
            appearance: Cell::new(false),
            hud: Cell::new(false),
            input: Cell::new(false),
            gestures: Cell::new(false),
            submaps: Cell::new(false),
            ui_elements: Cell::new(false),
            about: Cell::new(false),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_settings_view(
    window: &adw::ApplicationWindow,
    stack: gtk::Stack,
    model: &gio::ListStore,
    toast_overlay: adw::ToastOverlay,
    on_desc_toggle: Rc<dyn Fn(bool)>,
    on_fav_toggle: Rc<dyn Fn(bool)>,
    on_args_toggle: Rc<dyn Fn(bool)>,
    on_submap_toggle: Rc<dyn Fn(bool)>,
    on_close_toggle: Rc<dyn Fn(bool)>,
    on_sort_change: Rc<dyn Fn(String)>,
    on_show_toast: Rc<dyn Fn(String)>,
    on_focus_submap: Rc<dyn Fn(Option<String>)>,
    on_restore_clicked: Rc<dyn Fn()>,
) -> gtk::Widget {
    let config = Rc::new(RefCell::new(StyleConfig::load()));
    let lazy_state = Rc::new(LazyPageState::default());

    // Lazy-load input config only when needed
    let input_config: Rc<RefCell<Option<Rc<RefCell<crate::parser::input::InputConfig>>>>> =
        Rc::new(RefCell::new(None));
    let gestures_config: Rc<RefCell<Option<Rc<RefCell<crate::parser::input::GesturesConfig>>>>> =
        Rc::new(RefCell::new(None));

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // --- Header ---
    let stack_header = stack.clone();
    let header = create_page_header(
        "Settings",
        Some("Configure your preferences"),
        "Back",
        move || {
            stack_header.set_visible_child_name("home");
        },
    );
    header.set_margin_top(12);
    header.set_margin_bottom(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

    main_box.append(&header);
    main_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let close_btn_settings = header.last_child().and_downcast::<gtk::Button>();
    let on_close_toggle_settings = on_close_toggle.clone();
    let on_close_toggle_c = Rc::new(move |s: bool| {
        if let Some(btn) = &close_btn_settings {
            btn.set_visible(s);
        }
        on_close_toggle_settings(s);
    });

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
        .width_request(200)
        .build();

    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&sidebar)
        .build();

    sidebar_box.append(&sidebar_scroll);
    sidebar_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    sidebar_box.append(&settings_stack);

    main_box.append(&sidebar_box);

    // ================== PAGE 1: GENERAL (loaded immediately - it's the default) ==================
    let page_general = general::create_general_page(
        config.clone(),
        window,
        model,
        on_show_toast.clone(),
        on_restore_clicked,
    );
    settings_stack.add_titled(&page_general, Some("general"), "General");

    // ================== PLACEHOLDER PAGES (lazy loaded) ==================
    // Create lightweight placeholder boxes that will be replaced on first visit

    let placeholder_vars = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_vars, Some("variables"), "Variables");

    let placeholder_window = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_window, Some("window"), "Window");

    let placeholder_appearance = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_appearance, Some("appearance"), "Appearance");

    let placeholder_hud = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_hud, Some("hud"), "Wallpaper HUD");

    let placeholder_input = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_input, Some("input"), "Input");

    let placeholder_gestures = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_gestures, Some("gestures"), "Gestures");

    let placeholder_submaps = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_submaps, Some("submaps"), "Submaps");

    let placeholder_ui = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_ui, Some("ui"), "UI Elements");

    let placeholder_about = gtk::Box::new(gtk::Orientation::Vertical, 0);
    settings_stack.add_titled(&placeholder_about, Some("about"), "About");

    // ================== LAZY LOADING LOGIC ==================
    let config_c = config.clone();
    let window_c = window.clone();
    let model_c = model.clone();
    let stack_lazy = stack.clone();
    let toast_overlay_c = toast_overlay.clone();
    let on_show_toast_c = on_show_toast.clone();
    let on_desc_toggle_c = on_desc_toggle;
    let on_fav_toggle_c = on_fav_toggle;
    let on_args_toggle_c = on_args_toggle;
    let on_submap_toggle_c = on_submap_toggle;
    let on_close_toggle_lazy = on_close_toggle_c;
    let on_sort_change_c = on_sort_change;
    let on_focus_submap_c = on_focus_submap;
    let input_config_c = input_config;
    let gestures_config_c = gestures_config;

    settings_stack.connect_visible_child_name_notify(move |stack_inner| {
        let Some(name) = stack_inner.visible_child_name() else {
            return;
        };

        macro_rules! lazy_load {
            ($field:ident, $page_name:literal, $create_expr:expr) => {
                if name.as_str() == $page_name && !lazy_state.$field.get() {
                    lazy_state.$field.set(true);
                    let page = $create_expr;
                    replace_placeholder(stack_inner, $page_name, &page.upcast());
                }
            };
        }

        lazy_load!(
            variables,
            "variables",
            variables::create_variables_page(&window_c, on_show_toast_c.clone())
        );
        lazy_load!(
            window,
            "window",
            window::create_window_page(config_c.clone(), &window_c)
        );
        lazy_load!(
            appearance,
            "appearance",
            appearance::create_appearance_page(
                config_c.clone(),
                &window_c,
                on_show_toast_c.clone()
            )
        );

        // HUD uses a slightly different return type (Widget instead of PreferencesPage), but upcast works for both if Widget
        if name.as_str() == "hud" && !lazy_state.hud.get() {
            lazy_state.hud.set(true);
            let page = hud::create_hud_page(&model_c, on_show_toast_c.clone());
            replace_placeholder(stack_inner, "hud", &page);
        }

        if name.as_str() == "submaps" && !lazy_state.submaps.get() {
            lazy_state.submaps.set(true);
            let page = submaps::create_submaps_page(
                &model_c,
                config_c.clone(),
                &stack_lazy,
                &toast_overlay_c,
                on_focus_submap_c.clone(),
            );
            replace_placeholder(stack_inner, "submaps", &page.upcast());
        }

        lazy_load!(about, "about", about::create_about_page(&window_c));

        lazy_load!(
            ui_elements,
            "ui",
            ui_elements::create_ui_elements_page(
                config_c.clone(),
                on_desc_toggle_c.clone(),
                on_fav_toggle_c.clone(),
                on_args_toggle_c.clone(),
                on_submap_toggle_c.clone(),
                on_close_toggle_lazy.clone(),
                on_sort_change_c.clone(),
            )
        );

        // Input and Gestures share config loading logic
        if name.as_str() == "input" || name.as_str() == "gestures" {
            ensure_input_config_loaded(&input_config_c, &gestures_config_c);

            if name.as_str() == "input" && !lazy_state.input.get() {
                lazy_state.input.set(true);
                let ic = input_config_c.borrow().as_ref().unwrap().clone();
                let gc = gestures_config_c.borrow().as_ref().unwrap().clone();
                let page = input::create_input_page(ic, gc, on_show_toast_c.clone());
                replace_placeholder(stack_inner, "input", &page.upcast());
            } else if name.as_str() == "gestures" && !lazy_state.gestures.get() {
                lazy_state.gestures.set(true);
                let ic = input_config_c.borrow().as_ref().unwrap().clone();
                let gc = gestures_config_c.borrow().as_ref().unwrap().clone();
                let page = gestures::create_gestures_page(ic, gc, on_show_toast_c.clone());
                replace_placeholder(stack_inner, "gestures", &page.upcast());
            }
        }
    });

    main_box.upcast::<gtk::Widget>()
}

/// Populate a placeholder box with the actual page content
fn replace_placeholder(stack: &gtk::Stack, name: &str, new_page: &gtk::Widget) {
    if let Some(placeholder) = stack.child_by_name(name) {
        if let Some(container) = placeholder.downcast_ref::<gtk::Box>() {
            // Clear placeholder and add actual content
            container.append(new_page);
            new_page.set_vexpand(true);
            new_page.set_hexpand(true);
        }
    }
}

/// Ensure input/gestures configs are loaded (shared between input and gestures pages)
fn ensure_input_config_loaded(
    input_config: &Rc<RefCell<Option<Rc<RefCell<crate::parser::input::InputConfig>>>>>,
    gestures_config: &Rc<RefCell<Option<Rc<RefCell<crate::parser::input::GesturesConfig>>>>>,
) {
    if input_config.borrow().is_none() {
        let (ic, gc) = match load_input_config() {
            Ok((i, g)) => (i, g),
            Err(e) => {
                eprintln!("Failed to load input/gestures config: {}", e);
                (
                    crate::parser::input::InputConfig::default(),
                    crate::parser::input::GesturesConfig::default(),
                )
            }
        };
        *input_config.borrow_mut() = Some(Rc::new(RefCell::new(ic)));
        *gestures_config.borrow_mut() = Some(Rc::new(RefCell::new(gc)));
    }
}
