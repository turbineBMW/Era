use gtk::{glib, subclass::prelude::*};

use super::{SystemSettings, SystemSettingsImpl};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AndroidSystemSettings {}

    #[glib::object_subclass]
    impl ObjectSubclass for AndroidSystemSettings {
        const NAME: &'static str = "AndroidSystemSettings";
        type Type = super::AndroidSystemSettings;
        type ParentType = SystemSettings;
    }

    impl ObjectImpl for AndroidSystemSettings {
        fn constructed(&self) {
            self.parent_constructed();

            // TODO: Implement reading and observing system settings on Android.
        }
    }

    impl SystemSettingsImpl for AndroidSystemSettings {}
}

glib::wrapper! {
    /// API to access system settings on Android.
    pub struct AndroidSystemSettings(ObjectSubclass<imp::AndroidSystemSettings>)
        @extends SystemSettings;
}

impl AndroidSystemSettings {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for AndroidSystemSettings {
    fn default() -> Self {
        Self::new()
    }
}
