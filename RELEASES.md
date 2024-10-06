# Version 0.2.2

- Derived common traits for public enums ([#24], by [mwnDK1402])

[#24]: https://github.com/SLMT/telnet-rs/pull/24

# Version 0.2.1

- Added `connect_timeout` method to `Telnet` struct ([#21], by [Zylatis])

[#21]: https://github.com/SLMT/telnet-rs/pull/21

# Version 0.2.0

- Updated to Rust 2018 & idiomatic refinements ([#20], by [SaadiSave])

[#20]: https://github.com/SLMT/telnet-rs/pull/20

# Version 0.1.4

- Added Ability - as compile time feature - to have a zlib stream wrap the Stream (including the telnet negotiations) at runtime. ([#6], [#8], by [yehoshuapw])
- Improved src/option.rs (remove code duplication). ([#9], by [fogti])

[#6]: https://github.com/SLMT/telnet-rs/pull/6
[#8]: https://github.com/SLMT/telnet-rs/pull/8
[#9]: https://github.com/SLMT/telnet-rs/pull/9

# Version 0.1.3

- Fixed the bug of handling IAC escaping. ([#4], by [sethm])

[#4]: https://github.com/SLMT/telnet-rs/pull/4

# Version 0.1.2

- Added support for generic streams. ([#2], by [lux01])

[#2]: https://github.com/SLMT/telnet-rs/pull/2

# Version 0.1.1

- Made internal errors converted to `TelnetEvent::Error` ([#1], by [Rutger798])

[#1]: https://github.com/SLMT/telnet-rs/pull/1

# Version 0.1.0

- The initial version of telnet.rs

[sethm]: https://github.com/sethm
[lux01]: https://github.com/lux01
[yehoshuapw]: https://github.com/yehoshuapw
[Rutger798]: https://github.com/Rutger798
[sethm]: https://github.com/sethm
[SaadiSave]: https://github.com/SaadiSave
[Zylatis]: https://github.com/Zylatis
[fogti]: https://github.com/fogti
[mwnDK1402]: https://github.com/mwnDK1402
