pub struct KeyDef {
    pub label: &'static str,
    pub hypr_name: &'static str,
    pub width: f64,
}

impl KeyDef {
    pub const fn new(label: &'static str, hypr_name: &'static str, width: f64) -> Self {
        Self {
            label,
            hypr_name,
            width,
        }
    }
}

pub const ROW_FUNC: &[KeyDef] = &[
    KeyDef::new("Esc", "Escape", 1.0),
    KeyDef::new("F1", "F1", 1.0),
    KeyDef::new("F2", "F2", 1.0),
    KeyDef::new("F3", "F3", 1.0),
    KeyDef::new("F4", "F4", 1.0),
    KeyDef::new("F5", "F5", 1.0),
    KeyDef::new("F6", "F6", 1.0),
    KeyDef::new("F7", "F7", 1.0),
    KeyDef::new("F8", "F8", 1.0),
    KeyDef::new("F9", "F9", 1.0),
    KeyDef::new("F10", "F10", 1.0),
    KeyDef::new("F11", "F11", 1.0),
    KeyDef::new("F12", "F12", 1.0),
    KeyDef::new("PrtSc", "Print", 1.0),
    KeyDef::new("Del", "Delete", 1.0),
];

pub const ROW_ARROWS: &[KeyDef] = &[
    KeyDef::new("<", "Left", 1.0),
    KeyDef::new("v", "Down", 1.0),
    KeyDef::new("^", "Up", 1.0),
    KeyDef::new(">", "Right", 1.0),
];

// --- ANSI Layout ---

pub const ANSI_ROW_1: &[KeyDef] = &[
    KeyDef::new("`", "grave", 1.0),
    KeyDef::new("1", "1", 1.0),
    KeyDef::new("2", "2", 1.0),
    KeyDef::new("3", "3", 1.0),
    KeyDef::new("4", "4", 1.0),
    KeyDef::new("5", "5", 1.0),
    KeyDef::new("6", "6", 1.0),
    KeyDef::new("7", "7", 1.0),
    KeyDef::new("8", "8", 1.0),
    KeyDef::new("9", "9", 1.0),
    KeyDef::new("0", "0", 1.0),
    KeyDef::new("-", "minus", 1.0),
    KeyDef::new("=", "equal", 1.0),
    KeyDef::new("Bksp", "BackSpace", 2.0),
];

pub const ANSI_ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Y", "Y", 1.0),
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("[", "bracketleft", 1.0),
    KeyDef::new("]", "bracketright", 1.0),
    KeyDef::new("\\", "backslash", 1.5),
];

pub const ANSI_ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new(";", "semicolon", 1.0),
    KeyDef::new("'", "apostrophe", 1.0),
    KeyDef::new("Enter", "Return", 2.25),
];

pub const ANSI_ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 2.25),
    KeyDef::new("Z", "Z", 1.0),
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new("/", "slash", 1.0),
    KeyDef::new("Shift", "Shift_R", 2.75),
];

pub const ANSI_ROW_5: &[KeyDef] = &[
    KeyDef::new("Ctrl", "Control_L", 1.25),
    KeyDef::new("Sup", "Super_L", 1.25),
    KeyDef::new("Alt", "Alt_L", 1.25),
    KeyDef::new("Space", "space", 6.25),
    KeyDef::new("Alt", "Alt_R", 1.25),
    KeyDef::new("Sup", "Super_R", 1.25),
    KeyDef::new("Menu", "Menu", 1.25),
    KeyDef::new("Ctrl", "Control_R", 1.25),
];

// --- ISO Layout (UK-ish/International) ---

pub const ISO_ROW_1: &[KeyDef] = ANSI_ROW_1;

pub const ISO_ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Y", "Y", 1.0),
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("[", "bracketleft", 1.0),
    KeyDef::new("]", "bracketright", 1.0),
    KeyDef::new("#", "numbersign", 1.5),
];

pub const ISO_ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new(";", "semicolon", 1.0),
    KeyDef::new("'", "apostrophe", 1.0),
    KeyDef::new("#", "backslash", 1.0),
    KeyDef::new("Enter", "Return", 1.25),
];

pub const ISO_ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 1.25),
    KeyDef::new("\\", "backslash", 1.0),
    KeyDef::new("Z", "Z", 1.0),
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new("/", "slash", 1.0),
    KeyDef::new("Shift", "Shift_R", 2.75),
];

pub const ISO_ROW_5: &[KeyDef] = ANSI_ROW_5;

// --- JIS Layout (Japanese) ---

pub const JIS_ROW_1: &[KeyDef] = &[
    KeyDef::new("H/Z", "Zenkaku_Hankaku", 1.0),
    KeyDef::new("1", "1", 1.0),
    KeyDef::new("2", "2", 1.0),
    KeyDef::new("3", "3", 1.0),
    KeyDef::new("4", "4", 1.0),
    KeyDef::new("5", "5", 1.0),
    KeyDef::new("6", "6", 1.0),
    KeyDef::new("7", "7", 1.0),
    KeyDef::new("8", "8", 1.0),
    KeyDef::new("9", "9", 1.0),
    KeyDef::new("0", "0", 1.0),
    KeyDef::new("-", "minus", 1.0),
    KeyDef::new("^", "asciicircum", 1.0),
    KeyDef::new("¥", "yen", 1.0),
    KeyDef::new("BS", "BackSpace", 1.0),
];

pub const JIS_ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Y", "Y", 1.0),
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("@", "at", 1.0),
    KeyDef::new("[", "bracketleft", 1.0),
    KeyDef::new("Enter", "Return", 1.5),
];

pub const JIS_ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new(";", "semicolon", 1.0),
    KeyDef::new(":", "colon", 1.0),
    KeyDef::new("]", "bracketright", 1.0),
    KeyDef::new("", "", 1.25),
];

pub const JIS_ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 2.25),
    KeyDef::new("Z", "Z", 1.0),
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new("/", "slash", 1.0),
    KeyDef::new("\\", "backslash", 1.0),
    KeyDef::new("Shift", "Shift_R", 1.75),
];

pub const JIS_ROW_5: &[KeyDef] = &[
    KeyDef::new("Ctrl", "Control_L", 1.25),
    KeyDef::new("Sup", "Super_L", 1.25),
    KeyDef::new("Alt", "Alt_L", 1.25),
    KeyDef::new("Muhen", "Muhenkan", 1.0),
    KeyDef::new("Space", "space", 4.5),
    KeyDef::new("Henk", "Henkan", 1.0),
    KeyDef::new("Kana", "Hiragana_Katakana", 1.0),
    KeyDef::new("Alt", "Alt_R", 1.25),
    KeyDef::new("App", "Menu", 1.25),
    KeyDef::new("Ctrl", "Control_R", 1.25),
];

// --- ABNT2 Layout (Brazilian) ---

pub const ABNT2_ROW_1: &[KeyDef] = ISO_ROW_1;

pub const ABNT2_ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Y", "Y", 1.0),
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("´ `", "dead_acute", 1.0),
    KeyDef::new("[", "bracketleft", 1.0),
    KeyDef::new("Enter", "Return", 1.5),
];

pub const ABNT2_ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new("Ç", "ccedilla", 1.0),
    KeyDef::new("~", "dead_tilde", 1.0),
    KeyDef::new("]", "bracketright", 1.0),
    KeyDef::new("Enter", "Return", 1.25),
];

pub const ABNT2_ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 1.25),
    KeyDef::new("\\", "backslash", 1.0),
    KeyDef::new("Z", "Z", 1.0),
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new(";", "semicolon", 1.0),
    KeyDef::new("/", "slash", 1.0),
    KeyDef::new("Shift", "Shift_R", 1.75),
];

pub const ABNT2_ROW_5: &[KeyDef] = ANSI_ROW_5;

// --- Hungarian Layout (QWERTZ ISO) ---
// Key differences from standard ISO:
// - Y and Z swapped.
// - Accented vowels in place of brackets/punctuation.
// - 0 is to the left of 1.

pub const HU_ROW_1: &[KeyDef] = &[
    KeyDef::new("0", "0", 1.0), // TLDE is 0
    KeyDef::new("1", "1", 1.0),
    KeyDef::new("2", "2", 1.0),
    KeyDef::new("3", "3", 1.0),
    KeyDef::new("4", "4", 1.0),
    KeyDef::new("5", "5", 1.0),
    KeyDef::new("6", "6", 1.0),
    KeyDef::new("7", "7", 1.0),
    KeyDef::new("8", "8", 1.0),
    KeyDef::new("9", "9", 1.0),
    KeyDef::new("ö", "odiaeresis", 1.0),
    KeyDef::new("ü", "udiaeresis", 1.0),
    KeyDef::new("ó", "oacute", 1.0),
    KeyDef::new("Bksp", "BackSpace", 2.0),
];

pub const HU_ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Z", "Z", 1.0), // Z here
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("ő", "odoubleacute", 1.0),
    KeyDef::new("ú", "uacute", 1.0),
    KeyDef::new("ű", "udoubleacute", 1.5), // Often split, but mimicking ISO enter top... actually standard ISO has # here usually.
                                           // But HU has 'ű' on the BKSL key (ISO #).
];

pub const HU_ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new("é", "eacute", 1.0),
    KeyDef::new("á", "aacute", 1.0),
    KeyDef::new("ű", "udoubleacute", 1.0), // The key next to Return
    KeyDef::new("Enter", "Return", 1.25),
];

pub const HU_ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 1.25),
    KeyDef::new("í", "iacute", 1.0), // LSGT key
    KeyDef::new("Y", "Y", 1.0),      // Y here
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new("-", "minus", 1.0),
    KeyDef::new("Shift", "Shift_R", 2.75),
];

pub const HU_ROW_5: &[KeyDef] = ISO_ROW_5;

#[allow(clippy::type_complexity)]
pub fn get_layout_rows(layout: &str) -> (&[KeyDef], &[KeyDef], &[KeyDef], &[KeyDef], &[KeyDef]) {
    match layout {
        "ISO" => (ISO_ROW_1, ISO_ROW_2, ISO_ROW_3, ISO_ROW_4, ISO_ROW_5),
        "JIS" => (JIS_ROW_1, JIS_ROW_2, JIS_ROW_3, JIS_ROW_4, JIS_ROW_5),
        "ABNT2" => (
            ABNT2_ROW_1,
            ABNT2_ROW_2,
            ABNT2_ROW_3,
            ABNT2_ROW_4,
            ABNT2_ROW_5,
        ),
        "HU" | "HUNGARIAN" => (HU_ROW_1, HU_ROW_2, HU_ROW_3, HU_ROW_4, HU_ROW_5),
        _ => (ANSI_ROW_1, ANSI_ROW_2, ANSI_ROW_3, ANSI_ROW_4, ANSI_ROW_5),
    }
}

pub fn detect_layout(kb_layout: &str) -> &'static str {
    let kb_layout = kb_layout.to_lowercase();
    match kb_layout.as_str() {
        "jp" => "JIS",
        "br" => "ABNT2",
        "hu" => "HU",
        "us" => "ANSI",
        "gb" | "uk" | "de" | "fr" | "it" | "es" | "pt" | "no" | "se" | "fi" | "dk" | "pl" | "cz" => {
            "ISO"
        }
        _ => "ANSI",
    }
}
