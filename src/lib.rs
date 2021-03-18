//! #### MCCP2
//! A feature of some telnet servers is `MCCP2` which allows the downstream data to be compressed.
//! To use this, first enable the `zcstream` [rust feature](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) for this crate.
//! Then in the code deal with the relevant events, and switch the zlib when appropriate.
//!
//! Basic usage example:
//! ```ignore
//! match event {
//! 	TelnetEvent::Data(buffer) => {
//! 		println!("{}", &std::str::from_utf8(&(*buffer)).unwrap());
//! 	},
//! 	TelnetEvent::Negotiation(NegotiationAction::Will, TelnetOption::Compress2) => {
//! 		telnet.negotiate(NegotiationAction::Do, TelnetOption::Compress2);
//! 	},
//! 	TelnetEvent::Subnegotiation(TelnetOption::Compress2, _) => {
//! 		telnet.begin_zlib();
//! 	}
//! }
//! ```
mod negotiation;
mod option;
mod event;
mod byte;
mod stream;
mod parse;
pub mod format;
#[cfg(feature = "zcstream")]
mod zcstream;
#[cfg(feature = "zcstream")]
mod zlibstream;

pub use stream::Stream;
#[cfg(feature = "zcstream")]
pub use zlibstream::ZlibStream;
#[cfg(feature = "zcstream")]
pub use zcstream::ZCStream;
pub use option::TelnetOption;
pub use event::{Event, Events};
pub use negotiation::NegotiationAction;
pub use parse::Parser;

#[cfg(feature = "zcstream")]
type TStream = zcstream::ZCStream;
#[cfg(not(feature = "zcstream"))]
type TStream = stream::Stream;

#[cfg(test)]
mod tests {
    use super::*;
}
