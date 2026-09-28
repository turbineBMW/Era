use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{Calendar, prelude::*};
use gio::prelude::ListModelExt;
use tracing::warn;

mod sidebar_calendar_row;

use crate::utils::TemplateCallbacks;

use self::sidebar_calendar_row::SidebarCalendarRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "data/resources/ui/sidebar/sidebar.blp")]
    #[properties(wrapper_type = super::Sidebar)]
    pub struct Sidebar {}

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            SidebarCalendarRow::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
            TemplateCallbacks::bind_template_callbacks(klass);

            klass.set_css_name("sidebar");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Sidebar {}
    impl WidgetImpl for Sidebar {}
    impl BinImpl for Sidebar {}

    #[gtk::template_callbacks]
    impl Sidebar {
        #[template_callback]
        async fn toggle_calendar_visibility(list_view: gtk::ListView, position: u32) {
            let calendar = list_view
                .model()
                .unwrap()
                .item(position)
                .unwrap()
                .downcast::<Calendar>()
                .unwrap();
            let visible = calendar.is_visible();
            // TODO: Show error to the user
            match calendar.try_set_visible_future(!visible).await {
                Ok(()) => (),
                Err(error) => warn!("Failed to set calendar visibility: {error}"),
            }
        }
    }
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
