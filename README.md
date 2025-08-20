# rs-dbc

[![Crates.io](https://img.shields.io/crates/v/rs-dbc.svg)](https://crates.io/crates/rs-dbc)
[![docs](https://docs.rs/rs_dbc/badge.svg)](https://docs.rs/rs_dbc)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![pipeline status](https://gitlab.com/JP-Makers/rs-dbc/badges/main/pipeline.svg)](https://gitlab.com/JP-Makers/rs-dbc/-/commits/main)

**`rs-dbc`** is a library written in Rust for parsing and handling CAN DBC files.

# Example

```rust
use std::fs::File;
use std::io::{self, Read};
use rs_dbc::Dbc;

fn main() -> io::Result<()> {
    let mut f = File::open("./examples/simple.dbc")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let dbc = Dbc::from_slice(&buffer).expect("Failed to parse DBC file");

    for msg in dbc.messages {
        println!("Message Name: {}", msg.message_name);
        println!("Message ID: 0x{:X}", msg.message_id.raw());
        println!("ID-Format: {}", msg.message_id.kind());

        for sig in msg.signals {
            println!("Signal Name: {}", sig.name);
            println!("Byte Order: {}", sig.byte_order);
            println!("Value Type: {}", sig.value_type);
            println!("");
        }
        println!("");
    }
    Ok(())
}
```
## 📞 Support

- 🐛 **Issues**: [Git Issues](../../issues)
- 📧 **Contact**: Open an issue for questions

---

<div align="center">

**Made with ❤️ and Rust**

[⭐ Star this repo](../../stargazers) • [📝 Report bug](../../issues)

</div>