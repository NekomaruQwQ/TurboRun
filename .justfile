set shell := ["nu", "-c"]

run:
    ^cargo run -- \
        --config "target/TurboRun.toml" \
        --plugin-pack "plugins/base.nu"
