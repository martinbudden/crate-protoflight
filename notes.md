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

To check for flash usage:

```sh
cargo bloat --release --target thumbv7em-none-eabihf -n 20
```

rp2350 build

```sh
cargo build --target thumbv8m.main-none-eabihf --features rp2350
```
