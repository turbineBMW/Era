use std::{
    cell::{Cell, OnceCell, RefCell},
    sync::{LazyLock, Mutex},
};

use adw::{prelude::*, subclass::prelude::*};
use glib::{clone, subclass::Signal};
use gtk::Allocation;

mod year_view_month_cell;
mod year_view_year_row;

use crate::Application;

use self::{year_view_month_cell::YearViewMonthCell, year_view_year_row::YearViewYearRow};

const MINIMUM_NB_ROWS_ABOVE: f64 = 0.5;
const MINIMUM_NB_ROWS_BELOW: f64 = 8.;
const NB_ROWS: i32 = 12;
const VELOCITY_THRESHOLD_TO_RETURN: f64 = 300.;
const VELOCITY_THRESHOLD_TO_SNAP: f64 = 400.;
const VELOCITY_THRESHOLD_TO_SKIP: f64 = 2.;
const FIRST_STAGE_DIVISOR: f64 = 1.5;
const FIRST_TO_SECOND_STAGE_THRESHOLD: f64 = 300.;
const SECOND_STAGE_DIVISOR: f64 = 2.5;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP: f64 = 100.;
const DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW: f64 = 50.;

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

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Kalendasom/year_view.ui")]
    #[properties(wrapper_type = super::YearView)]
    pub struct YearView {
        #[property(get)]
        year: Cell<i32>,
        #[property(get, set, builder(YearViewStyling::default()))]
        styling: Cell<YearViewStyling>,
        year_rows: OnceCell<Mutex<Vec<YearViewYearRow>>>,
        year_row_height: Cell<i32>,
        scroll_offset: Cell<f64>,
        scroll_animation: RefCell<Option<adw::TimedAnimation>>,
        pixels_left: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for YearView {
        const NAME: &'static str = "YearView";
        type Type = super::YearView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.set_css_name("year-view");
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
            let current_year = application.current_datetime().year();
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
                        .param_types([i32::static_type(), i32::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for YearView {
        fn size_allocate(&self, width: i32, _height: i32, baseline: i32) {
            let year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let last_row = year_rows
                .last()
                .expect("There should be at least one year row")
                .to_owned();
            let (row_height, ..) = last_row.measure(gtk::Orientation::Vertical, width);

            self.year_row_height.set(row_height);

            for (i, row) in year_rows.iter().enumerate() {
                let i = i as i32;
                let scroll_offset = self.scroll_offset.get() as i32;
                let allocation =
                    Allocation::new(0, -scroll_offset + i * row_height, width, row_height);
                row.size_allocate(&allocation, baseline);
            }
        }
    }

    #[gtk::template_callbacks]
    impl YearView {
        /// Move the view by the given number of pixels.
        fn change_scroll_offset(&self, dy: f64) {
            let obj = self.obj();

            let mut year_rows = self.year_rows.get().unwrap().lock().unwrap();
            let height = obj.height();
            let row_height = self.year_row_height.get() as f64;

            // The offset of the top of the first row
            let top_offset = self.scroll_offset.get() + dy;
            // The offset of the bottom of the last row
            let bottom_offset = top_offset + height as f64;
            // The limit of the top offset before recycling happens
            let top_threshold = row_height * MINIMUM_NB_ROWS_ABOVE;
            // The limit of the bottom offset before recycling happens
            let bottom_threshold = (year_rows.len() as f64 - MINIMUM_NB_ROWS_BELOW) * row_height;

            // Update the year property
            let first_row = year_rows
                .first()
                .expect("There should be at least one year row")
                .to_owned();
            let first_year = first_row.year();
            self.year
                .set(first_year + (top_offset / row_height + 0.05) as i32);
            obj.notify_year();

            // Recycle a row if necessary and set the new scroll offset
            // TODO: Recycle multiple rows if needed
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

        /// Start a scroll animation.
        ///
        /// The animation will move the view by the given amount of pixels over a certain amount of
        /// time.
        fn start_scroll_animation(&self, pixels: f64) {
            let obj = self.obj();

            self.cancel_scroll_animation();

            self.pixels_left.set(pixels);

            let duration_ms = pixels.abs() / obj.height() as f64 * 1000.;
            let animation_target = adw::CallbackAnimationTarget::new(clone!(
                #[weak(rename_to = imp)]
                self,
                move |pixels_to_leave| {
                    let pixels_left = imp.pixels_left.get();

                    // If there are not enough pixels to leave, leave no pixel for later and skip
                    // the animation. This allows to fix a little jump at the end of the animation,
                    // while keeping the precision of the target scroll offset.
                    if pixels_to_leave.abs() < VELOCITY_THRESHOLD_TO_SKIP
                        && let Some(animation) = imp.scroll_animation.borrow().as_ref()
                    {
                        imp.pixels_left.set(0.);
                        imp.change_scroll_offset(pixels_left);

                        animation.skip();
                        return;
                    }

                    let dy = pixels_left - pixels_to_leave;
                    imp.pixels_left.set(pixels_to_leave);
                    imp.change_scroll_offset(dy);
                }
            ));
            let kinetic_scroll_animation =
                adw::TimedAnimation::new(&*obj, pixels, 0., duration_ms as u32, animation_target);
            kinetic_scroll_animation.set_easing(adw::Easing::EaseOutExpo);
            kinetic_scroll_animation.set_follow_enable_animations_setting(false);

            self.scroll_animation
                .replace(Some(kinetic_scroll_animation.clone()));
            kinetic_scroll_animation.play();
        }

        /// Cancel the current scroll animation, if any.
        fn cancel_scroll_animation(&self) {
            if let Some(kinetic_scroll_animation) = self.scroll_animation.borrow().as_ref() {
                kinetic_scroll_animation.pause();
            }
            self.scroll_animation.replace(None);
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
            self.cancel_scroll_animation();
        }

        #[template_callback]
        fn kinetic_scroll(
            &self,
            _dx: f64,
            dy: f64,
            controller: gtk::EventControllerScroll,
        ) -> bool {
            match controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => return false,
                gdk::ScrollDirection::Smooth => (),
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                direction => panic!("Unknown scroll direction: {direction:?}"),
            }

            self.cancel_scroll_animation();
            self.change_scroll_offset(dy);

            true
        }

        #[template_callback]
        fn kinetic_decelerate(&self, _dx: f64, dy: f64) {
            if dy.abs() < VELOCITY_THRESHOLD_TO_RETURN {
                return;
            }

            let dy = if dy > FIRST_TO_SECOND_STAGE_THRESHOLD {
                FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (dy - FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else if dy < -FIRST_TO_SECOND_STAGE_THRESHOLD {
                -FIRST_TO_SECOND_STAGE_THRESHOLD / FIRST_STAGE_DIVISOR
                    + (dy + FIRST_TO_SECOND_STAGE_THRESHOLD) / SECOND_STAGE_DIVISOR
            } else {
                dy / FIRST_STAGE_DIVISOR
            };

            let row_height = self.year_row_height.get() as f64;
            let scroll_offset = self.scroll_offset.get();
            let offset_from_a_row = scroll_offset % row_height;

            // Adjust the scroll to snap to the start of a row if close enough
            let pixels = if dy > VELOCITY_THRESHOLD_TO_SNAP {
                (dy / row_height).floor() * row_height + row_height - offset_from_a_row
            } else if dy < -VELOCITY_THRESHOLD_TO_SNAP {
                (dy / row_height).ceil() * row_height - offset_from_a_row
            } else {
                dy
            };
            self.start_scroll_animation(pixels);
        }

        #[template_callback]
        fn discrete_scroll(
            &self,
            _dx: f64,
            dy: f64,
            controller: gtk::EventControllerScroll,
        ) -> bool {
            match controller
                .current_event()
                .expect("Controller should have a current event")
                .downcast::<gdk::ScrollEvent>()
                .expect("A scroll controller should only have a scroll event")
                .direction()
            {
                gdk::ScrollDirection::Up | gdk::ScrollDirection::Down => (),
                gdk::ScrollDirection::Smooth => return false,
                gdk::ScrollDirection::Left | gdk::ScrollDirection::Right => {
                    panic!("Vertical scroll controller should not signal horizontal scroll events")
                }
                direction => panic!("Unknown scroll direction: {direction:?}"),
            }

            let height = self.obj().height() as f64;
            let row_height = self.year_row_height.get() as f64;
            let scroll_offset = self.scroll_offset.get();
            let number_of_scroll_steps = dy;
            let distance_to_previous_row_start = scroll_offset % row_height;
            let distance_to_next_row_start = row_height - distance_to_previous_row_start;

            // If one year row does not fit in the view, scroll by the biggest slice of a row that
            // fits in the view
            if row_height > height {
                let number_of_slices_per_row = (row_height / height).ceil() as i32;
                let slice_height = row_height / number_of_slices_per_row as f64;
                let scroll_request = number_of_scroll_steps * slice_height;
                let upcoming_distance_to_previous_row_start =
                    (scroll_offset + scroll_request) % row_height;
                let upcoming_distance_to_next_row_start =
                    row_height - upcoming_distance_to_previous_row_start;

                // Adjust the scroll to snap to the start of a row if close enough
                let pixels = if (upcoming_distance_to_previous_row_start)
                    < DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP
                {
                    scroll_request - upcoming_distance_to_previous_row_start
                } else if (upcoming_distance_to_next_row_start)
                    < DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_SNAP
                {
                    scroll_request + upcoming_distance_to_next_row_start
                } else {
                    scroll_request
                };
                self.start_scroll_animation(pixels);

                return true;
            }

            // Else, scroll to the next year row start
            let pixels = if number_of_scroll_steps < 0. {
                let mut pixels =
                    number_of_scroll_steps * row_height - distance_to_previous_row_start;
                if distance_to_previous_row_start > DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW {
                    pixels += row_height;
                }
                pixels
            } else {
                let mut pixels = number_of_scroll_steps * row_height + distance_to_next_row_start;
                if distance_to_next_row_start > DISCRETE_SCROLL_DISTANCE_THRESHOLD_TO_ROW {
                    pixels -= row_height;
                }
                pixels
            };
            self.start_scroll_animation(pixels);

            true
        }

        #[template_callback]
        fn swipe(&self, _dx: f64, dy: f64) {
            let pixels = -dy;
            self.start_scroll_animation(pixels);
        }
    }
}

glib::wrapper! {
    pub struct YearView(ObjectSubclass<imp::YearView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
