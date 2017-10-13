
#[derive(Debug)]
pub enum TelnetEvent<'a> {
    DataReceived(&'a [u8]),
    UnknownIAC(u8),
    Negotiation(u8),
    Subnegotiation(u8, &'a [u8]),
    Nothing
}
