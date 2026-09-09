grass = find_program('grass')

configure_file(
    input: 'style.scss',
    output: 'style.css',
    command: [grass, '@INPUT@', '@OUTPUT@'],
)

run_command(
    'cp',
    meson.current_build_dir() / 'style.css',
    meson.current_source_dir() / 'style.css',
    check: true,
)

blueprint_compiler = find_program('blueprint-compiler')

configure_file(
    input: 'ui/shortcuts_dialog.blp',
    output: 'shortcuts_dialog.ui',
    command: [blueprint_compiler, 'compile', '@INPUT@', '--output', '@OUTPUT@'],
)

run_command(
    'cp',
    meson.current_build_dir() / 'shortcuts_dialog.ui',
    meson.current_source_dir() / 'ui/shortcuts_dialog.ui',
    check: true,
)
