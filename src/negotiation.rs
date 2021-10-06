// This implements the Q method described in Section 7 of RFC 1143

use crate::byte::{BYTE_DO, BYTE_DONT, BYTE_WILL, BYTE_WONT};

/// Actions for telnet negotiation.
#[derive(Debug)]
pub enum Action {
    Will,
    Wont,
    Do,
    Dont,
}

impl Action {
    #[allow(clippy::must_use_candidate)]
    pub fn as_byte(&self) -> u8 {
        match *self {
            Action::Will => BYTE_WILL,
            Action::Wont => BYTE_WONT,
            Action::Do => BYTE_DO,
            Action::Dont => BYTE_DONT,
        }
    }
}
