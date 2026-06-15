use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Timeframe;
use glib::clone;

use crate::utils::TemplateCallbacks;

mod date_time_picker_group;

use self::date_time_picker_group::DateTimePickerGroup;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/timeframe_picker.ui")]
    #[properties(wrapper_type = super::TimeframePicker)]
    pub struct TimeframePicker {
        #[property(get, set = Self::set_timeframe)]
        timeframe: RefCell<Option<Timeframe>>,
        #[template_child]
        schedule_type: TemplateChild<adw::ToggleGroup>,
        #[template_child]
        start: TemplateChild<DateTimePickerGroup>,
        #[template_child]
        end: TemplateChild<DateTimePickerGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeframePicker {
        const NAME: &'static str = "TimeframePicker";
        type Type = super::TimeframePicker;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TimeframePicker {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_timeframe();

            self.schedule_type.connect_active_name_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_timeframe();
                }
            ));

            self.start.connect_date_time_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_timeframe();
                }
            ));
            self.end.connect_date_time_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    imp.update_timeframe();
                }
            ));
        }
    }

    impl WidgetImpl for TimeframePicker {}
    impl PreferencesGroupImpl for TimeframePicker {}

    #[gtk::template_callbacks]
    impl TimeframePicker {
        fn set_timeframe(&self, timeframe: &Timeframe) {
            if Some(timeframe) == self.timeframe.borrow().as_ref() {
                return;
            }

            self.timeframe.replace(Some(timeframe.clone()));

            self.schedule_type
                .set_active_name(Some(if timeframe.is_all_day() {
                    "all-day"
                } else {
                    "time-slot"
                }));
            self.start.set_date_only(timeframe.is_all_day());
            self.end.set_date_only(timeframe.is_all_day());
            self.start.set_date_time(timeframe.start().unwrap());
            self.end.set_date_time(timeframe.end().unwrap());

            self.obj().notify_timeframe();
        }

        fn update_timeframe(&self) {
            let all_day = self.schedule_type.active_name().as_deref() == Some("all-day");
            let start_date_time = self.start.date_time();
            let end_date_time = self.end.date_time();
            // if (valid)
            self.timeframe.replace(Some(Timeframe::new(
                all_day,
                &start_date_time,
                &end_date_time,
            )));
            self.obj().notify_timeframe();
        }
    }
}

glib::wrapper! {
    pub struct TimeframePicker(ObjectSubclass<imp::TimeframePicker>)
        @extends gtk::Widget, adw::PreferencesGroup,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
