desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_conf.set('app-name', application_name)
desktop_conf.set('project-name', meson.project_name())
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf,
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications',
)

test('Validate desktop file', desktop_file_validate, args: [desktop_file])

metainfo_conf = configuration_data()
metainfo_conf.set('app-id', application_id)
metainfo_conf.set('gettext-package', gettext_package)
metainfo_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: metainfo_conf,
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'metainfo',
)

test(
  'Validate metainfo file',
  appstreamcli,
  args: ['validate', '--no-net', '--explain', metainfo_file],
)

gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)
gschema_conf.set('gettext-package', gettext_package)
gschema_conf.set('resource-path', base_resource_path)
configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(application_id),
  configuration: gschema_conf,
  install_dir: datadir / 'glib-2.0' / 'schemas',
)

test(
  'Validate schema file',
  compile_schemas,
  args: ['--strict', '--dry-run', meson.current_source_dir()],
)

service_conf = configuration_data()
service_conf.set('bindir', bindir)
service_conf.set('app-id', application_id)
service_conf.set('project-name', meson.project_name())
configure_file(
  input: '@0@.service.in'.format(base_id),
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install_dir: datadir / 'dbus-1' / 'services',
)

subdir('icons')
subdir('resources')
