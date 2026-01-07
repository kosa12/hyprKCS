use gtk4 as gtk;
use gtk::glib;
use glib::Object;
use crate::parser::Keybind;

glib::wrapper! {
    pub struct KeybindObject(ObjectSubclass<imp::KeybindObject>);
}

impl KeybindObject {
    pub fn new(keybind: Keybind) -> Self {
        Object::builder()
            .property("mods", keybind.mods)
            .property("key", keybind.key)
            .property("dispatcher", keybind.dispatcher)
            .property("args", keybind.args)
            .property("line-number", keybind.line_number as u64)
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
        pub key: RefCell<String>,
        pub dispatcher: RefCell<String>,
        pub args: RefCell<String>,
        pub line_number: Cell<u64>,
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
                    glib::ParamSpecString::builder("key").build(),
                    glib::ParamSpecString::builder("dispatcher").build(),
                    glib::ParamSpecString::builder("args").build(),
                    glib::ParamSpecUInt64::builder("line-number").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "mods" => { self.mods.replace(value.get().unwrap()); },
                "key" => { self.key.replace(value.get().unwrap()); },
                "dispatcher" => { self.dispatcher.replace(value.get().unwrap()); },
                "args" => { self.args.replace(value.get().unwrap()); },
                "line-number" => { self.line_number.replace(value.get().unwrap()); },
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "mods" => self.mods.borrow().clone().to_value(),
                "key" => self.key.borrow().clone().to_value(),
                "dispatcher" => self.dispatcher.borrow().clone().to_value(),
                "args" => self.args.borrow().clone().to_value(),
                "line-number" => self.line_number.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
