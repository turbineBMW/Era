//! The settings popover in the sidebar's header.

use adw::{prelude::*, subclass::prelude::*};

use super::{DEFAULT_VIEW, VIEWS, WEEK_STARTS_ON_SUNDAY};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "data/resources/ui/preferences/preferences_panel.blp")]
    pub struct PreferencesPanel {
        #[template_child]
        default_view_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        week_starts_on_sunday_row: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesPanel {
        const NAME: &'static str = "PreferencesPanel";
        type Type = super::PreferencesPanel;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesPanel {
        fn constructed(&self) {
            self.parent_constructed();

            let Some(settings) = super::super::settings() else {
                self.obj().set_sensitive(false);
                return;
            };

            settings
                .bind(
                    WEEK_STARTS_ON_SUNDAY,
                    &*self.week_starts_on_sunday_row,
                    "active",
                )
                .build();

            settings
                .bind(DEFAULT_VIEW, &*self.default_view_row, "selected")
                .mapping(|variant, _| {
                    let view = variant.str()?;
                    let index = VIEWS.iter().position(|v| *v == view)?;
                    Some((index as u32).to_value())
                })
                .set_mapping(|value, _| {
                    let index = value.get::<u32>().ok()?;
                    VIEWS.get(index as usize).map(|view| view.to_variant())
                })
                .build();
        }
    }

    impl WidgetImpl for PreferencesPanel {}
    impl BinImpl for PreferencesPanel {}
}

glib::wrapper! {
    pub struct PreferencesPanel(ObjectSubclass<imp::PreferencesPanel>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
