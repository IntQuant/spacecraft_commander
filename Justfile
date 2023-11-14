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