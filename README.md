# telnet-rs

[![Build Status](https://travis-ci.org/SLMT/telnet-rs.svg?branch=master)](https://travis-ci.org/SLMT/telnet-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](https://img.shields.io/crates/v/telnet)](https://crates.io/crates/telnet)
[![API docs](https://docs.rs/telnet/badge.svg)](http://docs.rs/telnet)

A simple Telnet implementation.

## Examples

### Blocking Reading

```rust
use telnet::{Telnet, Event};

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    loop {
        let event = telnet.read().expect("Read error");

        if let Event::Data(buffer) = event {
            // Debug: print the data buffer
            println!("{:?}", buffer);
            // process the data buffer
        }
    }
}
```

### Non-Blocking Reading

```rust
use telnet::{Telnet, Event};

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    loop {
        let event = telnet.read_nonblocking().expect("Read error");

        if let Event::Data(buffer) = event {
            // Debug: print the data buffer
            println!("{:?}", buffer);
            // process the data buffer
        }

        // Do something else ...
    }
}
```

### Writing

```rust
use telnet::Telnet;

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    let buffer: [u8; 4] = [83, 76, 77, 84];
    telnet.write(&buffer).expect("Read error");
}
```

## TODOs

- reduce unnecessary data copy
- add coverage check
- add crate-level documentation
