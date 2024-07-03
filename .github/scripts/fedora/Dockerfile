FROM fedora:40

RUN dnf -y install @development-tools gcc-c++ wl-clipboard libxkbcommon-devel dbus-devel wxGTK-devel.x86_64

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.77.0

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN mkdir espanso && cargo install rust-script --version "0.7.0" && cargo install --force cargo-make --version 0.37.5

COPY . espanso

RUN cd espanso
