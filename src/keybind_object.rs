use crate::parser::Keybind;
use glib::subclass::prelude::*;
use gtk::glib;
use gtk4 as gtk;

glib::wrapper! {
    pub struct KeybindObject(ObjectSubclass<imp::KeybindObject>);
}

impl KeybindObject {
    pub fn new(keybind: Keybind, conflict_reason: Option<String>) -> Self {
        let obj: Self = glib::Object::new();

        {
            let imp = obj.imp();
            let mut data = imp.data.borrow_mut();

            data.mods = keybind.mods;
            data.clean_mods = keybind.clean_mods;
            data.key = keybind.key;
            data.dispatcher = keybind.dispatcher;
            data.args = keybind.args;
            data.submap = keybind.submap.unwrap_or_default();
            data.line_number = keybind.line_number as u64;
            data.file_path = keybind.file_path.to_str().unwrap_or("").to_string();

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
        pub submap: String,
        pub line_number: u64,
        pub file_path: String,
        pub is_conflicted: bool,
        pub conflict_reason: String,
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
                    glib::ParamSpecString::builder("submap").build(),
                    glib::ParamSpecUInt64::builder("line-number").build(),
                    glib::ParamSpecString::builder("file-path").build(),
                    glib::ParamSpecBoolean::builder("is-conflicted").build(),
                    glib::ParamSpecString::builder("conflict-reason").build(),
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
                "submap" => data.submap = value.get().unwrap(),
                "line-number" => data.line_number = value.get().unwrap(),
                "file-path" => data.file_path = value.get().unwrap(),
                "is-conflicted" => data.is_conflicted = value.get().unwrap(),
                "conflict-reason" => data.conflict_reason = value.get().unwrap(),
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
                "submap" => data.submap.to_value(),
                "line-number" => data.line_number.to_value(),
                "file-path" => data.file_path.to_value(),
                "is-conflicted" => data.is_conflicted.to_value(),
                "conflict-reason" => data.conflict_reason.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
