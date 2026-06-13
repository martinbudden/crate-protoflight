# Set global environment variables for all tasks
export RUST_LOG := "debug"
export RUST_BACKTRACE := "1"

run:
    cargo run

test:
    cargo test

watch:
    cargo watch

# Decodes the blackbox log using the relative path
bbdecode:
    ../../blackbox-tools/obj/blackbox_decode --debug blackbox_log.bbl
