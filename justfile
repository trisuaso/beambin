# build release
build database="sqlite":
    just build-styles
    cargo build -r --no-default-features --features {{database}} --bin beambin

mimalloc-build database="sqlite":
    just build-styles
    cargo build -r --no-default-features --features {{database}},mimalloc --bin beambin

build-styles:
    bunx tailwindcss -i ./crates/beambin/static/css/input.css -o ./crates/beambin/static/css/style.css

# build debug
build-d:
    just build-styles
    cargo build --bin beambin

# test
test:
    just build-styles
    cargo run --bin beambin
