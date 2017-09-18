#[derive(Serialize, Deserialize)]
pub struct Config {
    pub irc: IRCConfig,
    pub slack: SlackConfig,
}

#[derive(Serialize, Deserialize)]
pub struct IRCConfig {
    pub server: String,
    pub nickname: String,
    pub channels: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SlackConfig {
    pub token: String,
}
