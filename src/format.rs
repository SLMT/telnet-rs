use byte::*;
use option::TelnetOption;
use negotiation::NegotiationAction;

///
/// Format `data` to be sent over Telnet and appends the result to `buffer`.
///
fn format_internal(mut buffer: Vec<u8>, data: &[u8]) -> Vec<u8> {
    let mut start = 0;

    for i in 0..data.len() {
        if data[i] == BYTE_IAC {
            buffer.extend_from_slice(&data[start..(i+1)]);
            start = i;
        }
    }

    if start < data.len() {
        buffer.extend_from_slice(&data[start..]);
    }

    buffer
}

///
/// Formats a given data block to be sent over Telnet.
///
/// # Examples
/// ```rust
/// let data = [1, 2, 0xFF, 3, 4];
/// let buffer = telnet::format::data(&data);
/// assert_eq!(buffer, vec![1, 2, 0xFF, 0xFF, 3, 4].into());
/// ```
///
pub fn data(data: &[u8]) -> Box<[u8]> {
    format_internal(Vec::new(), data).into()
}

///
/// Formats a Telnet negotiation to be sent over Telnet.
///
/// # Examples
/// ```rust
/// use telnet::{NegotiationAction, TelnetOption};
/// 
/// let buffer = telnet::format::negotiation(NegotiationAction::Will, TelnetOption::Echo);
/// assert_eq!(buffer, vec![0xFF, 0xFB, 0x01].into());
/// ```
///
pub fn negotiation(action: NegotiationAction, opt: TelnetOption) -> Box<[u8]> {
    Box::new([BYTE_IAC, action.to_byte(), opt.to_byte()])
}


///
/// Formats a Telnet sub-negotiation to be sent over Telnet.
///
/// # Examples
/// ```rust
/// use telnet::{NegotiationAction, TelnetOption};
/// 
/// let data = [1, 2, 3];
/// let buffer = telnet::format::sub_negotiation(TelnetOption::TTYPE, &data);
/// assert_eq!(buffer, vec![0xFF, 0xFA, 24, 1, 2, 3, 0xFF, 0xF0].into());
/// ```
///
pub fn sub_negotiation(opt: TelnetOption, data: &[u8]) -> Box<[u8]> {
    let mut buf = format_internal(vec![BYTE_IAC, BYTE_SB, opt.to_byte()], data);
    buf.extend_from_slice(&[BYTE_IAC, BYTE_SE]);
    buf.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escape_test(data: &[u8], result: Vec<u8>) {
        let buffer = format_internal(Vec::new(), data);
        assert_eq!(buffer, result);
    }

    #[test]
    fn escape_double_iac() {
        escape_test(&[1, 2, 0xFF, 0xFF, 3, 4], vec![1, 2, 0xFF, 0xFF, 0xFF, 0xFF, 3, 4]);
    }

    #[test]
    fn escape_no_iac() {
        escape_test(&[1, 2, 3, 4], vec![1, 2, 3, 4]);
    }

    #[test]
    fn escape_iac_only() {
        escape_test(&[0xFF], vec![0xFF, 0xFF]);
    }

    #[test]
    fn escape_iac_sub_negotiation() {
        let data = [1, 0xFF, 3];
        let buffer = sub_negotiation(TelnetOption::TTYPE, &data);
        assert_eq!(buffer, vec![0xFF, 0xFA, 24, 1, 0xFF, 0xFF, 3, 0xFF, 0xF0].into());
    }
}
