# Notes

To run with logging use:

```sh
RUST_LOG=debug cargo run
```

or

```sh
RUST_LOG=debug RUST_BACKTRACE=1 cargo run
```

To decode the blackbox log use:

```sh
../../blackbox-tools/obj/blackbox_decode --debug blackbox_log.bbl
```

To check with a given target use:

```sh
cargo check --target thumbv8m.main-none-eabihf
```
