# build release
build database="sqlite":
    cargo build -r --no-default-features --features {{database}} --bin beambin

mimalloc-build database="sqlite":
    cargo build -r --no-default-features --features {{database}},mimalloc --bin beambin]

moka-build database="sqlite":
    cargo build -r --no-default-features --features {{database}},mimalloc,moka --bin beambin]

# build debug
build-d:
    cargo build --bin beambin

# test
test:
    cargo run --bin beambin --no-default-features --features sqlite,mimalloc,moka

# ...
clean-deps:
    cargo upgrade -i
    cargo machete
