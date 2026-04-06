set shell := ["nu", "-c"]

run:
    TURBORUN_CONFIG=target/config.toml \
    TURBORUN_PLUGIN=plugins \
    cargo run
