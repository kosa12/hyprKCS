use crate::parser::Keybind;
use crate::ui::utils::SearchQuery;
use fuzzy_matcher::FuzzyMatcher;
use glib::subclass::prelude::*;
use gtk::glib;
use gtk4 as gtk;

glib::wrapper! {
    pub struct KeybindObject(ObjectSubclass<imp::KeybindObject>);
}

impl KeybindObject {
    pub fn new(keybind: Keybind, conflict_reason: Option<String>, is_favorite: bool) -> Self {
        let obj: Self = glib::Object::new();

        {
            let imp = obj.imp();
            let mut data = imp.data.borrow_mut();

            data.mods = keybind.mods;
            data.clean_mods = keybind.clean_mods;
            data.key = keybind.key;
            data.dispatcher = keybind.dispatcher;
            data.args = keybind.args;
            data.description = keybind.description.unwrap_or_default();
            data.submap = keybind.submap.unwrap_or_default();
            data.line_number = keybind.line_number as u64;
            data.file_path = keybind.file_path.to_str().unwrap_or("").to_string();
            data.is_favorite = is_favorite;

            if let Some(reason) = conflict_reason {
                data.is_conflicted = true;
                data.conflict_reason = reason;
            } else {
                data.is_conflicted = false;
                data.conflict_reason.clear();
            }
        }

        obj
    }

    pub fn matches_query(
        &self,
        query: &SearchQuery,
        category: u32,
        matcher: &impl FuzzyMatcher,
    ) -> bool {
        let data = self.imp().data.borrow();

        // Category Filter
        let dispatcher_lower = data.dispatcher.to_lowercase();
        let args_lower = data.args.to_lowercase();
        let key_lower = data.key.to_lowercase();

        let category_match = match category {
            0 => true, // All
            1 => {
                dispatcher_lower.contains("workspace")
                    || dispatcher_lower.contains("movetoworkspace")
            }
            2 => {
                dispatcher_lower.contains("window")
                    || dispatcher_lower.contains("active")
                    || dispatcher_lower.contains("focus")
                    || dispatcher_lower.contains("fullscreen")
                    || dispatcher_lower.contains("group")
                    || dispatcher_lower.contains("split")
                    || dispatcher_lower.contains("pin")
            }
            3 => {
                args_lower.contains("volume")
                    || args_lower.contains("brightness")
                    || args_lower.contains("playerctl")
                    || dispatcher_lower.contains("audio")
            }
            4 => dispatcher_lower == "exec", // Custom/Script
            5 => key_lower.contains("mouse"),
            6 => data.is_favorite,
            _ => true,
        };

        if !category_match {
            return false;
        }

        // Advanced Search Filters
        if let Some(ref q_mods) = query.mods {
            if !data.mods.to_lowercase().contains(&q_mods.to_lowercase()) {
                return false;
            }
        }
        if let Some(ref q_key) = query.key {
            if !data.key.to_lowercase().contains(&q_key.to_lowercase()) {
                return false;
            }
        }
        if let Some(ref q_action) = query.action {
            if !dispatcher_lower.contains(&q_action.to_lowercase()) {
                return false;
            }
        }
        if let Some(ref q_args) = query.args {
            if !args_lower.contains(&q_args.to_lowercase()) {
                return false;
            }
        }
        if let Some(ref q_desc) = query.description {
            if !data
                .description
                .to_lowercase()
                .contains(&q_desc.to_lowercase())
            {
                return false;
            }
        }

        if query.general_query.is_empty() {
            return true;
        }

        let text_to_match = &query.general_query;

        matcher.fuzzy_match(&data.mods, text_to_match).is_some()
            || matcher.fuzzy_match(&data.key, text_to_match).is_some()
            || matcher
                .fuzzy_match(&data.dispatcher, text_to_match)
                .is_some()
            || matcher.fuzzy_match(&data.args, text_to_match).is_some()
            || matcher
                .fuzzy_match(&data.description, text_to_match)
                .is_some()
    }
}

mod imp {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk4 as gtk;
    use std::cell::RefCell;

    #[derive(Default, Clone)]
    pub struct KeybindData {
        pub mods: String,
        pub clean_mods: String,
        pub key: String,
        pub dispatcher: String,
        pub args: String,
        pub description: String,
        pub submap: String,
        pub line_number: u64,
        pub file_path: String,
        pub is_conflicted: bool,
        pub conflict_reason: String,
        pub is_favorite: bool,
    }

    #[derive(Default)]
    pub struct KeybindObject {
        pub data: RefCell<KeybindData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeybindObject {
        const NAME: &'static str = "KeybindObject";
        type Type = super::KeybindObject;
    }

    impl ObjectImpl for KeybindObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("mods").build(),
                    glib::ParamSpecString::builder("clean-mods").build(),
                    glib::ParamSpecString::builder("key").build(),
                    glib::ParamSpecString::builder("dispatcher").build(),
                    glib::ParamSpecString::builder("args").build(),
                    glib::ParamSpecString::builder("description").build(),
                    glib::ParamSpecString::builder("submap").build(),
                    glib::ParamSpecUInt64::builder("line-number").build(),
                    glib::ParamSpecString::builder("file-path").build(),
                    glib::ParamSpecBoolean::builder("is-conflicted").build(),
                    glib::ParamSpecString::builder("conflict-reason").build(),
                    glib::ParamSpecBoolean::builder("is-favorite").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let mut data = self.data.borrow_mut();
            match pspec.name() {
                "mods" => data.mods = value.get().unwrap(),
                "clean-mods" => data.clean_mods = value.get().unwrap(),
                "key" => data.key = value.get().unwrap(),
                "dispatcher" => data.dispatcher = value.get().unwrap(),
                "args" => data.args = value.get().unwrap(),
                "description" => data.description = value.get().unwrap(),
                "submap" => data.submap = value.get().unwrap(),
                "line-number" => data.line_number = value.get().unwrap(),
                "file-path" => data.file_path = value.get().unwrap(),
                "is-conflicted" => data.is_conflicted = value.get().unwrap(),
                "conflict-reason" => data.conflict_reason = value.get().unwrap(),
                "is-favorite" => data.is_favorite = value.get().unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let data = self.data.borrow();
            match pspec.name() {
                "mods" => data.mods.to_value(),
                "clean-mods" => data.clean_mods.to_value(),
                "key" => data.key.to_value(),
                "dispatcher" => data.dispatcher.to_value(),
                "args" => data.args.to_value(),
                "description" => data.description.to_value(),
                "submap" => data.submap.to_value(),
                "line-number" => data.line_number.to_value(),
                "file-path" => data.file_path.to_value(),
                "is-conflicted" => data.is_conflicted.to_value(),
                "conflict-reason" => data.conflict_reason.to_value(),
                "is-favorite" => data.is_favorite.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
