use xkbcommon::xkb;

/// A handler for resolving keyboard layout information using `libxkbcommon`.
///
/// It provides a high-level API to translate hardware keycodes into human-readable
/// labels and keysym names based on a specific keyboard layout and variant.
pub struct XkbHandler {
    state: xkb::State,
}

impl XkbHandler {
    /// Creates a new `XkbHandler` based on layout names.
    ///
    /// # Parameters
    /// - `layout`: The keyboard layout code (e.g., "us", "de", "fr").
    /// - `variant`: The layout variant (e.g., "dvorak", "colemak"). Can be empty.
    /// - `model`: The keyboard model (e.g., "pc105", "pc104", "apple"). Default "pc105".
    /// - `options`: XKB options (e.g., "caps:escape"). Can be empty.
    ///
    /// # Returns
    /// `Some(XkbHandler)` if the keymap could be compiled, otherwise `None`.
    pub fn new(layout: &str, variant: &str, model: &str, options: &str) -> Option<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let options_opt = if options.is_empty() {
            None
        } else {
            Some(options.to_string())
        };

        let model_str = if model.is_empty() { "pc105" } else { model };

        let keymap = xkb::Keymap::new_from_names(
            &context,
            "evdev", // rules
            model_str,
            layout,
            variant,
            options_opt,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )?;

        let state = xkb::State::new(&keymap);
        Some(Self { state })
    }

    /// Creates a new `XkbHandler` from a standalone `.xkb` keymap file.
    ///
    /// # Parameters
    /// - `path`: The full path to the `.xkb` file.
    ///
    /// # Returns
    /// `Some(XkbHandler)` if the file exists and is a valid XKB v1 keymap, otherwise `None`.
    pub fn from_file(path: &str) -> Option<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let mut file = std::fs::File::open(path).ok()?;
        let keymap = xkb::Keymap::new_from_file(
            &context,
            &mut file,
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )?;

        let state = xkb::State::new(&keymap);
        Some(Self { state })
    }

    /// Resolves display information for a given hardware keycode.
    ///
    /// Labels are automatically capitalized using Unicode-aware uppercase rules.
    /// Special characters (like the German 'ß') are handled according to standard
    /// Rust `to_uppercase()` behavior (e.g., "ß" becomes "SS").
    ///
    /// # Parameters
    /// - `keycode`: The evdev hardware keycode (e.g., 16 for 'Q').
    ///   Note: This method internally adds the required offset (+8) for XKB.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `String`: A human-readable display label (e.g., "Q", "Esc", "Spc").
    /// 2. `String`: The canonical Hyprland keysym name (e.g., "Q", "Escape", "space").
    pub fn get_key_info(&self, keycode: u32) -> (String, String) {
        // xkbcommon expects XKB keycodes (evdev + 8)
        let xkb_keycode: xkb::Keycode = (keycode + 8).into();

        // Get keysym
        let keysym = self.state.key_get_one_sym(xkb_keycode);
        let keysym_name = xkb::keysym_get_name(keysym);

        // Get UTF-8 representation (Label)
        let label = self.state.key_get_utf8(xkb_keycode);

        // Prioritize known special abbreviations for clean UI display
        let special_label = match keysym_name.as_str() {
            "Escape" => Some("Esc".to_string()),
            "BackSpace" => Some("Bksp".to_string()),
            "Return" => Some("Ent".to_string()),
            "Control_L" | "Control_R" => Some("Ctrl".to_string()),
            "Alt_L" | "Alt_R" => Some("Alt".to_string()),
            "Super_L" | "Super_R" => Some("Sup".to_string()),
            "Shift_L" | "Shift_R" => Some("Shft".to_string()),
            "Caps_Lock" => Some("Caps".to_string()),
            "Tab" => Some("Tab".to_string()),
            "space" => Some("Spc".to_string()),
            "Print" => Some("Prt".to_string()),
            "Delete" => Some("Del".to_string()),
            "Left" => Some("←".to_string()),
            "Right" => Some("→".to_string()),
            "Up" => Some("↑".to_string()),
            "Down" => Some("↓".to_string()),
            "bracketleft" => Some("[".to_string()),
            "bracketright" => Some("]".to_string()),
            "braceleft" => Some("{".to_string()),
            "braceright" => Some("}".to_string()),
            "semicolon" => Some(";".to_string()),
            "colon" => Some(":".to_string()),
            "apostrophe" => Some("'".to_string()),
            "quotedbl" => Some("\"".to_string()),
            "grave" => Some("`".to_string()),
            "asciitilde" => Some("~".to_string()),
            "backslash" => Some("\\".to_string()),
            "bar" => Some("|".to_string()),
            "comma" => Some(",".to_string()),
            "less" => Some("<".to_string()),
            "period" => Some(".".to_string()),
            "greater" => Some(">".to_string()),
            "slash" => Some("/".to_string()),
            "question" => Some("?".to_string()),
            "minus" => Some("-".to_string()),
            "underscore" => Some("_".to_string()),
            "equal" => Some("=".to_string()),
            "plus" => Some("+".to_string()),
            _ => None,
        };

        let display_label = if let Some(sl) = special_label {
            sl
        } else if label.is_empty() || label.chars().any(|c| c.is_control()) {
            // Fallback to keysym name if no printable character
            if keysym_name.chars().count() > 5 {
                keysym_name
                    .chars()
                    .take(5)
                    .collect::<String>()
                    .to_uppercase()
            } else {
                keysym_name.to_uppercase()
            }
        } else {
            label.to_uppercase()
        };

        (display_label, keysym_name)
    }
}
