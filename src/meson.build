cargo = find_program('cargo')
cargo_options = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_options += ['--target-dir', meson.project_build_root() / 'src']
cargo_env = ['CARGO_HOME=' + meson.project_build_root() / 'cargo-home']

full_version = version

if profile == 'Devel'
    rust_target = 'debug'
    vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
    full_version += '-' + vcs_tag
else
    cargo_options += ['--release']
    rust_target = 'release'
endif

if backend == 'eds'
    cargo_options += ['--no-default-features', '--features', 'clepsydre-eds']
elif backend == 'mock'
    cargo_options += ['--no-default-features', '--features', 'clepsydre-mock']
elif backend == 'p2panda'
    cargo_options += ['--no-default-features', '--features', 'clepsydre-p2panda']
endif

conf = configuration_data()
conf.set_quoted('APP_ID', application_id)
conf.set_quoted('APP_NAME', application_name)
conf.set_quoted('BASE_RESOURCE_PATH', base_resource_path)
conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
conf.set_quoted('LOCALEDIR', prefix / get_option('localedir'))
conf.set_quoted('PKGDATADIR', pkgdatadir)
conf.set_quoted('PROJECT_NAME', meson.project_name())
conf.set_quoted('VERSION', full_version)

configure_file(input: 'config.rs.in', output: 'config.rs', configuration: conf)

# Copy the config.rs output to the source directory.
run_command(
    'cp',
    meson.project_build_root() / 'src' / 'config.rs',
    meson.project_source_root() / 'src' / 'config.rs',
    check: true,
)

custom_target(
    'cargo-build',
    build_by_default: true,
    build_always_stale: true,
    output: meson.project_name(),
    console: true,
    install: true,
    install_dir: bindir,
    command: [
        'env',
        cargo_env,
        cargo,
        'build',
        cargo_options,
        '&&',
        'cp',
        'src' / rust_target / meson.project_name(),
        '@OUTPUT@',
    ],
)
