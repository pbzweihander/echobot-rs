use super::{SlackUser, SlackChannel};

#[derive(Serialize, Deserialize)]
pub struct RTMConnect {
    pub ok: bool,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UsersList {
    pub ok: bool,
    pub members: Vec<SlackUser>,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelsList {
    pub ok: bool,
    pub channels: Vec<SlackChannel>,
}
