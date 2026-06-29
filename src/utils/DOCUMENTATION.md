# Utilities & Helpers

This directory (`src/utils/`) contains reusable logic, macros, and GTK template callbacks that are shared across the application.

## Key Modules

### `macros.rs`
Contains application-wide macros.
- **`spawn!`:** A critical macro for the application's async architecture. It spawns a future on the default GLib main context, allowing async Clepsydre operations to run without blocking the GTK UI thread.

### Template Callbacks
These modules provide functions that can be bound directly in Blueprint UI templates.

- **`template_callbacks.rs`:** Reusable Blueprint expression helpers.
  - Includes logic for `not`, `both`, `either`, `ternary`, `string_empty`, `string_equals`, `is_some`, `is_none`, and `is_zero`.
- **`paintable_callbacks.rs`:** Helpers for visual rendering.
  - Generates circle or bar paintables from a `gdk::RGBA` color. These are heavily used to display calendar color indicators in the UI without manually drawing them in Rust.

### `child_property_ext.rs`
Provides the `ChildPropertyExt` trait, a convenience extension for widgets with a `child` property, streamlining the get-or-create pattern for UI components.

## Data & Filtering Helpers
Includes types like `attendee_type_filter.rs`, `participation_status_filter.rs`, and `datetime.rs` to handle calendar-specific data formatting and filtering.
