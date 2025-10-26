use adw::prelude::*;

/// Extensions trait for types with a `child` property.
pub trait ChildPropertyExt {
    /// The child of this widget, is any.
    fn child_property(&self) -> Option<gtk::Widget>;

    /// Set the child of this widget.
    fn set_child_property(&self, child: Option<&impl IsA<gtk::Widget>>);

    /// Get the child if it is of the proper type, or construct it with the
    /// given function and set is as the child of this widget before returning
    /// it.
    fn child_or_else<W>(&self, f: impl FnOnce() -> W) -> W
    where
        W: IsA<gtk::Widget>,
    {
        if let Some(child) = self.child_property().and_downcast() {
            child
        } else {
            let child = f();
            self.set_child_property(Some(&child));
            child
        }
    }

    // Get the child if it is of the proper type, or construct it with its
    // `Default` implementation and set is as the child of this widget before
    // returning it.
    // fn child_or_default<W>(&self) -> W
    // where
    //     W: IsA<gtk::Widget> + Default,
    // {
    //     self.child_or_else(Default::default)
    // }
}
