# telnet-rs

A simple Telnet implementation.

## Examples

### Blocking Reading

```rust
extern crate telnet;

use telnet::{Telnet, TelnetEvent};

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    loop {
        let event = telnet.read().expect("Read error");

        match event {
            TelnetEvent::Data(buffer) => {
                // Debug: print the data buffer
                println!("{:?}", buffer);
                // process the data buffer
            },
            _ => {}
        }
    }
}
```

### Non-Blocking Reading

```rust
extern crate telnet;

use telnet::{Telnet, TelnetEvent};

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    loop {
        let event = telnet.read_nonblocking().expect("Read error");

        match event {
            TelnetEvent::Data(buffer) => {
                // Debug: print the data buffer
                println!("{:?}", buffer);
                // process the data buffer
            },
            _ => {}
        }

        // Do something else ...
    }
}
```

### Writing

```rust
extern crate telnet;

use telnet::{Telnet};

fn main() {
    let mut telnet = Telnet::connect(("ptt.cc", 23), 256)
            .expect("Couldn't connect to the server...");

    let buffer: [u8; 4] = [83, 76, 77, 84];
    telnet.write(&buffer).expect("Read error");
}
```
