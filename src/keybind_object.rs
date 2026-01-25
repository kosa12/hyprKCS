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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        keybind: Keybind,
        conflict_reason: Option<String>,
        broken_reason: Option<String>,
        is_favorite: bool,
        mods_lower: Rc<str>,
        clean_mods_lower: Rc<str>,
        key_lower: Rc<str>,
        dispatcher_lower: Rc<str>,
        args_lower: Option<Rc<str>>,
        description_lower: Option<Rc<str>>,
        flags: Rc<str>,
    ) -> Self {
        let obj: Self = glib::Object::new();

        {
            let imp = obj.imp();
            let mut data = imp.data.borrow_mut();

            data.mods = keybind.mods;
            // Share Rc if mods and clean_mods are the same
            if data.mods.as_ref() == keybind.clean_mods.as_ref() {
                data.clean_mods = data.mods.clone();
            } else {
                data.clean_mods = keybind.clean_mods;
            }

            data.flags = flags;
            data.key = keybind.key;
            data.dispatcher = keybind.dispatcher;

            data.args = if keybind.args.is_empty() {
                None
            } else {
                Some(keybind.args)
            };
            data.description = keybind.description.filter(|d| !d.is_empty());
            data.submap = keybind.submap.filter(|s| !s.is_empty());

            data.line_number = keybind.line_number as u64;
            data.file_path = keybind.file_path.to_str().unwrap_or("").into();
            data.is_favorite = is_favorite;

            data.mods_lower = mods_lower;
            data.clean_mods_lower = clean_mods_lower;
            data.key_lower = key_lower;
            data.dispatcher_lower = dispatcher_lower;
            data.args_lower = args_lower;
            data.description_lower = description_lower;

            if let Some(reason) = conflict_reason {
                data.is_conflicted = true;
                data.conflict_reason = Some(reason.into());
            } else {
                data.is_conflicted = false;
                data.conflict_reason = None;
            }

            if let Some(reason) = broken_reason {
                data.is_broken = true;
                data.broken_reason = Some(reason.into());
            } else {
                data.is_broken = false;
                data.broken_reason = None;
            }
        }

        obj
    }

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

        let dispatcher_lower = &data.dispatcher_lower;
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
                data.args_lower.as_ref().is_some_and(|a| {
                    a.contains("volume") || a.contains("brightness") || a.contains("playerctl")
                }) || dispatcher_lower.contains("audio")
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
            // Match against both raw and clean mods for user convenience
            if !data.mods_lower.contains(q_mods) && !data.clean_mods_lower.contains(q_mods) {
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
            if !data.args_lower.as_ref().is_some_and(|a| a.contains(q_args)) {
                return false;
            }
        }
        if let Some(ref q_desc) = query.description {
            if !data
                .description_lower
                .as_ref()
                .is_some_and(|d| d.contains(q_desc))
            {
                return false;
            }
        }

        if query.general_query.is_empty() {
            return true;
        }

        let text_to_match: &str = query.general_query.as_ref();

        if data.mods_lower.contains(text_to_match)
            || data.clean_mods_lower.contains(text_to_match)
            || data.key_lower.contains(text_to_match)
            || data.dispatcher_lower.contains(text_to_match)
            || data
                .args_lower
                .as_ref()
                .is_some_and(|a| a.contains(text_to_match))
            || data
                .description_lower
                .as_ref()
                .is_some_and(|d| d.contains(text_to_match))
        {
            return true;
        }

        matcher
            .fuzzy_match(&data.mods_lower, text_to_match)
            .is_some()
            || matcher
                .fuzzy_match(&data.clean_mods_lower, text_to_match)
                .is_some()
            || matcher
                .fuzzy_match(&data.key_lower, text_to_match)
                .is_some()
            || matcher
                .fuzzy_match(&data.dispatcher_lower, text_to_match)
                .is_some()
            || data
                .args_lower
                .as_ref()
                .is_some_and(|a| matcher.fuzzy_match(a, text_to_match).is_some())
            || data
                .description_lower
                .as_ref()
                .is_some_and(|d| matcher.fuzzy_match(d, text_to_match).is_some())
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
        pub flags: Rc<str>,
        pub key: Rc<str>,
        pub dispatcher: Rc<str>,
        pub args: Option<Rc<str>>,
        pub description: Option<Rc<str>>,
        pub submap: Option<Rc<str>>,
        pub line_number: u64,
        pub file_path: Rc<str>,
        pub is_conflicted: bool,
        pub conflict_reason: Option<Rc<str>>,
        pub is_favorite: bool,
        pub is_broken: bool,
        pub broken_reason: Option<Rc<str>>,

        pub mods_lower: Rc<str>,
        pub clean_mods_lower: Rc<str>,
        pub key_lower: Rc<str>,
        pub dispatcher_lower: Rc<str>,
        pub args_lower: Option<Rc<str>>,
        pub description_lower: Option<Rc<str>>,
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

    fn to_lower_rc(s: &str) -> Rc<str> {
        let lower = s.to_lowercase();
        if lower == s {
            Rc::from(s)
        } else {
            Rc::from(lower)
        }
    }

    impl ObjectImpl for KeybindObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::LazyLock;
            static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
                vec![
                    glib::ParamSpecString::builder("mods").build(),
                    glib::ParamSpecString::builder("clean-mods").build(),
                    glib::ParamSpecString::builder("flags").build(),
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
                    glib::ParamSpecBoolean::builder("is-broken").build(),
                    glib::ParamSpecString::builder("broken-reason").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let mut data = self.data.borrow_mut();

            match pspec.name() {
                "mods" => {
                    let v: String = value.get().unwrap();
                    data.mods_lower = to_lower_rc(&v);
                    data.mods = v.into();
                }
                "clean-mods" => {
                    let v: String = value.get().unwrap();
                    data.clean_mods_lower = to_lower_rc(&v);
                    data.clean_mods = v.into();
                }
                "flags" => {
                    let v: String = value.get().unwrap();
                    data.flags = v.into();
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
                    data.args_lower = if v.is_empty() {
                        None
                    } else {
                        Some(to_lower_rc(&v))
                    };
                    data.args = if v.is_empty() { None } else { Some(v.into()) };
                }
                "description" => {
                    let v: String = value.get().unwrap();
                    data.description_lower = if v.is_empty() {
                        None
                    } else {
                        Some(to_lower_rc(&v))
                    };
                    data.description = if v.is_empty() { None } else { Some(v.into()) };
                }
                "submap" => {
                    let v: String = value.get().unwrap();
                    data.submap = if v.is_empty() { None } else { Some(v.into()) };
                }
                "line-number" => data.line_number = value.get().unwrap(),
                "file-path" => {
                    let v: String = value.get().unwrap();
                    data.file_path = v.into();
                }
                "is-conflicted" => data.is_conflicted = value.get().unwrap(),
                "conflict-reason" => {
                    let v: String = value.get().unwrap();
                    data.conflict_reason = if v.is_empty() { None } else { Some(v.into()) };
                }
                "is-favorite" => data.is_favorite = value.get().unwrap(),
                "is-broken" => data.is_broken = value.get().unwrap(),
                "broken-reason" => {
                    let v: String = value.get().unwrap();
                    data.broken_reason = if v.is_empty() { None } else { Some(v.into()) };
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let data = self.data.borrow();
            match pspec.name() {
                "mods" => data.mods.as_ref().to_value(),
                "clean-mods" => data.clean_mods.as_ref().to_value(),
                "flags" => data.flags.as_ref().to_value(),
                "key" => data.key.as_ref().to_value(),
                "dispatcher" => data.dispatcher.as_ref().to_value(),
                "args" => data.args.as_ref().map_or("", |s| s.as_ref()).to_value(),
                "description" => data
                    .description
                    .as_ref()
                    .map_or("", |s| s.as_ref())
                    .to_value(),
                "submap" => data.submap.as_ref().map_or("", |s| s.as_ref()).to_value(),
                "line-number" => data.line_number.to_value(),
                "file-path" => data.file_path.as_ref().to_value(),
                "is-conflicted" => data.is_conflicted.to_value(),
                "conflict-reason" => data
                    .conflict_reason
                    .as_ref()
                    .map_or("", |s| s.as_ref())
                    .to_value(),
                "is-favorite" => data.is_favorite.to_value(),
                "is-broken" => data.is_broken.to_value(),
                "broken-reason" => data
                    .broken_reason
                    .as_ref()
                    .map_or("", |s| s.as_ref())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
