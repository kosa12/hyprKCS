use hyprKCS::ui::utils::search::SearchQuery;

#[test]
fn test_parse_simple() {
    let q = SearchQuery::parse("mod:SUPER key:Q exec");
    assert_eq!(q.mods.as_deref(), Some("super"));
    assert_eq!(q.key.as_deref(), Some("q"));
    assert_eq!(q.general_query, "exec");
}

#[test]
fn test_parse_aliases() {
    let q = SearchQuery::parse("mods:SHIFT action:kill disp:ignore arg:something");
    assert_eq!(q.mods.as_deref(), Some("shift"));
    assert_eq!(q.action.as_deref(), Some("ignore"));
    assert_eq!(q.args.as_deref(), Some("something"));
}

#[test]
fn test_parse_general_only() {
    let q = SearchQuery::parse("JUST SEARCHING STUFF");
    assert_eq!(q.mods, None);
    assert_eq!(q.general_query, "just searching stuff");
}

#[test]
fn test_parse_mixed() {
    let q = SearchQuery::parse("Firefox mod:SUPER --private");
    assert_eq!(q.mods.as_deref(), Some("super"));
    assert_eq!(q.general_query, "firefox --private");
}

#[test]
fn test_parse_empty() {
    let q = SearchQuery::parse("");
    assert!(q.mods.is_none());
    assert!(q.key.is_none());
    assert!(q.general_query.is_empty());
}

#[test]
fn test_parse_multiple_same_tag() {
    // Last one wins currently
    let q = SearchQuery::parse("mod:SUPER mod:SHIFT");
    assert_eq!(q.mods.as_deref(), Some("shift"));
}

#[test]
fn test_parse_empty_value_tag() {
    let q = SearchQuery::parse("mod: key:Q");
    assert_eq!(q.mods, None); // "mod:" should be treated as general if empty
    assert_eq!(q.key.as_deref(), Some("q"));
    assert_eq!(q.general_query, "mod:");
}

#[test]
fn test_parse_colon_in_general_query() {
    let q = SearchQuery::parse("unknown:tag mod:SUPER");
    assert_eq!(q.mods.as_deref(), Some("super"));
    assert_eq!(q.general_query, "unknown:tag");
}
