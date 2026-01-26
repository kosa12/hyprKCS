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
