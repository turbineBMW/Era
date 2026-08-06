full_version = version

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
