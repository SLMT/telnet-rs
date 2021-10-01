#[derive(Debug)]
pub enum Error {
    UnexpectedByte(u8),
    InternalQueueErr,
    NegotiationErr,
    SubnegotiationErr(SubnegotiationType),
}

#[allow(clippy::enum_glob_use)]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            UnexpectedByte(b) => {
                f.write_fmt(format_args!("Unexpected byte after IAC inside SB: {}", &b))
            }
            InternalQueueErr => f.write_str("Internal Queue Error"),
            NegotiationErr => f.write_str("Negotiation failed"),
            SubnegotiationErr(s) => {
                use self::SubnegotiationType::*;
                match s {
                    Start => f.write_str("Subnegotiation failed (START)"),
                    Data => f.write_str("Subnegotiation failed (DATA)"),
                    End => f.write_str("Subnegotiation failed (END)"),
                }
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum SubnegotiationType {
    Start,
    Data,
    End,
}
