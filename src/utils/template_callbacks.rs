//! Collection of template callbacks.

/// Struct used as a collection of template callbacks.
pub struct TemplateCallbacks {}

#[gtk::template_callbacks(functions)]
impl TemplateCallbacks {
    /// Inverts the given boolean.
    #[template_callback]
    pub fn invert_boolean(boolean: bool) -> bool {
        !boolean
    }

    /// Returns `true` when the given string is not empty.
    #[template_callback]
    pub fn string_not_empty(string: &str) -> bool {
        !string.is_empty()
    }
}
