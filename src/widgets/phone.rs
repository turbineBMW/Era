//! The phone build (the `phone` feature, fork-only), for phone shells such as
//! omarchy-mobile's, where Era is called Calendar: one page at a time,
//! nothing to press at the top, and the controls at the bottom, in thumb
//! reach.
//!
//! It rearranges the narrow layout rather than replacing it, so every handler
//! stays upstream's:
//! - The narrow layout always, also in landscape, where the window's
//!   breakpoint would switch to the medium one.
//! - The header bars show only the title: no menu, search, back arrow or
//!   window buttons. Search joins Today, ‹ ›, the calendars and New Event in
//!   the bottom bar, which is the colour of the shell's bars.
//! - Search, Manage Calendars, About and the settings are the app's menu (the
//!   menubar, which the phone shell shows from its home bar).
//! - `app.go-back`, the shell's back gesture, closes an open dialog, else goes
//!   back a page (agenda to month to year); disabled on the first page, so
//!   the shell goes home.
//! - Dialogs are centred pop-ups that close on a tap outside, with their
//!   header's buttons (Cancel, Create, an event's menu) moved to a bar at the
//!   bottom.

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use super::{Window, window::Styling};
use crate::preferences::PreferencesPanel;

/// What the phone calls Era.
pub const NAME: &str = "Calendar";

/// Bars are the colour of the shell's bars (the theme's dark background,
/// libadwaita's header bar colour), with no line between them and the page.
const CSS: &str = "
actionbar.phone-bar > revealer > box {
  background-color: var(--headerbar-bg-color);
  box-shadow: none;
  padding: 6px 12px;
}
actionbar.phone-bar button, .phone-dialog-bar button { min-height: 44px; min-width: 44px; }
.phone-dialog-bar { padding: 8px 12px; background-color: var(--headerbar-bg-color); }
toolbarview > .top-bar { background-color: var(--headerbar-bg-color); }
";

/// A pop-up's size: 90 % of the window's width up to 640 px, and 2/3 of its
/// height up to 800 px, or 90 % when the window is short (landscape); as
/// omarchy-mobile's own apps size theirs.
const POPUP_MAX_WIDTH: i32 = 640;
const POPUP_MAX_HEIGHT: i32 = 800;
const SHORT_SCREEN: i32 = 600;
/// A dialog asking for at least this much room is a sheet (a pop-up); a
/// smaller one keeps its own size.
const SHEET_HEIGHT: i32 = 400;

/// Marks a widget this module has already rearranged.
const ADAPTED: &str = "phone-adapted";

pub fn setup(window: &Window, navigation: &adw::NavigationView) {
    window.set_title(Some(NAME));
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("a display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 2,
    );

    // One page at a time, also in landscape.
    window.set_styling(Styling::Narrow);
    window.connect_styling_notify(|window| {
        if window.styling() != Styling::Narrow {
            window.set_styling(Styling::Narrow);
        }
    });

    for tag in ["year", "month", "agenda"] {
        if let Some(page) = navigation.find_page(tag) {
            arrange_page(page.upcast_ref());
        }
    }
    setup_menu(window);
    setup_back(window, navigation);
    setup_dialogs(window);
}

/// A page's header bar shows only its title; its bottom bar gets Search.
fn arrange_page(page: &gtk::Widget) {
    for header in descendants::<adw::HeaderBar>(page) {
        title_only(&header);
        for button in packed(&header) {
            button.set_visible(false);
        }
    }
    for bar in descendants::<gtk::ActionBar>(page) {
        bar.add_css_class("phone-bar");
        let search = gtk::Button::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search")
            .action_name("win.search-events")
            .build();
        bar.pack_end(&search);
    }
}

/// Only the title: no back arrow or window buttons.
fn title_only(header: &adw::HeaderBar) {
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    header.set_show_back_button(false);
}

/// The app's menu, for the shell: what the header buttons and the main menu
/// held, and the settings, last.
fn setup_menu(window: &Window) {
    let settings = gio::SimpleAction::new("phone-settings", None);
    settings.connect_activate(glib::clone!(
        #[weak]
        window,
        move |_, _| show_settings(&window)
    ));
    window.add_action(&settings);

    let menu = gio::Menu::new();
    let calendar = gio::Menu::new();
    calendar.append(Some("Search"), Some("win.search-events"));
    calendar.append(Some("Manage Calendars"), Some("win.manage-calendars"));
    menu.append_section(None, &calendar);
    let about = gio::Menu::new();
    about.append(Some(&format!("About {NAME}")), Some("app.about"));
    menu.append_section(None, &about);
    let last = gio::Menu::new();
    last.append(Some("Settings"), Some("win.phone-settings"));
    menu.append_section(None, &last);
    if let Some(app) = window.application() {
        app.set_menubar(Some(&menu));
    }
    window.set_show_menubar(false);
}

/// The settings (the sidebar's popover on a desktop), as a centred modal.
fn show_settings(window: &Window) {
    let header = adw::HeaderBar::new();
    title_only(&header);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&glib::Object::new::<PreferencesPanel>()));
    let dialog = adw::Dialog::builder()
        .title("Settings")
        .child(&toolbar)
        .content_width(360)
        .build();
    dialog.present(Some(window));
}

/// `app.go-back`: an open dialog closes, else the navigation goes back a
/// page; disabled on the first page with no dialog, so the shell goes home.
fn setup_back(window: &Window, navigation: &adw::NavigationView) {
    let Some(app) = window.application() else {
        return;
    };
    let action = gio::SimpleAction::new("go-back", None);
    action.connect_activate(glib::clone!(
        #[weak]
        window,
        #[weak]
        navigation,
        move |_, _| {
            if let Some(dialog) = window.visible_dialog() {
                dialog.close();
            } else {
                navigation.pop();
            }
        }
    ));
    let update = glib::clone!(
        #[weak]
        window,
        #[weak]
        navigation,
        #[weak]
        action,
        move || {
            let can_pop = navigation
                .visible_page()
                .is_some_and(|page| navigation.previous_page(&page).is_some());
            action.set_enabled(can_pop || window.visible_dialog().is_some());
        }
    );
    update();
    let u = update.clone();
    navigation.connect_visible_page_notify(move |_| u());
    window.connect_visible_dialog_notify(move |_| update());
    app.add_action(&action);
}

/// Every dialog, as it is presented, becomes a phone dialog.
fn setup_dialogs(window: &Window) {
    window.dialogs().connect_items_changed(glib::clone!(
        #[weak]
        window,
        move |model, position, _, added| {
            for i in position..position + added {
                if let Some(dialog) = model.item(i).and_downcast::<adw::Dialog>() {
                    adapt_dialog(&window, &dialog);
                }
            }
        }
    ));
}

/// Centred, sized as a pop-up when it is a sheet, closed by a tap outside,
/// and with its header buttons at the bottom. Pages a dialog pushes later
/// (a calendar's details) get the same when they show.
fn adapt_dialog(window: &Window, dialog: &adw::Dialog) {
    if dialog.has_css_class(ADAPTED) {
        return;
    }
    dialog.add_css_class(ADAPTED);
    dialog.set_presentation_mode(adw::DialogPresentationMode::Floating);
    if dialog.content_height() >= SHEET_HEIGHT {
        fit_popup(window, dialog);
    }
    rearrange_headers(dialog.upcast_ref());
    for navigation in descendants::<adw::NavigationView>(dialog.upcast_ref()) {
        navigation.connect_visible_page_notify(|navigation| {
            if let Some(page) = navigation.visible_page() {
                rearrange_headers(page.upcast_ref());
            }
        });
    }
    // The sheet and its dimming are built when the dialog is first shown.
    glib::idle_add_local_once(glib::clone!(
        #[weak]
        dialog,
        move || {
            rearrange_headers(dialog.upcast_ref());
            close_on_tap_outside(&dialog);
        }
    ));
}

fn fit_popup(window: &Window, dialog: &adw::Dialog) {
    let fit = glib::clone!(
        #[weak]
        window,
        #[weak]
        dialog,
        move || {
            let (width, height) = (window.width(), window.height());
            if width <= 0 || height <= 0 {
                return;
            }
            dialog.set_content_width((width * 9 / 10).min(POPUP_MAX_WIDTH));
            dialog.set_content_height(if height < SHORT_SCREEN {
                height * 9 / 10
            } else {
                (height * 2 / 3).min(POPUP_MAX_HEIGHT)
            });
        }
    );
    fit();
    // The window's default size follows its actual size: fit again when the
    // phone turns.
    let fit_width = fit.clone();
    let a = window.connect_default_width_notify(move |_| fit_width());
    let b = window.connect_default_height_notify(move |_| fit());
    let handlers = std::cell::RefCell::new(Some((a, b)));
    dialog.connect_closed(glib::clone!(
        #[weak]
        window,
        move |_| {
            if let Some((a, b)) = handlers.take() {
                window.disconnect(a);
                window.disconnect(b);
            }
        }
    ));
}

/// The header bars under `widget` show only their title, and the buttons
/// packed into them move to a bar at the bottom of their toolbar view:
/// start ones on the left, end ones on the right.
fn rearrange_headers(widget: &gtk::Widget) {
    for header in descendants::<adw::HeaderBar>(widget) {
        title_only(&header);
        if header.has_css_class(ADAPTED) {
            continue;
        }
        header.add_css_class(ADAPTED);
        let (start, end) = packed_sides(&header);
        if start.is_empty() && end.is_empty() {
            continue;
        }
        let Some(toolbar) = header
            .ancestor(adw::ToolbarView::static_type())
            .and_downcast::<adw::ToolbarView>()
        else {
            continue;
        };
        let bar = gtk::CenterBox::new();
        bar.add_css_class("phone-dialog-bar");
        let left = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let right = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for child in start {
            unparent_from_box(&child);
            left.append(&child);
        }
        // End children are packed right to left; keep their visual order.
        for child in end.into_iter().rev() {
            unparent_from_box(&child);
            right.append(&child);
        }
        bar.set_start_widget(Some(&left));
        bar.set_end_widget(Some(&right));
        toolbar.add_bottom_bar(&bar);
    }
}

fn unparent_from_box(child: &gtk::Widget) {
    if let Some(parent) = child.parent().and_downcast::<gtk::Box>() {
        parent.remove(child);
    }
}

/// The widgets packed at the start and at the end of a header bar. Inside,
/// a header bar keeps them in two boxes, `start` and `end`, beside its title.
fn packed_sides(header: &adw::HeaderBar) -> (Vec<gtk::Widget>, Vec<gtk::Widget>) {
    let mut start = Vec::new();
    let mut end = Vec::new();
    for container in descendants::<gtk::Box>(header.upcast_ref()) {
        let side = if container.has_css_class("start") {
            &mut start
        } else if container.has_css_class("end") {
            &mut end
        } else {
            continue;
        };
        let mut child = container.first_child();
        while let Some(widget) = child {
            child = widget.next_sibling();
            // The window buttons have their own box; skip it.
            if widget.css_name() != "windowcontrols" {
                side.push(widget);
            }
        }
    }
    (start, end)
}

/// Every widget packed into a header bar.
fn packed(header: &adw::HeaderBar) -> Vec<gtk::Widget> {
    let (mut start, end) = packed_sides(header);
    start.extend(end);
    start
}

/// A press outside the dialog closes it, as Escape does. Outside is the
/// floating sheet's `dimming`, which covers the window around the card.
fn close_on_tap_outside(dialog: &adw::Dialog) {
    let Some(dimming) = find_css(dialog.upcast_ref(), "dimming") else {
        return;
    };
    let gesture = gtk::GestureClick::builder()
        .button(0)
        .propagation_phase(gtk::PropagationPhase::Capture)
        .build();
    gesture.connect_pressed(glib::clone!(
        #[weak]
        dialog,
        move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            dialog.close();
        }
    ));
    dimming.add_controller(gesture);
}

/// The first widget under `widget` whose CSS name is `name`.
fn find_css(widget: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    let mut child = widget.first_child();
    while let Some(c) = child {
        if c.css_name() == name {
            return Some(c);
        }
        if let Some(found) = find_css(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

/// Every widget of type `T` under `widget`.
fn descendants<T: IsA<gtk::Widget>>(widget: &gtk::Widget) -> Vec<T> {
    let mut found = Vec::new();
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(t) = c.downcast_ref::<T>() {
            found.push(t.clone());
        }
        found.extend(descendants::<T>(&c));
        child = c.next_sibling();
    }
    found
}
