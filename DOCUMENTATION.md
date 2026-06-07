# Contributing to Era

This document was generated with Claude Opus 4.6.

This document describes the internal architecture of Era for contributors who want to understand the codebase without having to reverse-engineer it. It is intentionally concise — when you need details, read the code.

---

## Table of Contents

- [Overview](#overview)
- [Tech Stack](#tech-stack)
- [Building](#building)
  - [Installing Clepsydre locally](#installing-clepsydre-locally)
  - [Flatpak (development)](#flatpak-development)
  - [Native build](#native-build)
- [Project Layout](#project-layout)
- [Architecture](#architecture)
  - [Entry point & Application](#entry-point--application)
  - [Clepsydre — the view-model](#clepsydre--the-view-model)
  - [Window & navigation](#window--navigation)
  - [Views](#views)
  - [Dialogs](#dialogs)
    - [CreateEventDialog](#createeventdialog)
    - [EventDetailsDialog](#eventdetailsdialog)
    - [SearchDialog](#searchdialog)
    - [CalendarManagerDialog](#calendarmanagerdialog)
    - [QrCodeDialog](#qrcodedialog)
    - [ErrorDialog](#errordialog)
  - [Reusable components](#reusable-components)
  - [Utility modules](#utility-modules)
  - [System settings](#system-settings)
  - [UI definitions](#ui-definitions)
- [Patterns & conventions](#patterns--conventions)

---

## Overview

Era is a GTK 4 / libadwaita calendar application written in Rust. It is **backend-agnostic**: all calendar data access goes through [Clepsydre](https://gitlab.gnome.org/TitouanReal/clepsydre), a GObject-based abstraction layer. The application currently uses the EDS (Evolution Data Server) backend, but switching to another Clepsydre implementation (e.g. [ccmd](https://gitlab.gnome.org/TitouanReal/ccmd)) is a one-line change in `application.rs`.

Era uses Clepsydre as a **view-model**. The UI is reactive on the data Clepsydre exposes (GObject properties, list models, signals). User actions (creating events, managing calendars, searching…) are forwarded to Clepsydre which talks to the actual backend.

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2024) |
| Toolkit | GTK 4 (≥ 4.20), libadwaita (≥ 1.8) |
| View-model | Clepsydre (+ clepsydre-eds) |
| Build system | Meson (project) + Cargo (Rust) |
| Task runner | [just](https://github.com/casey/just) |
| UI templates | Blueprint (`.blp` → compiled to `.ui` XML) |
| i18n | gettext |
| Date/time | jiff |
| Portals | ashpd |

## Building

### Installing Clepsydre locally

For native (non-Flatpak) development you need the Clepsydre C libraries installed on your system. The repo ships a `just` recipe that automates this:

```sh
just install-clepsydre
```

This reads the exact commit pinned in the Flatpak manifest (`build-aux/io.gitlab.TitouanReal.Era.Devel.json`), clones the Clepsydre repo, builds `libclepsydre` and `libclepsydre-eds` with Meson, and installs them under `/usr`.

### Flatpak (development)

The Flatpak manifest lives at `build-aux/io.gitlab.TitouanReal.Era.Devel.json`. It pulls libical, Evolution Data Server, and Clepsydre as module dependencies before building Era itself. Use GNOME Builder or `flatpak-builder` as usual.

### Native build

After installing Clepsydre:

```sh
meson setup builddir
meson compile -C builddir
```

## Project Layout

```
.
├── build-aux/          # Flatpak manifest
├── data/
│   ├── icons/          # App icons
│   ├── resources/
│   │   ├── icons/      # Symbolic icons bundled as GResource
│   │   ├── ui/         # Blueprint UI templates (mirrors src/widgets/ structure)
│   │   ├── style.css
│   │   └── resources.gresource.xml
│   ├── *.desktop.in.in
│   ├── *.gschema.xml.in
│   ├── *.metainfo.xml.in.in
│   └── *.service.in
├── po/                 # Translations
├── src/
│   ├── main.rs
│   ├── application.rs
│   ├── config.rs.in    # Build-time constants (app id, version…)
│   ├── system_settings.rs
│   ├── utils/
│   └── widgets/
│       ├── window.rs
│       ├── views/           # Year view, Month view
│       ├── calendar_manager_dialog/
│       ├── create_event_dialog/
│       ├── search_dialog/
│       ├── event_details_dialog.rs
│       ├── qr_code_dialog.rs
│       └── components/      # Shared widgets (LoadingButton, ErrorDialog…)
├── justfile
├── meson.build
└── Cargo.toml
```

## Architecture

### Entry point & Application

`main.rs` sets up logging (via `tracing`), gettext, and GResource loading, then creates and runs the `Application`.

`application.rs` defines `Application`, a subclass of `adw::Application`. On construction it:

- Initialises `SystemSettings` (system-level preferences).
- Creates the Clepsydre `Manager` (currently `clepsydre_eds::Manager`). This is the single entry point to all backend data.
- Stores the current date (year / month / day).
- Registers GActions (`quit`, `about`).

`Application` is a singleton accessible anywhere via `Application::default()`.

### Clepsydre — the view-model

The `Manager` (from Clepsydre) is the **central reactive data source**. It exposes:

- Whether the backend is available (`is_backend_available` property + notify signal).
- A list-model of calendars, collections, events.
- Async methods to create/remove/search events and calendars.

Widgets bind to Clepsydre GObject properties and connect to their notify signals to stay up-to-date. Mutations (create event, remove calendar…) are performed by calling async methods on Clepsydre objects (`Calendar`, `Event`, `Collection`), using the `_future` variants and awaiting them.

### Window & navigation

`Window` (`widgets/window.rs`, template `window.blp`) is the main application window. It contains:

- A top-level **`Stack`** that switches between a "no backend" status page and the calendar view, reacting to `Manager::is_backend_available`.
- An **`Adw.MultiLayoutView`** with two layouts:
  - **Wide** — `Adw.OverlaySplitView` with a sidebar + an `Adw.ViewStack` (Year / Month / Week / Days / Agenda tabs).
  - **Narrow** — a `Stack` that navigates linearly: Year → Month → Days.
- **`Adw.Breakpoint`** rules switch between wide and narrow layouts based on window width.

Window-level GActions:

| Action | Shortcut | Effect |
|---|---|---|
| `win.search-events` | `Ctrl+F` | Opens `SearchDialog` |
| `win.manage-calendars` | `F8` / `Ctrl+Alt+M` | Opens `CalendarManagerDialog` |
| `win.create-event` | `Ctrl+N` | Opens `CreateEventDialog` |

### Views

Views live under `src/widgets/views/`.

**YearView** — a custom scrollable widget that shows a vertical list of year rows, each containing 12 month cells. Scrolling is implemented manually with kinetic deceleration and snap-to-row animations. Clicking a month cell emits `month_clicked`, which the window uses to navigate to the month view. Adapts its grid layout via a `YearViewStyling` enum (narrow / medium / wide) driven by breakpoints.

**MonthView** — displays week rows for a given ISO week. Each row contains day cells. Clicking a day emits `day_clicked`, used by the window to open the days view (in narrow layout). Week rows are laid out manually in `size_allocate`.

### Dialogs

All dialogs are `adw::Dialog` subclasses. Errors from async Clepsydre calls are surfaced as `adw::Toast` notifications; each toast has a "Details" button that opens an `ErrorDialog`.

#### CreateEventDialog

`src/widgets/create_event_dialog/`

Form dialog for creating a new event. Fields: name, location, conference URL, description, schedule type (timed or all-day), start/end date-time. The "Create" button calls `Calendar::try_create_event_future` and closes the dialog on success.

Notable sub-widgets:

- **`CalendarComboRow`** — an `adw::ComboRow` that lets the user pick which calendar to create the event in. It flattens all collections' calendars into a single sorted list, grouped by collection via section headers (`CalendarComboRowHeader`). Each item shows the calendar's color indicator and name.
- **`DateTimePickerGroup`** — a preferences group containing a `DatePickerRow` (date entry + `gtk::Calendar` popover) and a time entry with hour/minute spin buttons. Also includes a timezone picker button that opens `TimeZonePickerDialog`, a searchable list of all IANA timezones. The group exposes a single `date-time` GObject property that the dialog binds to.

Validation logic (implemented as template callbacks) disables the create button when the name is empty or the end is before the start.

#### EventDetailsDialog

`src/widgets/event_details_dialog.rs`

Displays the details of an existing event. Uses an `adw::NavigationView` with two pages:

1. **Details page** — read-only view of the event name, timeframe, location, conference link, and description. Action buttons:
   - **Share** — exports the event as an `.ics` file via the `ashpd` file-chooser portal.
   - **QR Code** — opens a `QrCodeDialog` with the event data.
   - **Map** — opens the location as a `geo:` URI through the `ashpd` open-URI portal.
   - **Join** — opens the conference URL (only shown when the URL is parseable).
   - **Edit** — pushes the editor page.
   - **Remove** — calls `Event::try_remove_future`; the dialog auto-closes via a `connect_removed` signal handler.
2. **Editor page** — editable entry rows for name, location, conference, and description. (Save is not yet implemented.)

#### SearchDialog

`src/widgets/search_dialog/`

A dialog with a search entry that calls `Manager::search_events_future` on input. Results are shown in a `gtk::ListView`; each row is an `EventRow` displaying the event name, calendar color bar, and start/end times. Clicking a result opens an `EventDetailsDialog`.

The dialog shows a loading spinner while the search is in flight and switches to the results list when done.

#### CalendarManagerDialog

`src/widgets/calendar_manager_dialog/`

An `adw::NavigationView`-based dialog for managing calendars. It has three levels:

1. **`CollectionsListPage`** — lists all collections from the Clepsydre manager. Each collection is rendered by a `CollectionRow`, which shows the collection name, a sorted list of its calendars (as `CalendarRow` widgets), and a button to create a new calendar in that collection.
2. **`CalendarDetailsPage`** — pushed when a calendar row is activated (via the `calendar-manager.show-calendar-subpage` action, which passes the calendar URI). Shows an editable name entry (changes are saved on focus-out via `Calendar::try_set_name_future`) and a remove button (`Calendar::try_remove_future`). Reacts to the `removed` signal to auto-pop back.
3. **`CalendarCreationDialog`** — opened from a collection row. A small form with a name entry and a `gtk::ColorDialogButton`. Calls `Collection::try_create_calendar_future`.

`CalendarRow` also supports toggling calendar visibility via `Calendar::try_set_visible_future`, with a loading spinner replacing the color circle while the operation is in progress.

#### QrCodeDialog

`src/widgets/qr_code_dialog.rs`

Receives a URL string, generates a QR code SVG using the `qrcode` crate, converts it to a `gdk::Texture`, and displays it in a `gtk::Picture`. Used by `EventDetailsDialog` to share an event as a scannable code.

#### ErrorDialog

`src/widgets/components/error_dialog.rs`

A simple dialog that displays an error message in a read-only `gtk::TextView` with a "Copy" button that puts the text into the clipboard. Opened from toast "Details" buttons throughout the app via per-dialog `show-error` actions.

### Reusable components

Under `src/widgets/components/`:

- **`LoadingButton`** / **`LoadingButtonRow`** — buttons that can show a spinner while an async operation is in progress.
- **`LoadingBin`** — a bin widget with a loading state.
- **`ErrorDialog`** — see above.

### Utility modules

Under `src/utils/`:

| Module | Role |
|---|---|
| `macros` | `spawn!` macro — spawns a future on the default GLib main context. |
| `template_callbacks` | `TemplateCallbacks` — a collection of reusable Blueprint expression helpers (`not`, `both`, `either`, `ternary`, `string_empty`, `string_equals`, `is_some`, `is_none`, `is_zero`). Bound to UI templates. |
| `paintable_callbacks` | `PaintableCallbacks` — template callbacks that generate circle / bar paintables from a `gdk::RGBA` color, used to display calendar color indicators. |
| `child_property_ext` | `ChildPropertyExt` trait — convenience for widgets with a `child` property (get-or-create pattern). |

### System settings

`SystemSettings` (`system_settings.rs`) reads desktop preferences through the XDG Settings portal (`ashpd`):

- **Clock format** (12h / 24h) from `org.gnome.desktop.interface`.
- **First day of week** from `org.gnome.desktop.calendar`.

It exposes these as GObject properties and keeps them updated by listening to portal change signals.

### UI definitions

Blueprint files under `data/resources/ui/` mirror the widget tree in `src/widgets/`. They are compiled to GTK XML at build time and bundled as GResources. The resource prefix is `/io/gitlab/TitouanReal/Era`.

## Patterns & conventions

- **GObject subclassing** — every widget follows the gtk-rs `mod imp {}` + `glib::wrapper!` pattern.
- **Composite templates** — UI is defined in Blueprint, loaded with `#[template()]` and `bind_template()`.
- **Async via GLib main loop** — async operations use the `spawn!` macro and `clone!` with `#[weak]` references to avoid preventing finalisation.
- **Error handling in UI** — errors from Clepsydre are surfaced as `adw::Toast` notifications with a "Details" button that opens an `ErrorDialog`.
- **Actions** — feature entry points are modelled as GActions (`win.*`, `app.*`, per-dialog actions), with keyboard shortcuts bound in `class_init`.
- **Reactivity** — property bindings and `connect_*_notify` signals keep the UI in sync with Clepsydre data. The window reacts to backend availability changes by switching its visible stack page and force-closing open dialogs when the backend disappears.
