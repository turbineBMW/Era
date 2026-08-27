use std::cell::{OnceCell, RefCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Event, Subscription, prelude::*};
use glib::clone;

use crate::{
    Application,
    utils::{Date, TemplateCallbacks},
    widgets::event_details_dialog::EventDetailsDialog,
};

mod agenda_view_row;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/agenda_view.ui")]
    #[properties(wrapper_type = super::AgendaView)]
    pub struct AgendaView {
        #[property(get, set = Self::set_date)]
        date: RefCell<Date>,
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

            let subscription = manager.new_subscription(&now, &now).unwrap();
            self.subscription
                .set(subscription.clone())
                .expect("Subscription should not be initialized yet");

            self.obj().set_date(system.date());

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

            system.connect_datetime_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_system| {
                    imp.update_timeframe();
                }
            ));
        }
    }

    impl WidgetImpl for AgendaView {}
    impl BoxImpl for AgendaView {}

    #[gtk::template_callbacks]
    impl AgendaView {
        fn set_date(&self, date: Date) {
            if date == *self.date.borrow() {
                return;
            }

            self.date.replace(date);

            self.update_timeframe();

            self.obj().notify_date();
        }

        fn update_timeframe(&self) {
            let date = *self.date.borrow();
            let timezone = Application::default().system().datetime().timezone();
            let start = date.to_glib_date_time(&timezone);
            let end = start.add_days(1).unwrap();
            self.subscription.get().unwrap().set_timeframe(&start, &end);
        }

        #[template_callback]
        fn day_label(&self) -> String {
            let date = self.date.borrow().to_jiff();
            format!("{}-{:02}-{:02}", date.year(), date.month(), date.day())
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

impl AgendaView {}
