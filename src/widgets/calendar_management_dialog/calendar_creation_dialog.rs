use std::{cell::RefCell, sync::Mutex};

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::{CalendarCreationFlow, CalendarCreationItem, Collection, prelude::*};
use gdk::RGBA;
use glib::clone;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{debug, warn};

use crate::{
    spawn,
    widgets::components::{ErrorDialog, LoadingButton},
};

/// Answers to one calendar creation flow.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CalendarCreationAnswers {
    pub flow_id: String,
    pub parameters: Map<String, Value>,
}

#[derive(Debug)]
enum ParameterSource {
    Static {
        id: String,
        value: String,
    },
    Text {
        id: String,
        row: adw::EntryRow,
    },
    Color {
        id: String,
        button: gtk::ColorDialogButton,
    },
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/gitlab/TitouanReal/Era/calendar_creation_dialog.ui")]
    #[properties(wrapper_type = super::CalendarCreationDialog)]
    pub struct CalendarCreationDialog {
        #[property(get, set, construct_only)]
        collection: RefCell<Option<Collection>>,

        #[template_child]
        toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        flows_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        create: TemplateChild<LoadingButton>,
        #[template_child]
        flow_page: TemplateChild<adw::PreferencesPage>,

        flow_id: RefCell<String>,
        groups: Mutex<Vec<adw::PreferencesGroup>>,
        parameter_sources: Mutex<Vec<ParameterSource>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarCreationDialog {
        const NAME: &'static str = "CalendarCreationDialog";
        type Type = super::CalendarCreationDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("calendar-creation-dialog.create", None, |obj, _, _| {
                let imp = obj.imp();
                spawn!(clone!(
                    #[weak]
                    imp,
                    async move {
                        imp.create_calendar().await;
                    }
                ));
            });
            klass.add_binding_action(
                gdk::Key::S,
                gdk::ModifierType::CONTROL_MASK,
                "calendar-creation-dialog.create",
            );

            klass.install_action(
                "calendar-creation-dialog.show-error",
                Some(&String::static_variant_type()),
                |obj, _, param| {
                    let error_message = &param
                        .and_then(glib::Variant::get::<String>)
                        .expect("The parameter should be a string");
                    let dialog = ErrorDialog::new(error_message);
                    dialog.present(Some(obj));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CalendarCreationDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let collection = self.obj().collection().unwrap();

            let flows = collection.parsed_calendar_creation_flows().unwrap().flows;

            if flows.is_empty() {
                panic!("No calendar creation flows available");
            }

            for flow in flows {
                let row = adw::ActionRow::builder()
                    .title(&flow.title)
                    .subtitle(&flow.description)
                    .build();
                row.set_activatable(true);
                row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));

                let obj = self.obj().clone();
                row.connect_activated(move |_| {
                    obj.imp().open_flow_page(flow.clone());
                });

                self.flows_group.add(&row);
            }
        }
    }

    impl WidgetImpl for CalendarCreationDialog {}
    impl AdwDialogImpl for CalendarCreationDialog {}

    impl CalendarCreationDialog {
        fn open_flow_page(&self, flow: CalendarCreationFlow) {
            let mut groups = self.groups.lock().unwrap();
            for group in groups.iter() {
                group.unparent();
            }
            let mut parameters = self.parameter_sources.lock().unwrap();
            parameters.clear();
            self.toast_overlay.dismiss_all();

            self.flow_id.replace(flow.id);

            for section in &flow.sections {
                let group = adw::PreferencesGroup::builder()
                    .title(&section.title)
                    .description(&section.description)
                    .build();

                groups.push(group.clone());

                for item in &section.items {
                    match item {
                        CalendarCreationItem::Information { id, title, value } => {
                            let row = adw::ActionRow::builder()
                                .title(title)
                                .subtitle(value)
                                .build();

                            let copy_button = gtk::Button::builder()
                                .icon_name("edit-copy-symbolic")
                                .tooltip_text("Copy")
                                .valign(gtk::Align::Center)
                                .css_classes(["flat"])
                                .build();

                            let value_clone = value.clone();
                            copy_button.connect_clicked(clone!(
                                #[weak(rename_to = imp)]
                                self,
                                move |button| {
                                    button.clipboard().set_text(&value_clone);

                                    imp.toast_overlay.dismiss_all();
                                    let toast = adw::Toast::new("Copied to clipboard");
                                    imp.toast_overlay.add_toast(toast);
                                }
                            ));
                            row.add_suffix(&copy_button);

                            parameters.push(ParameterSource::Static {
                                id: id.clone(),
                                value: value.clone(),
                            });

                            group.add(&row);
                        }
                        CalendarCreationItem::Input { id, title } => {
                            let row = adw::EntryRow::builder().title(title).build();

                            parameters.push(ParameterSource::Text {
                                id: id.clone(),
                                row: row.clone(),
                            });
                            group.add(&row);
                        }
                        CalendarCreationItem::Color { id, title } => {
                            let rgba = RGBA::RED;
                            let dialog = gtk::ColorDialog::builder()
                                .modal(true)
                                .with_alpha(false)
                                .build();
                            let color_button = gtk::ColorDialogButton::builder()
                                .dialog(&dialog)
                                .rgba(&rgba)
                                .valign(gtk::Align::Center)
                                .build();
                            let row = adw::ActionRow::builder()
                                .title(title)
                                .activatable_widget(&color_button)
                                .build();
                            row.add_suffix(&color_button);

                            parameters.push(ParameterSource::Color {
                                id: id.clone(),
                                button: color_button,
                            });
                            group.add(&row);
                        }
                    };
                }

                self.flow_page.add(&group);
            }

            self.navigation_view.push_by_tag("flow-details");
        }

        async fn create_calendar(&self) {
            self.create.set_is_loading(true);

            let collection = self.obj().collection().unwrap();
            let answer = {
                let flow_id = self.flow_id.borrow();
                let mut parameters = Map::new();

                for source in self.parameter_sources.lock().unwrap().iter() {
                    match source {
                        ParameterSource::Static { id, value } => {
                            parameters.insert(id.to_string(), Value::String(value.to_string()));
                        }
                        ParameterSource::Text { id, row } => {
                            parameters
                                .insert(id.to_string(), Value::String(row.text().to_string()));
                        }
                        ParameterSource::Color { id, button } => {
                            let color: &gdk::RGBA = &button.rgba();

                            parameters.insert(
                                id.to_string(),
                                Value::String(
                                    {
                                        fn channel_to_u8(channel: f32) -> u8 {
                                            (channel.clamp(0.0, 1.0) * 255.0).round() as u8
                                        }
                                        format!(
                                            "#{:02x}{:02x}{:02x}",
                                            channel_to_u8(color.red()),
                                            channel_to_u8(color.green()),
                                            channel_to_u8(color.blue())
                                        )
                                    }
                                    .to_string(),
                                ),
                            );
                        }
                    }
                }

                let answer = CalendarCreationAnswers {
                    flow_id: flow_id.to_string(),
                    parameters,
                };

                serde_json::to_string(&answer)
                    .expect("Calendar creation answer should always serialize")
            };

            match collection.try_create_calendar_future(&answer).await {
                Ok(calendar) => {
                    debug!("Calendar created: {}", calendar.uri().unwrap());
                    self.obj().close();
                }
                Err(error) => {
                    self.create.set_is_loading(false);

                    warn!("Failed to create calendar: {error}");
                    let message = error.message().to_string();
                    self.toast_overlay.dismiss_all();
                    let toast = adw::Toast::new("An error occurred");
                    toast.set_button_label(Some("Details"));
                    toast.set_action_name(Some("calendar-creation-dialog.show-error"));
                    toast.set_action_target(Some(&message));
                    self.toast_overlay.add_toast(toast);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CalendarCreationDialog(ObjectSubclass<imp::CalendarCreationDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl CalendarCreationDialog {
    pub fn new(collection: &Collection) -> Self {
        glib::Object::builder()
            .property("collection", collection)
            .build()
    }
}
