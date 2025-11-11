use adw::{prelude::*, subclass::prelude::*};
use clepsydre::jiff;
use glib::clone;

use crate::{
    application::Application,
    utils,
    widgets::{
        CalendarManagerDialog, CreateEventDialog, SearchDialog,
        views::{MonthView, YearView},
    },
};

pub mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/window.ui")]
    pub struct Window {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
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
    impl ObjectSubclass for Window {
        const NAME: &'static str = "CalendarManagerWindow";
        type Type = super::Window;
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
                gdk::Key::F8,
                gdk::ModifierType::NO_MODIFIER_MASK,
                "win.manage-calendars",
            );
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

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let manager = Application::default().manager();

            manager.connect_backend_available_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |manager| {
                    if manager.backend_available() {
                        imp.stack.set_visible_child_name("calendar_view");
                    } else {
                        imp.stack.set_visible_child_name("no_backend");
                        let dialogs = imp.obj().dialogs().iter().collect::<Vec<_>>();
                        for maybe_dialog in dialogs {
                            let dialog: adw::Dialog = maybe_dialog.unwrap();
                            dialog.force_close();
                        }
                    }
                }
            ));
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}

    #[gtk::template_callbacks]
    impl Window {
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
        fn open_month_view(&self, year: i32, month: i8) {
            let date = jiff::civil::Date::new(year as i16, month, 1).expect("Date should be valid");
            let week = date.iso_week_date().week();
            self.month_view.set_year(year);
            self.month_view.set_week(week);
            self.wide_view_stack.set_visible_child_name("month");
            self.narrow_stack.set_visible_child_name("month");
        }

        #[template_callback(function)]
        fn get_year_month_label(year: i32, month: i8) -> String {
            let month_name = utils::get_month_name(month);
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
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
}
