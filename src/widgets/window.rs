use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use glib::{clone, translate::*};

use crate::{
    Application,
    utils::{EventPropertiesPreset, TemplateCallbacks},
    widgets::{
        CalendarManagementDialog, CreateEventDialog, SearchDialog, Sidebar,
        views::{MonthView, YearView},
    },
};

pub mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/window.ui")]
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
        narrow_view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        year_view: TemplateChild<YearView>,
        #[template_child]
        month_view: TemplateChild<MonthView>,

        colors_provider: gtk::CssProvider,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "EraWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            Sidebar::ensure_type();

            klass.install_action("win.search-events", None, |obj, _, _| {
                let dialog = SearchDialog::new();
                dialog.present(Some(obj));
            });
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "win.search-events",
            );

            klass.install_action("win.manage-calendars", None, |obj, _, _| {
                let dialog = CalendarManagementDialog::new();
                dialog.present(Some(obj));
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

            klass.install_action(
                "win.create-event",
                Some(&EventPropertiesPreset::static_variant_type()),
                |obj, _action_name, parameter| {
                    let preset = parameter
                        .unwrap()
                        .get::<EventPropertiesPreset>()
                        .expect("Parameter should be of type EventPropertiesPreset");

                    let dialog = CreateEventDialog::new(preset);
                    dialog.present(Some(obj));
                },
            );
            klass.add_binding(gdk::Key::N, gdk::ModifierType::CONTROL_MASK, |obj| {
                obj.imp().create_event();
                glib::Propagation::Stop
            });

            klass.install_action("win.today", None, |obj, _, _| {
                obj.imp().today();
            });
            klass.add_binding_action(gdk::Key::T, gdk::ModifierType::CONTROL_MASK, "win.today");
            klass.add_binding_action(
                gdk::Key::Home,
                gdk::ModifierType::NO_MODIFIER_MASK,
                "win.today",
            );
            klass.add_binding_action(gdk::Key::Down, gdk::ModifierType::ALT_MASK, "win.today");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            // Setup calendar colors CSS provider
            let display = RootExt::display(&*self.obj());
            gtk::style_context_add_provider_for_display(
                &display,
                &self.colors_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
            );

            let manager = Application::default().manager();

            if manager.is_backend_available() {
                self.stack.set_visible_child_name("calendar_view");
            } else {
                self.stack.set_visible_child_name("no_backend");
            }

            manager.connect_backend_available_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |manager| {
                    if manager.is_backend_available() {
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

            // Setup calendar colors
            let calendars_model = manager.calendars_model().unwrap();
            self.recalculate_calendar_colors_css();

            for i in 0..calendars_model.n_items() {
                let calendar: Calendar = calendars_model.item(i).unwrap().downcast().unwrap();
                calendar.connect_color_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.recalculate_calendar_colors_css();
                    }
                ));
            }

            calendars_model.connect_items_changed(clone!(
                #[weak(rename_to = imp)]
                self,
                move |model, position, _removed, added| {
                    for i in 0..added {
                        let calendar: Calendar =
                            model.item(position + i).unwrap().downcast().unwrap();
                        calendar.connect_color_notify(clone!(
                            #[weak]
                            imp,
                            move |_| {
                                imp.recalculate_calendar_colors_css();
                            }
                        ));
                    }
                    imp.recalculate_calendar_colors_css();
                },
            ));

            self.narrow_stack.connect_visible_child_name_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    match imp
                        .narrow_stack
                        .visible_child_name()
                        .expect("Narrow stack should have a visible child")
                        .as_str()
                    {
                        "year" => imp.wide_view_stack.set_visible_child_name("year"),
                        "month" => imp.wide_view_stack.set_visible_child_name("month"),
                        "days" => match imp
                            .narrow_view_stack
                            .visible_child_name()
                            .expect("Narrow stack should have a visible child")
                            .as_str()
                        {
                            "days" => imp.wide_view_stack.set_visible_child_name("week"),
                            "day" => imp.wide_view_stack.set_visible_child_name("days"),
                            "agenda" => imp.wide_view_stack.set_visible_child_name("agenda"),
                            name => panic!("Unknown narrow stack child name: {name}"),
                        },
                        name => panic!("Unknown narrow stack child name: {name}"),
                    }
                }
            ));

            self.narrow_view_stack
                .connect_visible_child_name_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        match imp
                            .narrow_stack
                            .visible_child_name()
                            .expect("Narrow stack should have a visible child")
                            .as_str()
                        {
                            "year" => imp.wide_view_stack.set_visible_child_name("year"),
                            "month" => imp.wide_view_stack.set_visible_child_name("month"),
                            "days" => match imp
                                .narrow_view_stack
                                .visible_child_name()
                                .expect("Narrow stack should have a visible child")
                                .as_str()
                            {
                                "days" => imp.wide_view_stack.set_visible_child_name("week"),
                                "day" => imp.wide_view_stack.set_visible_child_name("days"),
                                "agenda" => imp.wide_view_stack.set_visible_child_name("agenda"),
                                name => panic!("Unknown narrow view stack child name: {name}"),
                            },
                            name => panic!("Unknown narrow stack child name: {name}"),
                        }
                    }
                ));

            self.wide_view_stack
                .connect_visible_child_name_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        match imp
                            .wide_view_stack
                            .visible_child_name()
                            .expect("Narrow stack should have a visible child")
                            .as_str()
                        {
                            "year" => imp.narrow_stack.set_visible_child_name("year"),
                            "month" => imp.narrow_stack.set_visible_child_name("month"),
                            "week" => {
                                imp.narrow_stack.set_visible_child_name("days");
                                imp.narrow_view_stack.set_visible_child_name("days");
                            }
                            "days" => {
                                imp.narrow_stack.set_visible_child_name("days");
                                imp.narrow_view_stack.set_visible_child_name("day");
                            }
                            "agenda" => {
                                imp.narrow_stack.set_visible_child_name("days");
                                imp.narrow_view_stack.set_visible_child_name("agenda");
                            }
                            name => panic!("Unknown wide view stack child name: {name}"),
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
        fn today(&self) {
            let today = Application::default().current_datetime();
            let year = today.year();
            let month = today.month();
            let day = today.day_of_month();
            self.month_view.set_year_month_day(year, month, day);
            self.year_view.set_year(year);
        }

        fn recalculate_calendar_colors_css(&self) {
            let manager = Application::default().manager();
            let calendars_model = manager.calendars_model().unwrap();
            let mut css = String::new();

            for i in 0..calendars_model.n_items() {
                let calendar: Calendar = calendars_model.item(i).unwrap().downcast().unwrap();
                if let Some(color) = calendar.color() {
                    let color_str = color.to_string();
                    let color_id = glib::Quark::from_str(&color_str);
                    css.push_str(&format!(
                        ".color-{} {{ --calendar-bg-color: {}; }}\n",
                        color_id.into_glib(),
                        color_str
                    ));
                }
            }

            self.colors_provider.load_from_string(&css);
        }

        #[template_callback(function)]
        fn get_year_label(year: i32) -> String {
            year.to_string()
        }

        #[template_callback]
        fn open_month_view(&self, year: i32, month: i32) {
            self.month_view.set_year_month_day(year, month, 1);
            self.wide_view_stack.set_visible_child_name("month");
            self.narrow_stack.set_visible_child_name("month");
        }

        #[template_callback(function)]
        fn get_year_month_label(year: i32, month: i32) -> String {
            let month_name = TemplateCallbacks::capitalized_month_name(month);
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

        #[template_callback]
        fn create_event(&self) {
            let today = Application::default().current_datetime();
            let tomorrow = today.add_days(1).unwrap();

            let start = today.format_iso8601().unwrap().to_string();
            let end = tomorrow.format_iso8601().unwrap().to_string();

            let preset = EventPropertiesPreset {
                all_day: true,
                start,
                end,
                ..Default::default()
            };

            let _ = WidgetExt::activate_action(
                &*self.obj(),
                "win.create-event",
                Some(&preset.to_variant()),
            );
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
