use super::{User, Channel, RawMessage};

#[derive(Serialize, Deserialize)]
pub struct RTMConnect {
    pub ok: bool,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsersList {
    pub ok: bool,
    pub members: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct UsersInfo {
    pub ok: bool,
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelsList {
    pub ok: bool,
    pub channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelsInfo {
    pub ok: bool,
    pub channel: Channel,
}

#[derive(Serialize, Deserialize)]
pub struct ChatPostMessage {
    pub ok: bool,
    pub ts: String,
    pub channel: String,
    pub message: RawMessage,
}
