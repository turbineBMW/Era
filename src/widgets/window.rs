use adw::{prelude::*, subclass::prelude::*};
use ccm::jiff;
use gettextrs::gettext;
use gtk::{gdk, gio, glib};

use crate::widgets::{
    CalendarManagerDialog, CreateEventDialog, SearchDialog,
    views::{MonthView, YearView},
};

pub(crate) mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/CalendarManager/window.ui")]
    pub struct CalendarManagerWindow {
        #[template_child]
        main_view: TemplateChild<adw::MultiLayoutView>,
        #[template_child]
        wide_view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        narrow_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        month_view: TemplateChild<MonthView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarManagerWindow {
        const NAME: &'static str = "CalendarManagerWindow";
        type Type = super::CalendarManagerWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            YearView::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("win.search-events", None, |obj, _, _| {
                obj.imp().search_events();
            });

            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "win.search-events",
            );

            klass.install_action("win.manage-calendars", None, |obj, _, _| {
                obj.imp().manage_calendars();
            });

            klass.add_binding_action(
                gdk::Key::M,
                gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK,
                "win.manage-calendars",
            );

            klass.install_action("win.create-event", None, |obj, _, _| {
                obj.imp().create_event();
            });

            klass.add_binding_action(
                gdk::Key::N,
                gdk::ModifierType::CONTROL_MASK,
                "win.create-event",
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CalendarManagerWindow {}
    impl WidgetImpl for CalendarManagerWindow {}
    impl WindowImpl for CalendarManagerWindow {}
    impl ApplicationWindowImpl for CalendarManagerWindow {}
    impl AdwApplicationWindowImpl for CalendarManagerWindow {}

    #[gtk::template_callbacks]
    impl CalendarManagerWindow {
        #[template_callback]
        fn search_events(&self) {
            let dialog = SearchDialog::new();
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        fn manage_calendars(&self) {
            let dialog = CalendarManagerDialog::new();
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback]
        fn create_event(&self) {
            let dialog = CreateEventDialog::new();
            dialog.present(Some(&*self.obj()));
        }

        #[template_callback(function)]
        fn get_year_label(year: i32) -> String {
            year.to_string()
        }

        #[template_callback]
        fn open_month_view(&self, year: i32, month: i32) {
            let date =
                jiff::civil::Date::new(year as i16, month as i8, 1).expect("Date should be valid");
            let week = date.iso_week_date().week();
            self.month_view.set_year(year);
            self.month_view.set_week(week);
            self.wide_view_stack.set_visible_child_name("month");
            self.narrow_stack.set_visible_child_name("month");
        }

        #[template_callback(function)]
        fn get_year_month_label(year: i32, month: i32) -> String {
            let month_name = match month {
                1 => gettext("January"),
                2 => gettext("February"),
                3 => gettext("March"),
                4 => gettext("April"),
                5 => gettext("May"),
                6 => gettext("June"),
                7 => gettext("July"),
                8 => gettext("August"),
                9 => gettext("September"),
                10 => gettext("October"),
                11 => gettext("November"),
                12 => gettext("December"),
                _ => "".to_string(),
            };
            format!("{month_name} {year} ")
        }

        #[template_callback]
        fn go_back_to_year_view(&self) {
            self.wide_view_stack.set_visible_child_name("year");
            self.narrow_stack.set_visible_child_name("year");
        }

        #[template_callback]
        fn open_days_view(&self) {
            match self
                .main_view
                .layout_name()
                .expect("A layout should be selected")
                .as_str()
            {
                "wide" => (),
                "narrow" => self.narrow_stack.set_visible_child_name("days"),
                _ => (),
            }
        }

        #[template_callback]
        fn go_back_to_month_view(&self) {
            match self
                .main_view
                .layout_name()
                .expect("A layout should be selected")
                .as_str()
            {
                "wide" => (),
                "narrow" => self.narrow_stack.set_visible_child_name("month"),
                _ => (),
            }
        }
    }
}

glib::wrapper! {
    pub struct CalendarManagerWindow(ObjectSubclass<imp::CalendarManagerWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CalendarManagerWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
