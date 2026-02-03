commit := `jq -r '.modules[] | select(.name == "libclepsydre-eds") | .sources[] | select(.type == "git") | .commit' build-aux/io.gitlab.TitouanReal.Kalendasom.Devel.json`
build_dir := "/tmp/clepsydre-build"

install-clepsydre:
    @echo "Installing clepsydre at commit: {{ commit }}"

    # Clone and build clepsydre
    @rm {{ build_dir }} -rf
    @mkdir {{ build_dir }}

    git clone https://gitlab.gnome.org/TitouanReal/clepsydre.git "{{ build_dir }}/clepsydre"
    cd "{{ build_dir }}/clepsydre"
    pwd
    git -C "{{ build_dir }}/clepsydre" checkout "{{ commit }}"

    meson setup "{{ build_dir }}/clepsydre/build" "{{ build_dir }}/clepsydre" --prefix=/usr --libdir=lib/x86_64-linux-gnu -Dlibclepsydre=true -Dlibclepsydre-eds=true
    meson compile -C "{{ build_dir }}/clepsydre/build"
    meson install -C "{{ build_dir }}/clepsydre/build"

    @rm {{ build_dir }} -rf

    @echo "clepsydre installed successfully"
