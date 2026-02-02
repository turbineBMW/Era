FROM quay.io/gnome_infrastructure/gnome-build-meta:core-nightly

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && /root/.cargo/bin/rustup toolchain install nightly \
    && /root/.cargo/bin/rustup default nightly \
    && /root/.cargo/bin/rustup component add rustfmt clippy

ENV PATH="/root/.cargo/bin:${PATH}"

RUN chsh -s /usr/bin/fish
