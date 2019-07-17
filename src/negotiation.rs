// This implements the Q method described in Section 7 of RFC 1143

use byte::*;

///
/// Actions for telnet negotiation.
///
#[derive(Debug)]
pub enum NegotiationAction {
    Will, Wont, Do, Dont
}

impl NegotiationAction {
    pub fn to_byte(&self) -> u8 {
        match *self {
            NegotiationAction::Will => BYTE_WILL,
            NegotiationAction::Wont => BYTE_WONT,
            NegotiationAction::Do => BYTE_DO,
            NegotiationAction::Dont => BYTE_DONT
        }
    }
}
