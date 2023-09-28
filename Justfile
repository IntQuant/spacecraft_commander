check:
    cd rust && cargo check

build:
    cd rust && cargo build

run: build
    cd game && godot4

editor: build
    cd game && godot4 -e