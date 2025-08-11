use std::{
    cell::{Cell, OnceCell, RefCell},
    sync::{LazyLock, Mutex},
};

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    Allocation,
    glib::{self, clone, subclass::Signal},
};

mod year_view_month_cell;
mod year_view_year_row;

use crate::Application;

use self::{year_view_month_cell::*, year_view_year_row::*};

const MINIMUM_NB_ROWS_ABOVE: f64 = 0.5;
const MINIMUM_NB_ROWS_BELOW: f64 = 3.;
const NB_ROWS: i32 = 7;
const SPACING: i32 = 12;
const VELOCITY_THRESHOLD_TO_RETURN: f64 = 300.;
const VELOCITY_THRESHOLD_TO_SNAP: f64 = 400.;
const VELOCITY_THRESHOLD_TO_SKIP: f64 = 10.;
const FIRST_STAGE_DIVISOR: f64 = 1.5;
const FIRST_TO_SECOND_STAGE_THRESHOLD: f64 = 300.;
const SECOND_STAGE_DIVISOR: f64 = 2.5;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "YearViewStyling")]
pub enum YearViewStyling {
    #[enum_value(name = "Narrow", nick = "narrow")]
    Narrow,
    #[default]
    #[enum_value(name = "Medium", nick = "medium")]
    Medium,
    #[enum_value(name = "Wide", nick = "wide")]
    Wide,
}

pub(crate) mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/year_view.ui")]
    #[properties(wrapper_type = super::YearView)]
    pub struct YearView {
        #[property(get)]
        year: Cell<i32>,
        #[property(get, set, builder(YearViewStyling::default()))]
        styling: Cell<YearViewStyling>,
        // TODO: I should remove the OnceCell? Should I use Cell instead of Mutex?
        year_rows: OnceCell<Mutex<Vec<YearViewYearRow>>>,
        scroll_offset: Cell<f64>,
        scroll_animation: RefCell<Option<adw::TimedAnimation>>,
        last_velocity: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearView {
        const NAME: &'static str = "YearView";
        type Type = super::YearView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            YearViewMonthCell::ensure_type();
            YearViewYearRow::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for YearView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let application = Application::default();
            let current_year = application.current_year();
            self.year.set(current_year);
            obj.notify_year();

            let first_row = YearViewYearRow::new(current_year - 1);
            obj.bind_property("styling", &first_row, "styling")
                .sync_create()
                .build();
            first_row.connect_month_clicked(clone!(
                #[weak(rename_to = imp)]
                self,
                move |_row, year, month| {
                    imp.obj()
                        .emit_by_name::<()>("month-clicked", &[&year, &month]);
                }
            ));
            first_row.insert_before(&*self.obj(), None::<&gtk::Widget>);

            // TODO: Make sure 800 is the same as the default width of the window
            let (row_height, ..) = first_row.measure(gtk::Orientation::Vertical, 800);
            let offset = row_height as f64;
            self.scroll_offset.set(offset);

            let mut year_rows = vec![first_row];
            for year in current_year..current_year + NB_ROWS {
                let row = YearViewYearRow::new(year);
                obj.bind_property("styling", &row, "styling")
                    .sync_create()
                    .build();
                row.insert_before(&*self.obj(), None::<&gtk::Widget>);
                row.connect_month_clicked(clone!(
                    #[weak]
                    obj,
                    move |_row, year, month| {
                        obj.emit_by_name::<()>("month-clicked", &[&year, &month]);
                    }
                ));
                year_rows.push(row);
            }
            self.year_rows.get_or_init(|| Mutex::new(year_rows));
        }

        fn dispose(&self) {
            for row in self.year_rows.get().unwrap().lock().unwrap().iter() {
                row.unparent();
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("month-clicked")
                        // Year, Month
                        // TODO: Should these be something else than i32?
                        .param_types([i32::static_type(), i32::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for YearView {
        // TODO: Check if i have been allocated enough space
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let last_row = year_rows
                .last()
                .expect("There should be at least one year row")
                .to_owned();
            let (minimum_row_height, ..) = last_row.measure(gtk::Orientation::Vertical, width);

            for (i, row) in year_rows.iter().enumerate() {
                let allocation = Allocation::new(
                    0,
                    -self.scroll_offset.get() as i32 + (i as i32 * (minimum_row_height + SPACING)),
                    width,
                    minimum_row_height,
                );
                row.size_allocate(&allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl YearView {
        fn change_scroll_offset(&self, dy: f64) {
            let obj = self.obj();

            let mut year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let height = obj.height();
            let row_height = (year_rows
                .first()
                .expect("There should be at least one year row")
                .height()
                + SPACING) as f64;

            // The y offset of the top of the first row
            let top_offset = self.scroll_offset.get() + dy;
            // The y offset of the bottom of the last row
            let bottom_offset = top_offset + height as f64;

            let first_row = year_rows
                .first()
                .expect("There should be at least one year row")
                .to_owned();
            let first_year = first_row.year();
            self.year
                .set(first_year + top_offset as i32 / row_height as i32);
            obj.notify_year();

            let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE;
            let bottom_threshold = (year_rows.len() as f64 - MINIMUM_NB_ROWS_BELOW) * row_height;

            if top_offset < top_threshold {
                self.scroll_offset.set(top_offset + row_height);

                let first_row = year_rows
                    .first()
                    .expect("There should be at least one year row")
                    .to_owned();
                let last_row = year_rows.pop().unwrap();
                last_row.set_year(first_row.year() - 1);

                year_rows.insert(0, last_row);
            } else if bottom_offset > bottom_threshold {
                self.scroll_offset.set(top_offset - row_height);

                let first_row = year_rows.remove(0);
                let last_row = year_rows
                    .last()
                    .expect("There should be at least one year row")
                    .clone();
                first_row.set_year(last_row.year() + 1);

                year_rows.push(first_row);
            } else {
                self.scroll_offset.set(top_offset);
            }

            obj.queue_allocate();
        }

        fn inertia_scrolling(&self, mut velocity: f64) {
            let obj = self.obj();

            if let Some(kinetic_scroll_animation) = self.scroll_animation.borrow().as_ref() {
                kinetic_scroll_animation.pause();
            }
            self.scroll_animation.replace(None);

            if velocity.abs() < VELOCITY_THRESHOLD_TO_RETURN {
                return;
            }

            velocity = if velocity > FIRST_TO_SECOND_STAGE_THRESHOLD {
                FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (velocity - FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else if velocity < -FIRST_TO_SECOND_STAGE_THRESHOLD {
                -FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (velocity + FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else {
                velocity / FIRST_STAGE_DIVISOR
            };

            let row_height = (self
                .year_rows
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .first()
                .unwrap()
                .height()
                + SPACING) as f64;
            let offset_from_a_row = self.scroll_offset.get() % row_height;

            // Adjust velocity to snap to the start of a year row
            if velocity > VELOCITY_THRESHOLD_TO_SNAP {
                velocity =
                    (velocity / row_height).floor() * row_height + row_height - offset_from_a_row;
            } else if velocity < -VELOCITY_THRESHOLD_TO_SNAP {
                velocity = (velocity / row_height).ceil() * row_height
                    - offset_from_a_row
                    - SPACING as f64;
            };

            self.last_velocity.set(velocity);

            let duration_ms = velocity.abs() / obj.height() as f64 * 1000.;
            let animation_target = adw::CallbackAnimationTarget::new(clone!(
                #[weak(rename_to = imp)]
                self,
                move |new_velocity| {
                    if new_velocity.abs() < VELOCITY_THRESHOLD_TO_SKIP
                        && let Some(animation) = imp.scroll_animation.borrow().as_ref()
                    {
                        animation.skip();
                        return;
                    }

                    let dy = imp.last_velocity.get() - new_velocity;
                    imp.last_velocity.set(new_velocity);
                    imp.change_scroll_offset(dy);
                }
            ));
            let kinetic_scroll_animation =
                adw::TimedAnimation::new(&*obj, velocity, 0., duration_ms as u32, animation_target);
            kinetic_scroll_animation.set_easing(adw::Easing::EaseOutExpo);
            kinetic_scroll_animation.set_follow_enable_animations_setting(false);

            self.scroll_animation
                .replace(Some(kinetic_scroll_animation.clone()));
            kinetic_scroll_animation.play();
        }

        #[template_callback]
        fn get_year_label_narrow(&self) -> String {
            self.obj().year().to_string()
        }

        #[template_callback]
        fn month_cell_clicked(&self, year: i32, month: i32) {
            self.obj()
                .emit_by_name::<()>("month-clicked", &[&year, &month]);
        }

        #[template_callback]
        fn kinetic_scroll_begin(&self) {
            if let Some(kinetic_scroll_animation) = self.scroll_animation.borrow().as_ref() {
                kinetic_scroll_animation.pause();
            }
            self.scroll_animation.replace(None);
        }

        #[template_callback]
        fn kinetic_scroll(&self, _dx: f64, dy: f64) -> bool {
            if let Some(kinetic_scroll_animation) = self.scroll_animation.borrow().as_ref() {
                kinetic_scroll_animation.pause();
            }
            self.scroll_animation.replace(None);

            self.change_scroll_offset(dy);
            true
        }

        // TODO: Rename velocity to something more accurate
        #[template_callback]
        fn kinetic_decelerate(&self, _velocity_x: f64, velocity_y: f64) {
            self.inertia_scrolling(velocity_y);
        }

        #[template_callback]
        fn swipe(&self, _velocity_x: f64, velocity_y: f64) {
            self.inertia_scrolling(-velocity_y);
        }
    }
}

glib::wrapper! {
    pub struct YearView(ObjectSubclass<imp::YearView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
