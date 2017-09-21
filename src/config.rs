#[derive(Serialize, Deserialize)]
pub struct Config {
    pub irc: IRCConfig,
    pub slack: SlackConfig,
}

#[derive(Serialize, Deserialize)]
pub struct IRCConfig {
    pub server: String,
    pub nickname: String,
    pub channel: String,
}

#[derive(Serialize, Deserialize)]
pub struct SlackConfig {
    pub token: String,
    pub channel: String,
}
