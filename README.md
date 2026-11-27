# compressor-dwi

[![CI](https://github.com/phycrax/compressor-dwi/actions/workflows/ci.yml/badge.svg)](https://github.com/phycrax/compressor-dwi/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
![MSRV](https://img.shields.io/badge/rustc-1.88+-blue)

Async `no_std` Rust drivers for the Dual Wire Interface (DWI) of variable-speed compressor drives.

| Crate | Version | Docs | Compressor | Tested on hardware |
| --- | --- | --- | --- | --- |
| `secop-dwi` | [![crates.io](https://img.shields.io/crates/v/secop-dwi)](https://crates.io/crates/secop-dwi) | [![docs.rs](https://img.shields.io/docsrs/secop-dwi)](https://docs.rs/secop-dwi) | Secop NLV/SLVE | Yes, on SLVE18CN |
| `embraco-dwi` | [![crates.io](https://img.shields.io/crates/v/embraco-dwi)](https://crates.io/crates/embraco-dwi) | [![docs.rs](https://img.shields.io/docsrs/embraco-dwi)](https://docs.rs/embraco-dwi) | Embraco CF10B | No |
| `compressor-dwi` | [![crates.io](https://img.shields.io/crates/v/compressor-dwi)](https://crates.io/crates/compressor-dwi) | [![docs.rs](https://img.shields.io/docsrs/compressor-dwi)](https://docs.rs/compressor-dwi) | Shared core used by both | - |

## Usage

Add the crate for your compressor:

```toml
[dependencies]
secop-dwi = "0.1"
```

Set up your UART at 600 baud, 8 data bits, no parity, 1 stop bit.
Pass its two halves to the driver. `tx` must implement `embedded_io_async::Write`,
and `rx` must implement `embedded_io_async::Read + ReadReady`.

```rust
use secop_dwi::Compressor;

let mut compressor = Compressor::new(tx, rx);

// Run at 3000 RPM. Send 0 to stop.
let status = compressor.set_speed(3000).await?;
if let Some(faults) = status.faults {
    // Handle the faults.
}

let watts = compressor.read_power().await?;
```

`embraco-dwi` works the same way, with `embraco_dwi::Compressor`.

A call waits until the compressor answers. Wrap it in a timeout if the compressor might not answer.

Turn on the `defmt` feature to log the types with `defmt`.

## License

compressor-dwi is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/phycrax/compressor-dwi/blob/main/LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/phycrax/compressor-dwi/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
