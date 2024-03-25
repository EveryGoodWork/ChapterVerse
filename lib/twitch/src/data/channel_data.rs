#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    NotConnected,
    Connecting,
    Connected,
    Invalid,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Channel {
    pub(crate) name: String,
    pub state: ChannelState,
}
