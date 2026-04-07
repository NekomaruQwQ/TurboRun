set shell := ["nu", "-c"]

run:
    TURBORUN_CONFIG=target/TurboRun.toml \
    TURBORUN_PLUGIN=plugins \
    cargo run
