project(
    'era',
    'rust',
    version: '0.1.0',
    meson_version: '>= 1.12.0',
    default_options: [
        'warning_level=2',
        'werror=false',
    ],
)

backend = get_option('calendar-backend')
platform = get_option('platform')

i18n = import('i18n')
gnome = import('gnome')

era_deps = []

era_deps += dependency('gtk4', version: '>= 4.22')
era_deps += dependency('libadwaita-1', version: '>= 1.9')
era_deps += dependency(
    'clepsydre-0',
    version: '>= 0.1',
    default_options: {
        'libclepsydre': true,
            # TODO: In theory it's needed, but Android cross-build don't handle it
            # 'introspection': true,
    },
)
# TODO: Remove this once Meson handles git dependencies better
era_deps += dependency(
    'clepsydre-rebind-0.1-rs',
    version: '>= 0.1',
    default_options: {
        'clepsydre-rebind': true,
    },
)

if backend == 'android'
    # TODO: Remove this once Meson handles git dependencies better
    era_deps += dependency(
        'clepsydre-android-0.1-rs',
        version: '>= 0.1',
        default_options: {
            'clepsydre-android': true,
        },
    )
elif backend == 'eds'
    era_deps += dependency(
        'clepsydre-eds-0',
        version: '>= 0.1',
        default_options: {
            'libclepsydre-eds': true,
            'introspection': true,
        },
    )
    # TODO: Remove this once Meson handles git dependencies better
    era_deps += dependency(
        'clepsydre-eds-0.1-rs',
        version: '>= 0.1',
        default_options: {
            'clepsydre-eds': true,
        },
    )
elif backend == 'mock'
    # TODO: Remove this once Meson handles git dependencies better
    era_deps += dependency(
        'clepsydre-mock-0.1-rs',
        version: '>= 0.1',
        default_options: {
            'clepsydre-mock': true,
        },
    )
elif backend == 'p2panda'
    # TODO: Remove this once Meson handles git dependencies better
    era_deps += dependency(
        'clepsydre-p2panda-0.1-rs',
        version: '>= 0.1',
        default_options: {
            'clepsydre-p2panda': true,
        },
    )
endif

appstreamcli = find_program('appstreamcli', required: true)
compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: true)

application_name = 'Era'
base_resource_path = '/io/gitlab/TitouanReal/' + application_name

if backend == 'android'
    backend_name = 'Android'
elif backend == 'eds'
    backend_name = 'Eds'
elif backend == 'mock'
    backend_name = 'Mock'
elif backend == 'p2panda'
    backend_name = 'P2panda'
endif

base_id = 'org.gnome.gitlab.TitouanReal.' + application_name
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

features = []

if backend == 'android'
    features += 'backend-android'
elif backend == 'eds'
    features += 'backend-eds'
elif backend == 'mock'
    features += 'backend-mock'
elif backend == 'p2panda'
    features += 'backend-p2panda'
endif

if platform == 'flatpak'
    features += 'platform-flatpak'
elif platform == 'android'
    features += 'platform-android'
endif

cargo_module = import('rust').workspace(
    default_features: false,
    features: features,
)
crate = cargo_module.package()
crate.executable(
    install: true,
    android_exe_type: 'application',
    dependencies: era_deps,
)

gnome.post_install(
    glib_compile_schemas: true,
    gtk_update_icon_cache: true,
    update_desktop_database: true,
)
