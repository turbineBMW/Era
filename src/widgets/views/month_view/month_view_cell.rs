use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, Timeframe};
use glib::clone;
use tracing::warn;

use crate::{
    application::Application,
    utils::{Date, TemplateCallbacks},
    widgets::window::Styling,
};

use super::{event_drag_payload::EventDragPayload, month_view_inner::MonthViewInner};

mod imp {

    use clepsydre::prelude::EventExt;

    use crate::spawn;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_cell.blp")]
    #[properties(wrapper_type = super::MonthViewCell)]
    pub struct MonthViewCell {
        #[property(get, set = Self::set_date, construct)]
        date: Cell<Date>,
        #[property(get, set = Self::set_styling, construct, builder(Styling::default()))]
        styling: Cell<Styling>,

        #[template_child]
        header: TemplateChild<adw::Bin>,
        #[template_child]
        day_number: TemplateChild<gtk::Label>,
        #[template_child]
        month_abbreviation: TemplateChild<gtk::Label>,
        #[template_child]
        month_name: TemplateChild<gtk::Label>,
        #[template_child]
        drop_target: TemplateChild<gtk::DropTarget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MonthViewCell {
        const NAME: &'static str = "MonthViewCell";
        type Type = super::MonthViewCell;
        type ParentType = adw::Bin;

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
    impl ObjectImpl for MonthViewCell {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_header();
            self.update_style_classes();

            self.drop_target
                .set_types(&[EventDragPayload::static_type()]);

            Application::default()
                .system()
                .connect_datetime_notify(clone!(
                    #[weak(rename_to = imp)]
                    self,
                    move |_| {
                        imp.update_style_classes();
                    }
                ));
        }
    }

    impl WidgetImpl for MonthViewCell {}
    impl BinImpl for MonthViewCell {}

    #[gtk::template_callbacks]
    impl MonthViewCell {
        /// Sets the date of the cell.
        fn set_date(&self, date: Date) {
            if self.date.get() == date {
                return;
            }

            self.date.set(date);

            self.update_header();
            self.update_style_classes();

            self.obj().notify_date();
        }

        /// Sets the styling used for the cell.
        fn set_styling(&self, styling: Styling) {
            if self.styling.get() == styling {
                return;
            }

            self.styling.set(styling);

            self.update_header();
            self.update_style_classes();

            self.obj().notify_styling();
        }

        /// Updates the style classes based on the cell's styling.
        fn update_style_classes(&self) {
            let styling = self.styling.get();
            let date = self.date.get();
            let today = Application::default().system().date();

            match styling {
                Styling::Narrow => {
                    self.obj().remove_css_class("medium");
                    self.obj().add_css_class("narrow");
                }
                Styling::Medium | Styling::Wide => {
                    self.obj().add_css_class("medium");
                    self.obj().remove_css_class("narrow");
                }
            }

            if date == today {
                self.obj().add_css_class("today");
            } else {
                self.obj().remove_css_class("today");
            }
        }

        /// Updates the header based on the cell's date and styling.
        fn update_header(&self) {
            let day = self.date.get().to_jiff().day();
            let styling = self.styling.get();

            match (day, styling) {
                (1, Styling::Narrow) => {
                    self.header.set_child(Some(&*self.month_abbreviation));
                    self.header.set_halign(gtk::Align::Center);
                }
                (1, Styling::Medium | Styling::Wide) => {
                    self.header.set_child(Some(&*self.month_name));
                    self.header.set_halign(gtk::Align::Start);
                }
                (2..=31, Styling::Narrow) => {
                    self.header.set_child(Some(&*self.day_number));
                    self.header.set_halign(gtk::Align::Center);
                }
                (2..=31, Styling::Medium | Styling::Wide) => {
                    self.header.set_child(Some(&*self.day_number));
                    self.header.set_halign(gtk::Align::Start);
                }
                _ => unreachable!(),
            }
        }

        pub(super) fn header_height(&self, width: i32) -> i32 {
            let (_minimum_header_height, natural_header_height, ..) =
                self.header.measure(gtk::Orientation::Vertical, width);

            natural_header_height
        }

        #[template_callback]
        fn event_drop(
            &self,
            _value: &glib::Value,
            x: f64,
            y: f64,
            drop_target: gtk::DropTarget,
        ) -> bool {
            let EventDragPayload { event, anchor } =
                drop_target.value_as::<EventDragPayload>().unwrap();

            let month_view = self
                .obj()
                .ancestor(MonthViewInner::static_type())
                .and_downcast::<MonthViewInner>()
                .unwrap();

            let point_in_cell_coords = gtk::graphene::Point::new(x as f32, y as f32);
            let point_in_month_view_coords = self
                .obj()
                .compute_point(&month_view, &point_in_cell_coords)
                .unwrap();

            let drop_date = month_view.date_at_coords(
                point_in_month_view_coords.x() as f64,
                point_in_month_view_coords.y() as f64,
            );

            let day_delta = (drop_date.to_jiff() - anchor.to_jiff()).get_days();
            if day_delta == 0 {
                return true;
            }

            let timeframe = event.timeframe().unwrap();
            let all_day = timeframe.is_all_day();
            let start = timeframe.start().unwrap();
            let end = timeframe.end().unwrap();

            let new_start = start.add_days(day_delta).unwrap();
            let new_end = end.add_days(day_delta).unwrap();
            let new_timeframe = Timeframe::new(all_day, &new_start, &new_end).unwrap();

            spawn!(clone!(
                #[strong]
                event,
                async move {
                    let uri = event.uri().unwrap();
                    if let Err(error) = event
                        .try_update_future(
                            None::<&Calendar>,
                            None,
                            None,
                            None,
                            None,
                            Some(&new_timeframe),
                        )
                        .await
                    {
                        warn!("Failed to move event {uri}: {}", error);
                    }
                }
            ));

            true
        }
    }
}

glib::wrapper! {
    pub struct MonthViewCell(ObjectSubclass<imp::MonthViewCell>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonthViewCell {
    pub fn new(date: Date) -> Self {
        glib::Object::builder().property("date", date).build()
    }

    pub fn header_height(&self, width: i32) -> i32 {
        self.imp().header_height(width)
    }
}
