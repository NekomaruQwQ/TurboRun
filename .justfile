set shell := ["nu", "-c"]

dev:
    ^cargo run -- \
        --config "target/TurboRun.toml" \
        --plugin-pack "plugins/base.nu"
run:
    ^cargo run --release -- \
        --config "target/TurboRun.toml" \
        --plugin-pack "plugins/base.nu"
