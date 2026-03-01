//! Collection of template callbacks.

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
    pub fn both(first: bool, second: bool) -> bool {
        first && second
    }

    /// Applies a logical "or".
    #[template_callback]
    pub fn either(first: bool, second: bool) -> bool {
        first || second
    }

    /// Applies a logical "".
    #[template_callback]
    pub fn neither(first: bool, second: bool) -> bool {
        !first && !second
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
}
