check:
    cd rust && cargo check

build:
    cd rust && cargo build

run: build
    cd game && RUST_BACKTRACE=1 godot4 -- server

run-client: build
    cd game && RUST_BACKTRACE=1  godot4 -- client

editor: build
    cd game && godot4 -e

doc:
    cd rust && cargo doc --no-deps --document-private-items --open

export:
    cd rust && cargo build --release --target x86_64-pc-windows-gnu
    cd game && godot4 --export-release "Windows Desktop"
    cd rust && cargo build --release
    cd game && godot4 --export-release "Linux/X11"
