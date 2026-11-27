# secop-dwi

Async `no_std` driver for the Dual Wire Interface (DWI) of Secop NLV/SLVE compressor drives.

Tested on a Secop SLVE18CN.

## Usage

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

let rpm = compressor.read_actual_speed().await?;
let watts = compressor.read_power().await?;
```

A call waits until the compressor answers. Wrap it in a timeout if the compressor might not answer.

Turn on the `defmt` feature to log the types with `defmt`.

## License

MIT or Apache-2.0, at your option.
