#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscordKind {
    Stable,
    Ptb,
    Canary,
    Development,
}
impl From<DiscordKind> for String {
    #[inline(always)]
    fn from(kind: DiscordKind) -> Self {
        match kind {
            DiscordKind::Stable => "Discord",
            DiscordKind::Ptb => "DiscordPTB",
            DiscordKind::Canary => "DiscordCanary",
            DiscordKind::Development => "DiscordDevelopment",
        }.to_string()
    }
}
impl ToString for DiscordKind {
    #[inline(always)]
    fn to_string(&self) -> String {
        String::from(*self)
    }
}
impl From<String> for DiscordKind {
    #[inline(always)]
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "stable" => DiscordKind::Stable,
            "ptb" => DiscordKind::Ptb,
            "canary" => DiscordKind::Canary,
            "development" => DiscordKind::Development,
            _ => panic!("Invalid DiscordKind: {}", kind),
        }
    }
}