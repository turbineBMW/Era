blueprints = custom_target(
  'blueprints',
  input: files(
    # Calendar Manager Dialog
    'ui/calendar_manager_dialog/calendar_creation_dialog.blp',
    'ui/calendar_manager_dialog/calendar_details_page.blp',
    'ui/calendar_manager_dialog/calendar_manager_dialog.blp',
    'ui/calendar_manager_dialog/calendar_row.blp',
    'ui/calendar_manager_dialog/collection_row.blp',
    'ui/calendar_manager_dialog/collections_list.blp',
    'ui/calendar_manager_dialog/collections_list_page.blp',

    # Components
    'ui/components/error_dialog.blp',
    'ui/components/loading_bin.blp',
    'ui/components/loading_button_row.blp',

    # Create Event Dialog
    'ui/create_event_dialog/calendar_combo_row/calendar_combo_row.blp',
    'ui/create_event_dialog/calendar_combo_row/calendar_combo_row_header.blp',
    'ui/create_event_dialog/calendar_combo_row/calendar_combo_row_item.blp',
    'ui/create_event_dialog/calendar_combo_row/calendar_combo_row_list_item.blp',
    'ui/create_event_dialog/create_event_dialog.blp',
    'ui/create_event_dialog/date_time_picker_group/date_picker_row.blp',
    'ui/create_event_dialog/date_time_picker_group/date_time_picker_group.blp',
    'ui/create_event_dialog/date_time_picker_group/time_zone_picker_dialog.blp',

    # Event Details Dialog
    'ui/event_details_dialog.blp',

    # Qr Code Dialog
    'ui/qr_code_dialog.blp',

    # Search Dialog
    'ui/search_dialog/event_row.blp',
    'ui/search_dialog/search_dialog.blp',

    # Shortcuts Dialog
    'ui/shortcuts_dialog.blp',

    # Views
    'ui/views/month_view/month_view.blp',
    'ui/views/month_view/month_view_week_row.blp',
    'ui/views/year_view/year_view.blp',
    'ui/views/year_view/year_view_month_cell.blp',
    'ui/views/year_view/year_view_year_row.blp',

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
  dependencies: blueprints,
  install: true,
  install_dir: pkgdatadir,
)
