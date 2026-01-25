pub struct SearchQuery {
    pub mods: Option<String>,
    pub key: Option<String>,
    pub action: Option<String>,
    pub args: Option<String>,
    pub description: Option<String>,
    pub general_query: String,
}

impl SearchQuery {
    pub fn parse(text: &str) -> Self {
        let mut mods = None;
        let mut key = None;
        let mut action = None;
        let mut args = None;
        let mut description = None;
        let mut general_terms = Vec::new();

        for token in text.split_whitespace() {
            let token_lower = token.to_lowercase();
            if let Some((tag, value)) = token_lower.split_once(':') {
                if value.is_empty() {
                    general_terms.push(token_lower);
                    continue;
                }
                match tag {
                    "mod" | "mods" => mods = Some(value.to_string()),
                    "key" => key = Some(value.to_string()),
                    "act" | "action" | "disp" | "dispatcher" => action = Some(value.to_string()),
                    "arg" | "args" => args = Some(value.to_string()),
                    "desc" | "description" => description = Some(value.to_string()),
                    _ => general_terms.push(token_lower),
                }
            } else {
                general_terms.push(token_lower);
            }
        }

        SearchQuery {
            mods,
            key,
            action,
            args,
            description,
            general_query: general_terms.join(" "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
