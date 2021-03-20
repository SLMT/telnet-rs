use byte::*;
use negotiation::NegotiationAction;
use option::TelnetOption;

pub struct FormattedData<'a>(std::vec::IntoIter<&'a [u8]>);

impl<'a> FormattedData<'a> {
    ///
    /// Create owned copy of the formatted data.
    ///
    /// # Examples
    /// ```rust
    /// let data = [1, 2, 0xFF, 3, 4];
    /// let data_formatted = telnet::format::data(&data).to_owned();
    /// assert_eq!(data_formatted, vec![1, 2, 0xFF, 0xFF, 3, 4].into_boxed_slice());
    /// ```
    ///
    pub fn to_owned(self) -> Box<[u8]> {
        self.append_to(Vec::new()).into()
    }

    fn append_to(self, mut buffer: Vec<u8>) -> Vec<u8> {
        self.0.for_each(|buf| buffer.extend_from_slice(buf));
        buffer
    }
}

impl<'a> From<Vec<&'a [u8]>> for FormattedData<'a> {
    fn from(slices: Vec<&'a [u8]>) -> Self {
        Self(slices.into_iter())
    }
}

impl<'a> Iterator for FormattedData<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

///
/// Formats the data buffer to be sent over Telnet.
///
/// # Examples
/// ```rust
/// use std::io::Write;
///
/// let mut telnet_stream = Vec::new();
/// let data = [1, 2, 0xFF, 3, 4];
///
/// for buf in telnet::format::data(&data) {
///     telnet_stream.write_all(buf).unwrap();
/// }
///
/// assert_eq!(telnet_stream, vec![1, 2, 0xFF, 0xFF, 3, 4]);
/// ```
///
pub fn data(buffer: &[u8]) -> FormattedData {
    let mut slices = Vec::new();
    let mut start = 0;

    for i in 0..buffer.len() {
        if buffer[i] == BYTE_IAC {
            slices.push(&buffer[start..(i + 1)]);
            start = i;
        }
    }

    if start < buffer.len() {
        slices.push(&buffer[start..]);
    }

    slices.into()
}

///
/// Formats a Telnet negotiation to be sent over Telnet.
///
/// # Examples
/// ```rust
/// use telnet::{NegotiationAction, TelnetOption};
///
/// let buffer = telnet::format::negotiation(NegotiationAction::Will, TelnetOption::Echo);
/// assert_eq!(buffer, vec![0xFF, 0xFB, 0x01].into_boxed_slice());
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
/// let parameters = [1, 2, 3];
/// let buffer = telnet::format::sub_negotiation(TelnetOption::TTYPE, &parameters);
/// assert_eq!(buffer, vec![0xFF, 0xFA, 24, 1, 2, 3, 0xFF, 0xF0].into_boxed_slice());
/// ```
///
pub fn sub_negotiation(opt: TelnetOption, parameters: &[u8]) -> Box<[u8]> {
    let mut buffer = data(parameters).append_to(vec![BYTE_IAC, BYTE_SB, opt.to_byte()]);
    buffer.extend_from_slice(&[BYTE_IAC, BYTE_SE]);
    buffer.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escape_test(buffer: &[u8], result: Box<[u8]>) {
        let buffer = data(buffer).to_owned();
        assert_eq!(buffer, result);
    }

    #[test]
    fn escape_double_iac() {
        escape_test(
            &[1, 2, 0xFF, 0xFF, 3, 4],
            vec![1, 2, 0xFF, 0xFF, 0xFF, 0xFF, 3, 4].into(),
        );
    }

    #[test]
    fn escape_no_iac() {
        escape_test(&[1, 2, 3, 4], vec![1, 2, 3, 4].into());
    }

    #[test]
    fn escape_iac_only() {
        escape_test(&[0xFF], vec![0xFF, 0xFF].into());
    }

    #[test]
    fn escape_iac_sub_negotiation() {
        let parameters = [1, 0xFF, 3];
        let buffer = sub_negotiation(TelnetOption::TTYPE, &parameters);
        assert_eq!(
            buffer,
            vec![0xFF, 0xFA, 24, 1, 0xFF, 0xFF, 3, 0xFF, 0xF0].into()
        );
    }
}
