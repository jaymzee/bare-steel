# bare-steel
bare metal coding in rust

based on [Writing an OS in Rust](https://os.phil-opp.com/freestanding-rust-binary/)

changes:
rust-toolchain to: nightly-2021-07-14

first install bootimage with:
``` sh
cargo install bootimage
```

and add components to this install of rust with:
``` sh
rustup component add rust-src --toolchain nightly-2021-07-14-x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview
```

last working rust toolchain: rustc 1.59.0-nightly (404c8471a 2021-12-14)
