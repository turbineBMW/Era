use std::cell::{Cell, OnceCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Subscription, Timeframe, prelude::*};
use glib::{TimeZone, clone};

use crate::{
    Application, utils::TemplateCallbacks, widgets::event_details_dialog::EventDetailsDialog,
};

mod agenda_view_row;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/agenda_view.ui")]
    #[properties(wrapper_type = super::AgendaView)]
    pub struct AgendaView {
        #[property(get)]
        year: Cell<i32>,
        #[property(get)]
        month: Cell<i32>,
        #[property(get)]
        day: Cell<i32>,
        #[property(get)]
        subscription: OnceCell<Subscription>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        events: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AgendaView {
        const NAME: &'static str = "AgendaView";
        type Type = super::AgendaView;
        type ParentType = gtk::Box;

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
    impl ObjectImpl for AgendaView {
        fn constructed(&self) {
            self.parent_constructed();

            let application = Application::default();
            let manager = application.manager();
            let system = application.system();

            let now = system.datetime();

            let tomorrow = now.add_days(1).unwrap();
            let timeframe =
                Timeframe::new(true, &now, &tomorrow).expect("start should be before end");
            let subscription = manager.new_subscription(&timeframe).unwrap().unwrap();
            self.subscription
                .set(subscription.clone())
                .expect("Subscription should not be initialized yet");

            self.set_year_month_day(now.year(), now.month(), now.day_of_month());

            self.events
                .set_model(Some(&gtk::NoSelection::new(Some(subscription.clone()))));

            subscription.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |subscription, _, _, _| {
                    if subscription.n_items() == 0 {
                        imp.stack.set_visible_child_name("empty");
                    } else {
                        imp.stack.set_visible_child_name("events");
                    }
                }
            ));
        }
    }
    impl WidgetImpl for AgendaView {}
    impl BoxImpl for AgendaView {}

    #[gtk::template_callbacks]
    impl AgendaView {
        /// Sets the triplet year-month-day.
        pub(super) fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
            if self.year.get() != year {
                self.year.set(year);
                self.obj().notify_year();
            }

            if self.month.get() != month {
                self.month.set(month);
                self.obj().notify_month();
            }

            if self.day.get() != day {
                self.day.set(day);
                self.obj().notify_day();
            }

            let start = glib::DateTime::new(&TimeZone::utc(), year, month, day, 0, 0, 0.).unwrap();
            let end = start.add_days(1).unwrap();
            let timeframe = Timeframe::new(true, &start, &end).unwrap();
            self.subscription
                .get()
                .unwrap()
                .set_timeframe(Some(&timeframe));
        }

        #[template_callback]
        fn day_label(&self) -> String {
            format!(
                "{}-{:02}-{:02}",
                self.year.get(),
                self.month.get(),
                self.day.get()
            )
        }

        #[template_callback]
        fn open_event_details(&self, item: u32) {
            let obj = self.obj();
            let event = self
                .events
                .model()
                .unwrap()
                .item(item)
                .unwrap()
                .downcast::<Event>()
                .unwrap();
            let dialog = EventDetailsDialog::new(&event);
            dialog.present(Some(&*obj));
        }
    }
}

glib::wrapper! {
    pub struct AgendaView(ObjectSubclass<imp::AgendaView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AgendaView {
    /// Sets the triplet year-month-day.
    pub fn set_year_month_day(&self, year: i32, month: i32, day: i32) {
        self.imp().set_year_month_day(year, month, day);
    }
}
