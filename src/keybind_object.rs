use gtk4 as gtk;
use gtk::glib;
use glib::Object;
use crate::parser::Keybind;

glib::wrapper! {
    pub struct KeybindObject(ObjectSubclass<imp::KeybindObject>);
}

impl KeybindObject {
    pub fn new(keybind: Keybind, is_conflicted: bool) -> Self {
        Object::builder()
            .property("mods", keybind.mods)
            .property("clean-mods", keybind.clean_mods)
            .property("key", keybind.key)
            .property("dispatcher", keybind.dispatcher)
            .property("args", keybind.args)
            .property("line-number", keybind.line_number as u64)
            .property("file-path", keybind.file_path.to_str().unwrap_or(""))
            .property("is-conflicted", is_conflicted)
            .build()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};
    use gtk4 as gtk;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct KeybindObject {
        pub mods: RefCell<String>,
        pub clean_mods: RefCell<String>,
        pub key: RefCell<String>,
        pub dispatcher: RefCell<String>,
        pub args: RefCell<String>,
        pub line_number: Cell<u64>,
        pub file_path: RefCell<String>,
        pub is_conflicted: Cell<bool>,
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
                    glib::ParamSpecUInt64::builder("line-number").build(),
                    glib::ParamSpecString::builder("file-path").build(),
                    glib::ParamSpecBoolean::builder("is-conflicted").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "mods" => { self.mods.replace(value.get().unwrap()); },
                "clean-mods" => { self.clean_mods.replace(value.get().unwrap()); },
                "key" => { self.key.replace(value.get().unwrap()); },
                "dispatcher" => { self.dispatcher.replace(value.get().unwrap()); },
                "args" => { self.args.replace(value.get().unwrap()); },
                "line-number" => { self.line_number.replace(value.get().unwrap()); },
                "file-path" => { self.file_path.replace(value.get().unwrap()); },
                "is-conflicted" => { self.is_conflicted.replace(value.get().unwrap()); },
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "mods" => self.mods.borrow().clone().to_value(),
                "clean-mods" => self.clean_mods.borrow().clone().to_value(),
                "key" => self.key.borrow().clone().to_value(),
                "dispatcher" => self.dispatcher.borrow().clone().to_value(),
                "args" => self.args.borrow().clone().to_value(),
                "line-number" => self.line_number.get().to_value(),
                "file-path" => self.file_path.borrow().clone().to_value(),
                "is-conflicted" => self.is_conflicted.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
