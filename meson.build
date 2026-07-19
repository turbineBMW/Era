project(
    'era',
    'rust',
    version: '0.1.0',
    meson_version: '>= 1.8.5',
    default_options: [
        'warning_level=2',
        'werror=false',
    ],
)

backend = get_option('calendar-backend')

i18n = import('i18n')
gnome = import('gnome')

dependency('gtk4', version: '>= 4.22')
dependency('libadwaita-1', version: '>= 1.9')
dependency('clepsydre-0', version: '>= 0.1')

if backend == 'eds'
    dependency('clepsydre-eds-0', version: '>= 0.1')
endif

appstreamcli = find_program('appstreamcli', required: true)
compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: true)

application_name = 'Era'
base_resource_path = '/io/gitlab/TitouanReal/' + application_name

if backend == 'eds'
    backend_name = 'Eds'
elif backend == 'mock'
    backend_name = 'Mock'
elif backend == 'p2panda'
    backend_name = 'P2panda'
endif

base_id = 'io.gitlab.TitouanReal.' + application_name
application_id = base_id + '.' + backend_name
version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

if get_option('buildtype') == 'release'
    profile = 'Stable'
else
    profile = 'Devel'
    application_id += '.Devel'
endif

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
    glib_compile_schemas: true,
    gtk_update_icon_cache: true,
    update_desktop_database: true,
)
