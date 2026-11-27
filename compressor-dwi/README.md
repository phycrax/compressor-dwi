# compressor-dwi

Shared core of the Dual Wire Interface (DWI) used by variable-speed compressor drives.
It builds and checks the 5-byte frames, and sends them over any `embedded-io-async` UART.

You probably want one of these instead:

- [`secop-dwi`](https://crates.io/crates/secop-dwi): Secop NLV/SLVE. Tested on SLVE18CN.
- [`embraco-dwi`](https://crates.io/crates/embraco-dwi): Embraco CF10B. Not tested on hardware yet.

## Usage

Use this crate directly to talk to a compressor that the crates above do not cover.

```rust
use compressor_dwi::{Command, Driver};

let mut driver = Driver::new(tx, rx);

// Send command 0x3C with data 0x3980, and expect a reply with command byte 0x80.
let value = driver.transact(&Command::new(0x3C, 0x3980, 0x80)).await?;
```

## License

MIT or Apache-2.0, at your option.
