use hyprKCS::ui::views::keyboard_layouts::detect_layout;

#[test]
fn test_detect_layout_simple() {
    assert_eq!(detect_layout("us"), "ANSI");
    assert_eq!(detect_layout("jp"), "JIS");
    assert_eq!(detect_layout("br"), "ABNT2");
    assert_eq!(detect_layout("hu"), "HU");
}

#[test]
fn test_detect_layout_iso() {
    assert_eq!(detect_layout("gb"), "ISO");
    assert_eq!(detect_layout("uk"), "ISO");
    assert_eq!(detect_layout("de"), "ISO");
    assert_eq!(detect_layout("fr"), "ISO");
}

#[test]
fn test_detect_layout_comma_separated() {
    assert_eq!(detect_layout("us,ru"), "ANSI");
    assert_eq!(detect_layout("gb,us"), "ISO");
    assert_eq!(detect_layout("jp,us"), "JIS");
}

#[test]
fn test_detect_layout_whitespace() {
    assert_eq!(detect_layout(" us "), "ANSI");
    assert_eq!(detect_layout("  gb , us  "), "ISO");
}

#[test]
fn test_detect_layout_case_insensitive() {
    assert_eq!(detect_layout("US"), "ANSI");
    assert_eq!(detect_layout("JP"), "JIS");
    assert_eq!(detect_layout("Gb"), "ISO");
}

#[test]
fn test_detect_layout_unknown() {
    assert_eq!(detect_layout("unknown"), "ANSI");
    assert_eq!(detect_layout(""), "ANSI");
}
