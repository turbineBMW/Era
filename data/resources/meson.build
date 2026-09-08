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
        'ui/shortcuts_dialog.blp',
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
