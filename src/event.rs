
pub enum TelnetEvent<'a> {
    DataReceived(&'a [u8])
}
