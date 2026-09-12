use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::Timeframe;
use glib::{DateTime, clone};

use crate::utils::TemplateCallbacks;

mod date_time_picker_group;

use self::date_time_picker_group::DateTimePickerGroup;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/components/timeframe_picker/timeframe_picker.blp")]
    #[properties(wrapper_type = super::TimeframePicker)]
    pub struct TimeframePicker {
        #[property(get, nullable)]
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
                    let new_end = if let Some(timeframe) = imp.timeframe.borrow().clone() {
                        let old_start = timeframe.start().unwrap();
                        let old_end = timeframe.end().unwrap();

                        let diff = old_end.difference(&old_start);

                        let new_start = imp.start.date_time();
                        let new_end = new_start.add(diff).unwrap();
                        Some(new_end)
                    } else {
                        None
                    };

                    if let Some(new_end) = new_end {
                        imp.end.set_date_time(new_end);
                    }

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
        pub(super) fn set_data(&self, all_day: bool, start: &DateTime, end: &DateTime) {
            self.schedule_type
                .set_active_name(Some(if all_day { "all-day" } else { "time-slot" }));
            self.start.set_date_only(all_day);
            self.end.set_date_only(all_day);

            self.start.set_date_time(start);
            self.end.set_date_time(
                end.add_days(if all_day { -1 } else { 0 })
                    .expect("Datetime should exist"),
            );

            // Notification is sent by the handlers of DateTimePickerGroup::notify_date_time or
            // schedule_type::notify_active_name if necessary
        }

        fn update_timeframe(&self) {
            let all_day = self.schedule_type.active_name().as_deref() == Some("all-day");
            let start_date_time = self.start.date_time();
            let mut end_date_time = self.end.date_time();
            if all_day {
                end_date_time = end_date_time.add_days(1).expect("Datetime should exist");
            }
            let timeframe = Timeframe::new(all_day, &start_date_time, &end_date_time).ok();
            self.timeframe.replace(timeframe);
            self.obj().notify_timeframe();
        }
    }
}

glib::wrapper! {
    pub struct TimeframePicker(ObjectSubclass<imp::TimeframePicker>)
        @extends gtk::Widget, adw::PreferencesGroup,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl TimeframePicker {
    pub fn set_data(&self, all_day: bool, start: &DateTime, end: &DateTime) {
        self.imp().set_data(all_day, start, end);
    }
}
