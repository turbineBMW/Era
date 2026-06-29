# Shared Components

This directory (`src/widgets/components/`) contains reusable custom widgets used across different views and dialogs in Era.

## Error Handling

### ErrorDialog (`error_dialog.rs`)
A simple dialog that displays an error message in a read-only `gtk::TextView`.
- **Integration:** Typically opened from `adw::Toast` notifications (via the "Details" button) throughout the app using per-dialog `show-error` actions.
- **Features:** Includes a convenient "Copy" button that places the error text into the system clipboard for easy bug reporting.

## Loading States

These widgets encapsulate standard visual loading indicators for async operations.

### LoadingButton & LoadingButtonRow
Buttons that can transition into a loading state (e.g., showing a spinner) while an async operation (like creating a calendar or event) is in progress, preventing double-clicks and providing immediate user feedback.

### LoadingBin
A bin widget with a loading state, used to obscure or replace its child content with a loading spinner when data is being fetched or processed.
