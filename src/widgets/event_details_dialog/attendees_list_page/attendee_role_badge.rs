use std::cell::Cell;

use adw::{prelude::*, subclass::prelude::*};
use clepsydre::AttendeeRole;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::AttendeeRoleBadge)]
    pub struct AttendeeRoleBadge {
        label: gtk::Label,
        /// The role displayed by this badge.
        #[property(get, set = Self::set_role, explicit_notify, builder(AttendeeRole::Required))]
        role: Cell<AttendeeRole>,
    }

    impl Default for AttendeeRoleBadge {
        fn default() -> Self {
            Self {
                label: gtk::Label::default(),
                role: Cell::new(AttendeeRole::Required),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttendeeRoleBadge {
        const NAME: &'static str = "AttendeeRoleBadge";
        type Type = super::AttendeeRoleBadge;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("badge");
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AttendeeRoleBadge {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_child(Some(&self.label));
            self.update_badge();
        }
    }

    impl WidgetImpl for AttendeeRoleBadge {}
    impl BinImpl for AttendeeRoleBadge {}

    impl AttendeeRoleBadge {
        /// Set the role displayed by this badge.
        fn set_role(&self, role: AttendeeRole) {
            if self.role.get() == role {
                return;
            }

            self.role.set(role);
            self.update_badge();
            self.obj().notify_role();
        }

        /// Update the badge for the current state.
        fn update_badge(&self) {
            let obj = self.obj();
            let role = self.role.get();

            self.label.set_text(match role {
                AttendeeRole::Chair => "Chair",
                AttendeeRole::Required => "Required",
                AttendeeRole::Optional => "Optional",
                AttendeeRole::Nonparticipant => "Non Participant",
                _ => unreachable!(),
            });

            if role == AttendeeRole::Chair {
                obj.add_css_class("chair");
            } else {
                obj.remove_css_class("chair");
            }

            if role == AttendeeRole::Optional {
                obj.add_css_class("optional");
            } else {
                obj.remove_css_class("optional");
            }

            if role == AttendeeRole::Nonparticipant {
                obj.add_css_class("non-participant");
            } else {
                obj.remove_css_class("non-participant");
            }
        }
    }
}

glib::wrapper! {
    /// Inline widget displaying a badge with the role of an event attendee.
    pub struct AttendeeRoleBadge(ObjectSubclass<imp::AttendeeRoleBadge>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AttendeeRoleBadge {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
