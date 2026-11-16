use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Timeframe, prelude::*};
use glib::clone;
use tracing::warn;

use crate::{spawn, utils::Date};

use super::{event_drag_payload::EventDragPayload, month_view_inner::MonthViewInner};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/views/month_view/month_view_cell.blp")]
    #[properties(wrapper_type = super::MonthViewCell)]
    pub struct MonthViewCell {
        #[property(get, set, construct)]
        date: Cell<Date>,

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
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MonthViewCell {
        fn constructed(&self) {
            self.parent_constructed();

            self.drop_target
                .set_types(&[EventDragPayload::static_type()]);
        }
    }

    impl WidgetImpl for MonthViewCell {}
    impl BinImpl for MonthViewCell {}

    #[gtk::template_callbacks]
    impl MonthViewCell {
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

            let timeframe = event.timeframe();
            let all_day = timeframe.is_all_day();
            let start = timeframe.start();
            let end = timeframe.end();

            let new_start = start.add_days(day_delta).unwrap();
            let new_end = end.add_days(day_delta).unwrap();
            let new_timeframe = Timeframe::new(all_day, &new_start, &new_end).unwrap();

            spawn!(clone!(
                #[strong]
                event,
                async move {
                    let uri = event.uri();
                    if let Err(error) = event
                        .try_update_future(None, None, None, None, Some(&new_timeframe))
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
}
