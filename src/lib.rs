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
mod byte;
mod event;
pub mod format;
mod negotiation;
mod option;
mod parse;
mod stream;
#[cfg(feature = "zcstream")]
mod zcstream;
#[cfg(feature = "zcstream")]
mod zlibstream;

pub use event::{Event, Events};
pub use negotiation::NegotiationAction;
pub use option::TelnetOption;
pub use parse::Parser;
pub use stream::Stream;
#[cfg(feature = "zcstream")]
pub use zcstream::ZCStream;
#[cfg(feature = "zcstream")]
pub use zlibstream::ZlibStream;

#[cfg(feature = "zcstream")]
type TStream = zcstream::ZCStream;
#[cfg(not(feature = "zcstream"))]
type TStream = stream::Stream;

use std::io::{Read, Result, Write};

pub struct Reader<R: Read> {
    parser: Parser,
    reader: R,
    buffer: Box<[u8]>,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R, capacity: usize) -> Self {
        Self {
            parser: Parser::new(),
            reader,
            buffer: vec![0; capacity].into(),
        }
    }

    pub fn read(&mut self) -> Result<Events> {
        let n = self.reader.read(&mut self.buffer)?;
        Ok(self.parser.parse(&self.buffer[..n]))
    }
}

pub struct Writer<W: Write>(W);

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self(writer)
    }

    pub fn write_all(&mut self, buffer: &[u8]) -> Result<()> {
        for buf in format::data(buffer) {
            self.0.write_all(buf)?;
        }
        Ok(())
    }

    pub fn negotiate(&mut self, action: NegotiationAction, opt: TelnetOption) -> Result<()> {
        let buf = format::negotiation(action, opt);
        self.0.write_all(&buf)
    }

    pub fn sub_negotiate(&mut self, opt: TelnetOption, parameters: &[u8]) -> Result<()> {
        let buf = format::sub_negotiation(opt, parameters);
        self.0.write_all(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
