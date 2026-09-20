commit := `jq -r '.modules[] | select(.name == "libclepsydre-eds") | .sources[] | select(.type == "git") | .commit' build-aux/org.gnome.gitlab.TitouanReal.Era.Eds.Devel.json`
clepsydre_build_dir := "/tmp/build-clepsydre"
clepsydre_eds_build_dir := "/tmp/build-eds"

clean:
    rm build -rf

compile: install-clepsydre
    cargo install grass
    meson setup build --reconfigure
    meson compile -C build

install-clepsydre:
    @echo "Installing clepsydre at commit: {{ commit }}"

    @rm {{ clepsydre_build_dir }} -rf
    @mkdir {{ clepsydre_build_dir }}

    git clone https://gitlab.gnome.org/TitouanReal/clepsydre.git "{{ clepsydre_build_dir }}/clepsydre"
    cd "{{ clepsydre_build_dir }}/clepsydre"
    pwd
    git -C "{{ clepsydre_build_dir }}/clepsydre" checkout "{{ commit }}"

    meson setup "{{ clepsydre_build_dir }}/clepsydre/build" "{{ clepsydre_build_dir }}/clepsydre" --prefix=/usr --libdir=lib/x86_64-linux-gnu -Dlibclepsydre=true
    meson compile -C "{{ clepsydre_build_dir }}/clepsydre/build"
    meson install -C "{{ clepsydre_build_dir }}/clepsydre/build"

    @rm {{ clepsydre_build_dir }} -rf

    @rm {{ clepsydre_eds_build_dir }} -rf
    @mkdir {{ clepsydre_eds_build_dir }}

    git clone https://gitlab.gnome.org/TitouanReal/clepsydre.git "{{ clepsydre_eds_build_dir }}/clepsydre"
    cd "{{ clepsydre_eds_build_dir }}/clepsydre"
    pwd
    git -C "{{ clepsydre_eds_build_dir }}/clepsydre" checkout "{{ commit }}"

    meson setup "{{ clepsydre_eds_build_dir }}/clepsydre/build" "{{ clepsydre_eds_build_dir }}/clepsydre" --prefix=/usr --libdir=lib/x86_64-linux-gnu -Dlibclepsydre-eds=true
    meson compile -C "{{ clepsydre_eds_build_dir }}/clepsydre/build"
    meson install -C "{{ clepsydre_eds_build_dir }}/clepsydre/build"

    @rm {{ clepsydre_eds_build_dir }} -rf

    @echo "clepsydre installed successfully"
