# esp32-hal

A hardware abstraction layer for the [esp32](https://en.wikipedia.org/wiki/ESP32) written in Rust.

Contributions are welcome :)

Join in on the discussion: https://matrix.to/#/#esp-rs:matrix.org!

## Running examples

There are currently two ways to flash the esp32:

  * The `flash` script using `esptool` 
    - If you are familiar with the esp ecosystem, there is a `flash` script in this repo which utilizes the espressif esptool to flash the esp32 over usb.
    Example usage:
     ```rust
        ./flash -p /dev/ttyUSB0 -e blinky --release
     ```
  
  * The [`espflash`](https://github.com/icewind1991/espflash) cargo subcommand
    - A Rust rewrite of the esptool, with a cargo subcommand. Example usage:
     ```rust
        cargo espflash --example blinky --release /dev/ttyUSB0
     ```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
