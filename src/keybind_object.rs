use crate::parser::Keybind;
use crate::ui::utils::SearchQuery;
use fuzzy_matcher::FuzzyMatcher;
use glib::subclass::prelude::*;
use gtk::glib;
use gtk4 as gtk;
use std::rc::Rc;

glib::wrapper! {
    pub struct KeybindObject(ObjectSubclass<imp::KeybindObject>);
}

impl KeybindObject {
    pub fn new(keybind: Keybind, conflict_reason: Option<String>, is_favorite: bool) -> Self {
        let obj: Self = glib::Object::new();

        {
            let imp = obj.imp();
            let mut data = imp.data.borrow_mut();

            // Helper to get lowercased Rc<str> efficiently
            fn to_lower_rc(s: &str) -> Rc<str> {
                let lower = s.to_lowercase();
                if lower == s {
                    Rc::from(s)
                } else {
                    Rc::from(lower)
                }
            }

            data.mods = keybind.mods;
            data.clean_mods = keybind.clean_mods;
            data.key = keybind.key;
            data.dispatcher = keybind.dispatcher;
            data.args = keybind.args;

            let desc: Rc<str> = keybind.description.unwrap_or_else(|| "".into());
            data.description = desc.clone();
            data.submap = keybind.submap.unwrap_or_else(|| "".into());
            data.line_number = keybind.line_number as u64;
            data.file_path = keybind.file_path.to_str().unwrap_or("").into();
            data.is_favorite = is_favorite;

            // Pre-calculate lowercased versions for faster searching, reusing Rc if already lowercase
            data.mods_lower = to_lower_rc(&data.mods);
            data.key_lower = to_lower_rc(&data.key);
            data.dispatcher_lower = to_lower_rc(&data.dispatcher);
            data.args_lower = to_lower_rc(&data.args);
            data.description_lower = to_lower_rc(&data.description);

            if let Some(reason) = conflict_reason {
                data.is_conflicted = true;
                data.conflict_reason = reason.into();
            } else {
                data.is_conflicted = false;
                data.conflict_reason = "".into();
            }
        }

        obj
    }

    /// Access internal data efficiently without going through GObject property system
    pub fn with_data<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&imp::KeybindData) -> R,
    {
        let data = self.imp().data.borrow();
        f(&data)
    }

    pub fn matches_query(
        &self,
        query: &SearchQuery,
        category: u32,
        matcher: &impl FuzzyMatcher,
    ) -> bool {
        let data = self.imp().data.borrow();

        // Category Filter - using cached lowercased strings
        let dispatcher_lower = &data.dispatcher_lower;
        let args_lower = &data.args_lower;
        let key_lower = &data.key_lower;

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
            4 => dispatcher_lower.as_ref() == "exec", // Custom/Script
            5 => key_lower.contains("mouse"),
            6 => data.is_favorite,
            _ => true,
        };

        if !category_match {
            return false;
        }

        // Advanced Search Filters - Query parts are already lowercased in SearchQuery::parse
        if let Some(ref q_mods) = query.mods {
            if !data.mods_lower.contains(q_mods) {
                return false;
            }
        }
        if let Some(ref q_key) = query.key {
            if !data.key_lower.contains(q_key) {
                return false;
            }
        }
        if let Some(ref q_action) = query.action {
            if !data.dispatcher_lower.contains(q_action) {
                return false;
            }
        }
        if let Some(ref q_args) = query.args {
            if !data.args_lower.contains(q_args) {
                return false;
            }
        }
        if let Some(ref q_desc) = query.description {
            if !data.description_lower.contains(q_desc) {
                return false;
            }
        }

        if query.general_query.is_empty() {
            return true;
        }

        let text_to_match = &query.general_query;

        matcher
            .fuzzy_match(&data.mods_lower, text_to_match)
            .is_some()
            || matcher
                .fuzzy_match(&data.key_lower, text_to_match)
                .is_some()
            || matcher
                .fuzzy_match(&data.dispatcher_lower, text_to_match)
                .is_some()
            || matcher
                .fuzzy_match(&data.args_lower, text_to_match)
                .is_some()
            || matcher
                .fuzzy_match(&data.description_lower, text_to_match)
                .is_some()
    }
}

pub mod imp {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk4 as gtk;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default, Clone)]
    pub struct KeybindData {
        pub mods: Rc<str>,
        pub clean_mods: Rc<str>,
        pub key: Rc<str>,
        pub dispatcher: Rc<str>,
        pub args: Rc<str>,
        pub description: Rc<str>,
        pub submap: Rc<str>,
        pub line_number: u64,
        pub file_path: Rc<str>,
        pub is_conflicted: bool,
        pub conflict_reason: Rc<str>,
        pub is_favorite: bool,

        // Cached lowercase fields for search optimization
        pub mods_lower: Rc<str>,
        pub key_lower: Rc<str>,
        pub dispatcher_lower: Rc<str>,
        pub args_lower: Rc<str>,
        pub description_lower: Rc<str>,
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
            use std::sync::LazyLock;
            static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
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

            // Helper to get lowercased Rc<str> efficiently
            fn to_lower_rc(s: &str) -> Rc<str> {
                let lower = s.to_lowercase();
                if lower == s {
                    Rc::from(s)
                } else {
                    Rc::from(lower)
                }
            }

            match pspec.name() {
                "mods" => {
                    let v: String = value.get().unwrap();
                    data.mods_lower = to_lower_rc(&v);
                    data.mods = v.into();
                }
                "clean-mods" => {
                    let v: String = value.get().unwrap();
                    data.clean_mods = v.into();
                }
                "key" => {
                    let v: String = value.get().unwrap();
                    data.key_lower = to_lower_rc(&v);
                    data.key = v.into();
                }
                "dispatcher" => {
                    let v: String = value.get().unwrap();
                    data.dispatcher_lower = to_lower_rc(&v);
                    data.dispatcher = v.into();
                }
                "args" => {
                    let v: String = value.get().unwrap();
                    data.args_lower = to_lower_rc(&v);
                    data.args = v.into();
                }
                "description" => {
                    let v: String = value.get().unwrap();
                    data.description_lower = to_lower_rc(&v);
                    data.description = v.into();
                }
                "submap" => {
                    let v: String = value.get().unwrap();
                    data.submap = v.into();
                }
                "line-number" => data.line_number = value.get().unwrap(),
                "file-path" => {
                    let v: String = value.get().unwrap();
                    data.file_path = v.into();
                }
                "is-conflicted" => data.is_conflicted = value.get().unwrap(),
                "conflict-reason" => {
                    let v: String = value.get().unwrap();
                    data.conflict_reason = v.into();
                }
                "is-favorite" => data.is_favorite = value.get().unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let data = self.data.borrow();
            match pspec.name() {
                "mods" => data.mods.as_ref().to_value(),
                "clean-mods" => data.clean_mods.as_ref().to_value(),
                "key" => data.key.as_ref().to_value(),
                "dispatcher" => data.dispatcher.as_ref().to_value(),
                "args" => data.args.as_ref().to_value(),
                "description" => data.description.as_ref().to_value(),
                "submap" => data.submap.as_ref().to_value(),
                "line-number" => data.line_number.to_value(),
                "file-path" => data.file_path.as_ref().to_value(),
                "is-conflicted" => data.is_conflicted.to_value(),
                "conflict-reason" => data.conflict_reason.as_ref().to_value(),
                "is-favorite" => data.is_favorite.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
