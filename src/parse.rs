use byte::*;
use event::{Events, Event};
use negotiation::NegotiationAction;
use option::TelnetOption;

#[derive(Debug)]
enum ParsingState {
    NormalData(usize),
    IAC,
    SB,
    SBData(TelnetOption, Vec<u8>, bool), // option, data, iac
    Negotiation(NegotiationAction),
}

pub struct Parser {
    state: ParsingState,
}

impl Parser {
    ///
    /// Create new parser.
    ///
    pub fn new() -> Parser {
        Parser {
            state: ParsingState::NormalData(0),
        }
    }

    ///
    /// Parses all the `Event`s from the supplied buffer.
    ///
    /// # Examples
    /// ```rust
    /// use telnet::{Parser, Event::Data};
    /// 
    /// let buffer = [1, 2, 0xFF, 0xFF, 3, 4];
    /// 
    /// let mut parser = Parser::new();
    /// let events: Vec<_> = parser.parse(&buffer).collect();
    /// 
    /// assert_eq!(events, vec![Data(&[1, 2]), Data(&[0xFF, 3, 4])]);
    /// ```
    ///
    pub fn parse<'a>(&mut self, buffer: &'a [u8]) -> Events<'a> {
        if let ParsingState::NormalData(data_start) = self.state {
            assert!(data_start == 0);
        }

        let mut events: Vec<Event> = (0..buffer.len())
            .filter_map(|i| self.parse_byte(buffer, i))
            .collect();

        if let ParsingState::NormalData(data_start) = self.state {
            if data_start < buffer.len() {
                events.push(Event::Data(&buffer[data_start..]));
            }

            // Reset for next call to read
            self.state = ParsingState::NormalData(0);
        }

        events.into()
    }

    ///
    /// Parses a single byte from the supplied buffer.
    ///
    /// Expects to be called as part of a loop where `index`, starting at 0, is incremented by 1
    /// on each call until the end of the buffer is reached.
    ///
    fn parse_byte<'a>(&mut self, buffer: &'a [u8], index: usize) -> Option<Event<'a>> {
        let byte = buffer[index];

        match self.state {
            // Normal Data
            ParsingState::NormalData(data_start) => {
                if byte == BYTE_IAC {
                    // The following bytes will be commands

                    // Update the state
                    self.state = ParsingState::IAC;

                    // Send the data before this byte
                    if data_start < index {
                        return Some(Event::Data(&buffer[data_start..index]));
                    }
                }
            }

            // Telnet Commands
            ParsingState::IAC => {
                let mut err = false;

                self.state = match byte {
                    // Negotiation Commands
                    BYTE_WILL => ParsingState::Negotiation(NegotiationAction::Will),
                    BYTE_WONT => ParsingState::Negotiation(NegotiationAction::Wont),
                    BYTE_DO => ParsingState::Negotiation(NegotiationAction::Do),
                    BYTE_DONT => ParsingState::Negotiation(NegotiationAction::Dont),
                    // Subnegotiation
                    BYTE_SB => ParsingState::SB,
                    // Escaping
                    // TODO: Write a test case for this
                    BYTE_IAC => ParsingState::NormalData(index),
                    // Unknown IAC commands
                    _ => {
                        err = true;
                        ParsingState::NormalData(index + 1)
                    }
                };

                // TODO: Write a test case for this
                if err {
                    return Some(Event::UnknownIAC(byte));
                }
            }

            // Negotiation
            ParsingState::Negotiation(action) => {
                self.state = ParsingState::NormalData(index + 1);
                let opt = TelnetOption::parse(byte);
                return Some(Event::Negotiation(action, opt));
            }

            // Start subnegotiation
            ParsingState::SB => {
                let opt = TelnetOption::parse(byte);
                self.state = ParsingState::SBData(opt, Vec::new(), false);
            }

            // Subnegotiation's data
            ParsingState::SBData(opt, ref mut data, ref mut iac) => {
                if *iac {
                    // IAC inside Subnegotiation's data
                    *iac = false;

                    match byte {
                        // The end of subnegotiation
                        BYTE_SE => {
                            let data_boxed = data.clone().into();
                            self.state = ParsingState::NormalData(index + 1);
                            return Some(Event::Subnegotiation(opt, data_boxed));
                        }
                        // Escaping
                        BYTE_IAC => data.push(BYTE_IAC),
                        // TODO: Write a test case for this
                        b => {
                            return Some(Event::Error(format!(
                                "Unexpected byte after IAC inside SB: {}",
                                b
                            )))
                        }
                    }
                } else {
                    if byte == BYTE_IAC {
                        *iac = true;
                    } else {
                        data.push(byte);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_test(buffer: &[u8], result: Vec<Event>) {
        let mut parser = Parser::new();
        for i in 0..buffer.len() {
            let events: Vec<_> = parser.parse(&buffer[..i])
                .chain(parser.parse(&buffer[i..]))
                .collect();
            assert_eq!(events, result);
        }
    }

    #[test]
    fn parse_double_iac() {
        let buffer = [1, 0xFF, 0xFF, 0xFF, 0xFF];
        parse_test(&buffer, vec![
            Event::Data(&[1]),
            Event::Data(&[0xFF]),
            Event::Data(&[0xFF])
        ]);
    }

    #[test]
    fn parse_iac_only() {
        let buffer = [0xFF, 0xFF];
        parse_test(&buffer, vec![Event::Data(&[0xFF])]);
    }

    #[test]
    fn parse_negotiation() {
        let buffer = [1, 0xFF, 0xFB, 0x01, 2];
        parse_test(&buffer, vec![
            Event::Data(&[1]),
            Event::Negotiation(NegotiationAction::Will, TelnetOption::Echo),
            Event::Data(&[2])
        ]);
    }

    #[test]
    fn parse_sub_negotiation() {
        let buffer = [1, 0xFF, 0xFA, 24, 1, 2, 3, 0xFF, 0xF0, 2];
        parse_test(&buffer, vec![
            Event::Data(&[1]),
            Event::Subnegotiation(TelnetOption::TTYPE, vec![1, 2, 3].into()),
            Event::Data(&[2])
        ]);
    }

    #[test]
    fn parse_iac_sub_negotiation() {
        let buffer = [0xFF, 0xFA, 24, 1, 0xFF, 0xFF, 3, 0xFF, 0xF0];
        parse_test(&buffer, vec![
            Event::Subnegotiation(TelnetOption::TTYPE, vec![1, 0xFF, 3].into()),
        ]);
    }
}
