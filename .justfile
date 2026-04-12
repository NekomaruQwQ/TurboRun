set shell := ["nu", "-c"]

run:
    ^cargo run -- \
        --config "target/TurboRun.yaml" \
        --plugin-pack "plugins/base.nu"
