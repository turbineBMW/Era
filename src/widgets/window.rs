use std::cell::{Cell, OnceCell};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use gettextrs::gettext;
use glib::{clone, translate::*};
use jiff::ToSpan;

use crate::{
    Application,
    utils::{Date, EventPropertiesPreset},
    widgets::{
        CalendarManagementDialog, EventCreationDialog, SearchDialog, Sidebar,
        views::{AgendaView, MonthView, YearView},
    },
};

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "Styling")]
pub enum Styling {
    #[enum_value(name = "Narrow", nick = "narrow")]
    Narrow,
    #[default]
    #[enum_value(name = "Medium", nick = "medium")]
    Medium,
    #[enum_value(name = "Wide", nick = "wide")]
    Wide,
}

pub mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/window.blp")]
    #[properties(wrapper_type = super::Window)]
    pub struct Window {
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        main_view: TemplateChild<adw::MultiLayoutView>,
        #[template_child]
        wide_view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        narrow_navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        narrow_navigation_view_year_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        narrow_navigation_view_month_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        narrow_navigation_view_agenda_page: TemplateChild<adw::NavigationPage>,
        #[template_child]
        year_view: TemplateChild<YearView>,
        #[template_child]
        month_view: TemplateChild<MonthView>,
        #[template_child]
        agenda_view: TemplateChild<AgendaView>,
        #[template_child]
        event_creation_dialog: TemplateChild<EventCreationDialog>,
        #[template_child]
        calendar_management_dialog: TemplateChild<CalendarManagementDialog>,
        #[template_child]
        search_dialog: TemplateChild<SearchDialog>,

        wide_view_stack_visible_child_name_handler_id: OnceCell<glib::SignalHandlerId>,
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
                obj.imp().search_dialog.present(Some(obj));
            });
            klass.add_binding_action(
                gdk::Key::F,
                gdk::ModifierType::CONTROL_MASK,
                "win.search-events",
            );

            klass.install_action("win.manage-calendars", None, |obj, _, _| {
                obj.imp().calendar_management_dialog.present(Some(obj));
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

                    obj.imp().event_creation_dialog.set_data(preset);
                    obj.imp().event_creation_dialog.present(Some(obj));
                },
            );
            klass.add_binding(gdk::Key::N, gdk::ModifierType::CONTROL_MASK, |obj| {
                obj.imp().create_event();
                glib::Propagation::Stop
            });

            klass.install_action("win.today", None, |obj, _, _| {
                let today = Application::default().system().date();
                obj.imp().year_view.set_date(today);
                obj.imp().month_view.set_date(today);
                obj.imp().agenda_view.set_date(today);
            });
            klass.add_binding_action(gdk::Key::T, gdk::ModifierType::CONTROL_MASK, "win.today");
            klass.add_binding_action(
                gdk::Key::Home,
                gdk::ModifierType::NO_MODIFIER_MASK,
                "win.today",
            );

            klass.install_action("win.scroll-up", None, |obj, _, _| {
                match obj
                    .imp()
                    .narrow_navigation_view
                    .visible_page_tag()
                    .expect("Narrow navigation view should have a visible page")
                    .as_str()
                {
                    "year" => {
                        obj.imp().year_view.scroll_up();
                    }
                    "month" => {
                        obj.imp().month_view.scroll_up();
                    }
                    "agenda" => {
                        obj.imp().agenda_view.scroll_up();
                    }
                    name => panic!("Unknown narrow navigation view page tag: {name}"),
                };
            });
            klass.add_binding_action(gdk::Key::Up, gdk::ModifierType::ALT_MASK, "win.scroll-up");

            klass.install_action("win.scroll-down", None, |obj, _, _| {
                match obj
                    .imp()
                    .narrow_navigation_view
                    .visible_page_tag()
                    .expect("Narrow navigation view should have a visible page")
                    .as_str()
                {
                    "year" => {
                        obj.imp().year_view.scroll_down();
                    }
                    "month" => {
                        obj.imp().month_view.scroll_down();
                    }
                    "agenda" => {
                        obj.imp().agenda_view.scroll_down();
                    }
                    name => panic!("Unknown narrow navigation view page tag: {name}"),
                };
            });
            klass.add_binding_action(
                gdk::Key::Down,
                gdk::ModifierType::ALT_MASK,
                "win.scroll-down",
            );

            klass.install_action("win.zoomin", None, |obj, _, _| {
                match obj
                    .imp()
                    .narrow_navigation_view
                    .visible_page_tag()
                    .expect("Narrow navigation view should have a visible page")
                    .as_str()
                {
                    "month" => {
                        obj.imp().month_view.zoom_in();
                    }
                    "year" | "agenda" => {}
                    name => panic!("Unknown narrow navigation view page tag: {name}"),
                };
            });
            klass.add_binding_action(
                gdk::Key::plus,
                gdk::ModifierType::CONTROL_MASK,
                "win.zoomin",
            );
            klass.add_binding_action(
                gdk::Key::equal,
                gdk::ModifierType::CONTROL_MASK,
                "win.zoomin",
            );

            klass.install_action("win.zoomout", None, |obj, _, _| {
                match obj
                    .imp()
                    .narrow_navigation_view
                    .visible_page_tag()
                    .expect("Narrow navigation view should have a visible page")
                    .as_str()
                {
                    "month" => {
                        obj.imp().month_view.zoom_out();
                    }
                    "year" | "agenda" => {}
                    name => panic!("Unknown narrow navigation view page tag: {name}"),
                };
            });
            klass.add_binding_action(
                gdk::Key::minus,
                gdk::ModifierType::CONTROL_MASK,
                "win.zoomout",
            );
            klass.add_binding_action(
                gdk::Key::underscore,
                gdk::ModifierType::CONTROL_MASK,
                "win.zoomout",
            );

            klass.install_action(
                "win.push-month-view",
                Some(&glib::VariantType::new("(iii)").unwrap()),
                |obj, _action_name, parameter| {
                    let imp = obj.imp();
                    let handler_id = imp
                        .wide_view_stack_visible_child_name_handler_id
                        .get()
                        .unwrap();

                    let (year, month, day) = parameter
                        .unwrap()
                        .get::<(i32, i32, i32)>()
                        .expect("Parameter should be of type (i32, i32, i32)");
                    let date: Date = jiff::civil::Date::new(year as i16, month as i8, day as i8)
                        .unwrap()
                        .into();

                    // Setting the visible child name will trigger the signal handler,
                    // so we need to block it first to avoid the narrow navigation view being told
                    // to push a page. We handled the navigation view push ourselves.
                    imp.wide_view_stack.block_signal(handler_id);
                    imp.wide_view_stack.set_visible_child_name("month");
                    imp.wide_view_stack.unblock_signal(handler_id);

                    imp.narrow_navigation_view.push_by_tag("month");

                    imp.month_view.set_date(date);
                },
            );

            klass.install_action(
                "win.push-agenda-view",
                Some(&glib::VariantType::new("(iii)").unwrap()),
                |obj, _action_name, parameter| {
                    let imp = obj.imp();
                    let handler_id = imp
                        .wide_view_stack_visible_child_name_handler_id
                        .get()
                        .unwrap();

                    let (year, month, day) = parameter
                        .unwrap()
                        .get::<(i32, i32, i32)>()
                        .expect("Parameter should be of type (i32, i32, i32)");
                    let date: Date = jiff::civil::Date::new(year as i16, month as i8, day as i8)
                        .unwrap()
                        .into();

                    // Setting the visible child name will trigger the signal handler,
                    // so we need to block it first to avoid the narrow navigation view being told
                    // to push a page. We handled the navigation view push ourselves.
                    imp.wide_view_stack.block_signal(handler_id);
                    imp.wide_view_stack.set_visible_child_name("agenda");
                    imp.wide_view_stack.unblock_signal(handler_id);

                    imp.narrow_navigation_view.push_by_tag("agenda");

                    imp.agenda_view.set_date(date);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
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
                self.stack.set_visible_child_name("backend-available");
            } else {
                self.stack.set_visible_child_name("backend-unavailable");
            }

            manager.connect_backend_available_notify(clone!(
                #[weak(rename_to = imp)]
                self,
                move |manager| {
                    if manager.is_backend_available() {
                        imp.stack.set_visible_child_name("backend-available");
                    } else {
                        imp.stack.set_visible_child_name("backend-unavailable");
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

            self.narrow_navigation_view
                .connect_visible_page_tag_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        match imp
                            .narrow_navigation_view
                            .visible_page_tag()
                            .expect("Narrow navigation view should have a visible page tag")
                            .as_str()
                        {
                            "year" => imp.wide_view_stack.set_visible_child_name("year"),
                            "month" => imp.wide_view_stack.set_visible_child_name("month"),
                            "agenda" => imp.wide_view_stack.set_visible_child_name("agenda"),
                            name => panic!("Unknown narrow navigation view page tag: {name}"),
                        }
                    }
                ));

            let wide_view_stack_visible_child_name_handler_id = self
                .wide_view_stack
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
                            "year" => imp
                                .narrow_navigation_view
                                .replace(&[imp.narrow_navigation_view_year_page.get()]),
                            "month" => imp.narrow_navigation_view.replace(&[
                                imp.narrow_navigation_view_year_page.get(),
                                imp.narrow_navigation_view_month_page.get(),
                            ]),
                            "agenda" => imp.narrow_navigation_view.replace(&[
                                imp.narrow_navigation_view_year_page.get(),
                                imp.narrow_navigation_view_month_page.get(),
                                imp.narrow_navigation_view_agenda_page.get(),
                            ]),
                            name => panic!("Unknown wide view stack child name: {name}"),
                        }
                    }
                ));

            self.wide_view_stack_visible_child_name_handler_id
                .set(wide_view_stack_visible_child_name_handler_id)
                .unwrap();

            // fork: open on the view chosen in the settings
            self.wide_view_stack
                .set_visible_child_name(&crate::preferences::default_view());

            // fork: the phone layout
            #[cfg(feature = "phone")]
            super::super::phone::setup(&self.obj(), &self.narrow_navigation_view);
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}

    #[gtk::template_callbacks]
    impl Window {
        fn set_styling(&self, styling: Styling) {
            self.styling.set(styling);

            match styling {
                Styling::Narrow => {
                    self.main_view.set_layout_name("narrow");
                }
                Styling::Medium => {
                    self.main_view.set_layout_name("medium");
                }
                Styling::Wide => {
                    panic!("No wide mode for window");
                }
            }

            self.obj().notify_styling();
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
                        ".color-{} {{ --calendar-color: {}; }}\n",
                        color_id.into_glib(),
                        color_str
                    ));
                }
            }

            self.colors_provider.load_from_string(&css);
        }

        #[template_callback(function)]
        fn medium_view_title(
            view: &str,
            year_view_date: &Date,
            month_view_date: &Date,
            agenda_view_date: &Date,
        ) -> String {
            match view {
                "year" => Self::year_view_title(year_view_date),
                "month" => Self::month_view_title(month_view_date),
                "agenda" => Self::agenda_view_title(agenda_view_date),
                _ => panic!("Unknown view: {view}"),
            }
        }

        #[template_callback(function)]
        fn year_view_title(year_view_date: &Date) -> String {
            year_view_date
                .to_glib_date_time_utc()
                .format(&gettext("%Y"))
                .unwrap()
                .to_string()
        }

        #[template_callback(function)]
        fn month_view_title(month_view_date: &Date) -> String {
            month_view_date
                .to_glib_date_time_utc()
                .format(&gettext("%0B %Y"))
                .unwrap()
                .to_string()
        }

        #[template_callback(function)]
        fn agenda_view_title(agenda_view_date: &Date) -> String {
            agenda_view_date
                .to_glib_date_time_utc()
                .format(&gettext("%0B %Y"))
                .unwrap()
                .to_string()
        }

        #[template_callback(function)]
        fn get_year_label(year: i32) -> String {
            year.to_string()
        }

        #[template_callback]
        fn create_event(&self) {
            let now = Application::default().system().datetime();

            let tzid = now.timezone().identifier();
            let jiff_tz = jiff::tz::TimeZone::get(&tzid).unwrap();

            let start_of_current_hour =
                jiff::Zoned::new(jiff::Timestamp::new(now.to_unix(), 0).unwrap(), jiff_tz)
                    .round(
                        jiff::ZonedRound::new()
                            .smallest(jiff::Unit::Hour)
                            .mode(jiff::RoundMode::Trunc),
                    )
                    .unwrap();

            let start = (start_of_current_hour.clone() + 1.hour()).to_string();
            let end = (start_of_current_hour + 2.hours()).to_string();

            let preset = EventPropertiesPreset {
                all_day: false,
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
