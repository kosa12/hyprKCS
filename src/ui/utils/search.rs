use std::borrow::Cow;

pub struct SearchQuery {
    pub mods: Option<Cow<'static, str>>,
    pub key: Option<Cow<'static, str>>,
    pub action: Option<Cow<'static, str>>,
    pub args: Option<Cow<'static, str>>,
    pub description: Option<Cow<'static, str>>,
    pub general_query: Cow<'static, str>,
}

impl SearchQuery {
    pub fn parse(text: &str) -> Self {
        // Fast path for empty query
        if text.is_empty() {
            return SearchQuery {
                mods: None,
                key: None,
                action: None,
                args: None,
                description: None,
                general_query: Cow::Borrowed(""),
            };
        }

        let mut mods = None;
        let mut key = None;
        let mut action = None;
        let mut args = None;
        let mut description = None;
        let mut general_terms: Vec<Cow<'static, str>> = Vec::with_capacity(4);

        for token in text.split_whitespace() {
            let token_lower = token.to_lowercase();
            if let Some((tag, value)) = token_lower.split_once(':') {
                if value.is_empty() {
                    general_terms.push(Cow::Owned(token_lower));
                    continue;
                }
                match tag {
                    "mod" | "mods" => mods = Some(Cow::Owned(value.to_string())),
                    "key" => key = Some(Cow::Owned(value.to_string())),
                    "act" | "action" | "disp" | "dispatcher" => {
                        action = Some(Cow::Owned(value.to_string()))
                    }
                    "arg" | "args" => args = Some(Cow::Owned(value.to_string())),
                    "desc" | "description" => description = Some(Cow::Owned(value.to_string())),
                    _ => general_terms.push(Cow::Owned(token_lower)),
                }
            } else {
                general_terms.push(Cow::Owned(token_lower));
            }
        }

        let general_query = if general_terms.is_empty() {
            Cow::Borrowed("")
        } else {
            Cow::Owned(
                general_terms
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        };

        SearchQuery {
            mods,
            key,
            action,
            args,
            description,
            general_query,
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
        assert_eq!(q.general_query.as_ref(), "exec");
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
        assert_eq!(q.general_query.as_ref(), "just searching stuff");
    }

    #[test]
    fn test_parse_mixed() {
        let q = SearchQuery::parse("Firefox mod:SUPER --private");
        assert_eq!(q.mods.as_deref(), Some("super"));
        assert_eq!(q.general_query.as_ref(), "firefox --private");
    }

    #[test]
    fn test_parse_empty() {
        let q = SearchQuery::parse("");
        assert!(q.mods.is_none());
        assert!(q.key.is_none());
        assert!(q.general_query.is_empty());
    }
}
