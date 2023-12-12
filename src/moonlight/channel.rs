
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Stable,
    Nightly,
    Git 
}

impl From<String> for Channel {
    #[inline(always)]
    fn from(channel: String) -> Self {
        match channel.as_str() {
            "stable" => Channel::Stable,
            "nightly" => Channel::Nightly,
            "git" => Channel::Git,
            _ => panic!("Invalid channel: {}", channel),
        }
    }
}