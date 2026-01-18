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
                    general_terms.push(token.to_string());
                    continue;
                }
                match tag.to_lowercase().as_str() {
                    "mod" | "mods" => mods = Some(value.to_string()),
                    "key" => key = Some(value.to_string()),
                    "act" | "action" | "disp" | "dispatcher" => action = Some(value.to_string()),
                    "arg" | "args" => args = Some(value.to_string()),
                    "desc" | "description" => description = Some(value.to_string()),
                    _ => general_terms.push(token.to_string()),
                }
            } else {
                general_terms.push(token.to_string());
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
        assert_eq!(q.mods, Some("SUPER".to_string()));
        assert_eq!(q.key, Some("Q".to_string()));
        assert_eq!(q.general_query, "exec");
    }

    #[test]
    fn test_parse_aliases() {
        let q = SearchQuery::parse("mods:SHIFT action:kill disp:ignore arg:something");
        assert_eq!(q.mods, Some("SHIFT".to_string()));
        // Last one wins if duplicates, but here we just check if they are picked up.
        // My implementation overwrites.
        assert_eq!(q.action, Some("ignore".to_string())); 
        assert_eq!(q.args, Some("something".to_string()));
    }

    #[test]
    fn test_parse_general_only() {
        let q = SearchQuery::parse("just searching stuff");
        assert_eq!(q.mods, None);
        assert_eq!(q.general_query, "just searching stuff");
    }

    #[test]
    fn test_parse_mixed() {
        let q = SearchQuery::parse("firefox mod:SUPER --private");
        assert_eq!(q.mods, Some("SUPER".to_string()));
        assert_eq!(q.general_query, "firefox --private");
    }
}
