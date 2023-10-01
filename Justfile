check:
    cd rust && cargo check

build:
    cd rust && cargo build

run: build
    cd game && godot4 -- server

run-client: build
    cd game && godot4 -- client

editor: build
    cd game && godot4 -e