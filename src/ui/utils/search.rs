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
            if let Some((tag, value)) = token.split_once(':') {
                if value.is_empty() {
                    general_terms.push(token.to_lowercase());
                    continue;
                }
                match tag.to_lowercase().as_str() {
                    "mod" | "mods" => mods = Some(value.to_lowercase()),
                    "key" => key = Some(value.to_lowercase()),
                    "act" | "action" | "disp" | "dispatcher" => action = Some(value.to_lowercase()),
                    "arg" | "args" => args = Some(value.to_lowercase()),
                    "desc" | "description" => description = Some(value.to_lowercase()),
                    _ => general_terms.push(token.to_lowercase()),
                }
            } else {
                general_terms.push(token.to_lowercase());
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
        assert_eq!(q.mods, Some("super".to_string()));
        assert_eq!(q.key, Some("q".to_string()));
        assert_eq!(q.general_query, "exec");
    }

    #[test]
    fn test_parse_aliases() {
        let q = SearchQuery::parse("mods:SHIFT action:kill disp:ignore arg:something");
        assert_eq!(q.mods, Some("shift".to_string()));
        assert_eq!(q.action, Some("ignore".to_string()));
        assert_eq!(q.args, Some("something".to_string()));
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
        assert_eq!(q.mods, Some("super".to_string()));
        assert_eq!(q.general_query, "firefox --private");
    }
}
