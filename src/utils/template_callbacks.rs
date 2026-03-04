//! Collection of template callbacks.

use clepsydre::ParticipationStatus;
use glib::Object;

/// Struct used as a collection of template callbacks.
pub struct TemplateCallbacks {}

#[gtk::template_callbacks(functions)]
impl TemplateCallbacks {
    /// Inverts the given boolean.
    #[template_callback]
    pub fn not(boolean: bool) -> bool {
        !boolean
    }

    /// Applies a logical "and".
    #[template_callback]
    pub fn both(left: bool, right: bool) -> bool {
        left && right
    }

    /// Applies a logical "or".
    #[template_callback]
    pub fn either(left: bool, right: bool) -> bool {
        left || right
    }

    /// Applies a logical "and(not, not)".
    #[template_callback]
    pub fn neither(left: bool, right: bool) -> bool {
        !left && !right
    }

    /// Applies a logical "and(not, not, not)".
    #[template_callback]
    pub fn none_of_three(left: bool, middle: bool, right: bool) -> bool {
        !left && !middle && !right
    }

    /// Applies a ternary operator.
    #[template_callback]
    pub fn ternary(condition: bool, left: &str, right: &str) -> String {
        if condition {
            left.to_string()
        } else {
            right.to_string()
        }
    }

    /// Returns `true` when the given string is empty.
    #[template_callback]
    pub fn string_empty(string: &str) -> bool {
        string.is_empty()
    }

    /// Returns `true` when the given strings are equal.
    #[template_callback]
    pub fn string_equals(left: &str, right: &str) -> bool {
        left == right
    }

    /// Returns `true` when the given option object is some.
    #[template_callback]
    pub fn is_some(option: Option<Object>) -> bool {
        option.is_some()
    }

    /// Returns `true` when the given option object is none.
    #[template_callback]
    pub fn is_none(option: Option<Object>) -> bool {
        option.is_none()
    }

    /// Returns `true` when the given number is zero.
    #[template_callback]
    pub fn is_zero(int: u32) -> bool {
        int == 0
    }

    /// Returns `true` when the participation statuses are equals.
    #[template_callback]
    pub fn participation_status_equals(
        left: ParticipationStatus,
        right: ParticipationStatus,
    ) -> bool {
        left == right
    }
}
