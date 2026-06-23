sass_bin = find_program('grass')

stylesheet = custom_target(
    'stylesheet',
    input: 'style.scss',
    output: 'style.css',
    command: [sass_bin, '@INPUT@', '@OUTPUT@'],
)

blueprints = custom_target(
    'blueprints',
    input: files(
        # Calendar Manager Dialog
        'ui/calendar_manager_dialog/calendar_creation_dialog.blp',
        'ui/calendar_manager_dialog/calendar_details_page.blp',
        'ui/calendar_manager_dialog/calendar_manager_dialog.blp',
        'ui/calendar_manager_dialog/calendar_row.blp',
        'ui/calendar_manager_dialog/collection_row.blp',
        'ui/calendar_manager_dialog/collections_list_page.blp',

        # Components
        # Calendar Combo Row
        'ui/components/calendar_combo_row/calendar_combo_row.blp',
        'ui/components/calendar_combo_row/calendar_combo_row_list_item.blp',
        # Error Dialog
        'ui/components/error_dialog.blp',
        # Loading Bin
        'ui/components/loading_bin.blp',
        # Loading Button Row
        'ui/components/loading_button_row.blp',
        # Timeframe Picker
        'ui/components/timeframe_picker/date_picker_row.blp',
        'ui/components/timeframe_picker/date_time_picker_group.blp',
        'ui/components/timeframe_picker/time_zone_picker_dialog.blp',
        'ui/components/timeframe_picker/timeframe_picker.blp',

        # Create Event Dialog
        'ui/create_event_dialog.blp',

        # Event Details Dialog
        'ui/event_details_dialog/attendee_list_row/attendee_list_row.blp',
        'ui/event_details_dialog/attendee_list_row/attendee_row.blp',
        'ui/event_details_dialog/attendee_list_row/attendee_status_section.blp',
        'ui/event_details_dialog/attendees_list_page/attendee_details_row.blp',
        'ui/event_details_dialog/attendees_list_page/attendees_list_page.blp',
        'ui/event_details_dialog/event_details_dialog.blp',

        # Qr Code Dialog
        'ui/qr_code_dialog.blp',

        # Search Dialog
        'ui/search_dialog/search_dialog.blp',
        'ui/search_dialog/search_result_row.blp',

        # Shortcuts Dialog
        'ui/shortcuts_dialog.blp',

        # Sidebar
        'ui/sidebar.blp',

        # Views
        # Month View
        'ui/views/month_view/event_widget.blp',
        'ui/views/month_view/month_view.blp',
        'ui/views/month_view/month_view_header.blp',
        'ui/views/month_view/month_view_inner.blp',
        'ui/views/month_view/month_view_row.blp',
        'ui/views/month_view/overflow_button.blp',
        # Year View
        'ui/views/year_view/year_view.blp',
        'ui/views/year_view/year_view_cell.blp',
        'ui/views/year_view/year_view_row.blp',

        'ui/window.blp',
    ),
    output: '.',
    command: [
        find_program('blueprint-compiler'),
        'batch-compile',
        '@OUTPUT@',
        '@CURRENT_SOURCE_DIR@',
        '@INPUT@',
    ],
)

gnome.compile_resources(
    meson.project_name(),
    'resources.gresource.xml',
    gresource_bundle: true,
    dependencies: [blueprints, stylesheet],
    install: true,
    install_dir: pkgdatadir,
)
