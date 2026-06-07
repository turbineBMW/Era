use std::sync::LazyLock;

use adw::{prelude::*, subclass::prelude::*};
use gio::ListStore;
use glib::{GString, TimeZone, closure_local, subclass::Signal};
use gtk::{Entry, FilterListModel};

use crate::utils::TemplateCallbacks;

use super::object_time_zone::ObjectTimeZone;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/time_zone_picker_dialog.ui")]
    #[properties(wrapper_type = super::TimeZonePickerDialog)]
    pub struct TimeZonePickerDialog {
        #[template_child]
        input: TemplateChild<Entry>,
        #[template_child]
        filter: TemplateChild<FilterListModel>,
        #[template_child]
        time_zones: TemplateChild<ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeZonePickerDialog {
        const NAME: &'static str = "TimeZonePickerDialog";
        type Type = super::TimeZonePickerDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            ObjectTimeZone::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("time-zone-picker-dialog");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TimeZonePickerDialog {
        fn constructed(&self) {
            // TODO: Create the model once at app start to reduce the time the dialog takes to
            // display
            for tzid in jiff::tz::db().available() {
                let Some(time_zone) = TimeZone::from_identifier(Some(tzid.as_str())) else {
                    continue;
                };
                let object_time_zone = ObjectTimeZone::new(time_zone);
                self.time_zones.append(&object_time_zone);
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("time-zone-picked")
                        .param_types([GString::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for TimeZonePickerDialog {}
    impl AdwDialogImpl for TimeZonePickerDialog {}

    #[gtk::template_callbacks]
    impl TimeZonePickerDialog {
        #[template_callback]
        fn select_time_zone(&self, position: u32) {
            let tzid = self
                .filter
                .item(position)
                .expect("Item should exist at position")
                .downcast::<ObjectTimeZone>()
                .expect("Object should be a ObjectTimeZone")
                .time_zone()
                .identifier();
            self.obj().emit_by_name::<()>("time-zone-picked", &[&tzid]);
            self.obj().close();
        }
    }
}

glib::wrapper! {
    pub struct TimeZonePickerDialog(ObjectSubclass<imp::TimeZonePickerDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl TimeZonePickerDialog {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Connect to the signal emitted when a time zone is picked.
    pub fn connect_time_zone_picked<F: Fn(&Self, GString) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "time-zone-picked",
            true,
            closure_local!(move |obj: Self, time_zone_id: GString| {
                f(&obj, time_zone_id);
            }),
        )
    }
}
