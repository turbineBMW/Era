//! Collection of GTK template callbacks.

/// Struct used as a collection of GTK template callbacks.
pub struct TemplateCallbacks {}

#[gtk::template_callbacks(functions)]
impl TemplateCallbacks {
    /// Inverts the given boolean.
    #[template_callback]
    pub fn invert_boolean(boolean: bool) -> bool {
        !boolean
    }
}
