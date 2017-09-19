use super::{User, Channel};

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
pub struct ChannelsList {
    pub ok: bool,
    pub channels: Vec<Channel>,
}
