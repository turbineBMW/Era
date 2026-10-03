if release_type == 'stable'
    base_id_release_type = base_id
elif release_type == 'nightly'
    base_id_release_type = base_id + '.Nightly'
elif release_type == 'devel'
    base_id_release_type = base_id + '.Devel'
endif

install_data(
    '@0@.svg'.format(base_id_release_type),
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps',
    rename: '@0@.svg'.format(application_id),
)

install_data(
    '@0@-symbolic.svg'.format(base_id),
    install_dir: iconsdir / 'hicolor' / 'symbolic' / 'apps',
    rename: '@0@-symbolic.svg'.format(application_id),
)
