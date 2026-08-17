use gtk::{glib, subclass::prelude::*};

use super::{System, SystemImpl};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AndroidSystem {}

    #[glib::object_subclass]
    impl ObjectSubclass for AndroidSystem {
        const NAME: &'static str = "EraAndroidSystem";
        type Type = super::AndroidSystem;
        type ParentType = System;
    }

    impl ObjectImpl for AndroidSystem {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Implement reading and observing system on Android.
        }
    }

    impl SystemImpl for AndroidSystem {}
}

glib::wrapper! {
    /// API to access system state on Android.
    pub struct AndroidSystem(ObjectSubclass<imp::AndroidSystem>)
        @extends System;
}

impl AndroidSystem {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for AndroidSystem {
    fn default() -> Self {
        Self::new()
    }
}
