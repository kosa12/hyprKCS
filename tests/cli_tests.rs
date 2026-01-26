use hyprKCS::cli::Args;
use std::path::PathBuf;

#[test]
fn test_cli_parsing_basic() {
    let args = vec!["hyprkcs", "-p"];
    let parsed = Args::parse_from(args);
    assert!(parsed.print);
    assert!(!parsed.doctor);
    assert!(parsed.config.is_none());
    assert!(parsed.search.is_none());
}

#[test]
fn test_cli_parsing_full() {
    let args = vec!["hyprkcs", "-c", "/path/to/config", "-s", "my search", "--doctor"];
    let parsed = Args::parse_from(args);
    assert!(!parsed.print); // -p was not provided
    assert!(parsed.doctor);
    assert_eq!(parsed.config, Some(PathBuf::from("/path/to/config")));
    assert_eq!(parsed.search.as_deref(), Some("my search"));
}

#[test]
fn test_cli_parsing_long_flags() {
    let args = vec!["hyprkcs", "--config", "/alt/path", "--print", "--search", "term"];
    let parsed = Args::parse_from(args);
    assert!(parsed.print);
    assert!(!parsed.doctor);
    assert_eq!(parsed.config, Some(PathBuf::from("/alt/path")));
    assert_eq!(parsed.search.as_deref(), Some("term"));
}

#[test]
fn test_cli_parsing_empty() {
    let args = vec!["hyprkcs"];
    let parsed = Args::parse_from(args);
    assert!(!parsed.print);
    assert!(!parsed.doctor);
    assert!(parsed.config.is_none());
    assert!(parsed.search.is_none());
}
