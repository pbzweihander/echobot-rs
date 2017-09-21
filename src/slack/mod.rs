extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate tungstenite;
extern crate url;

use ::std::error::Error;
use ::std::collections::HashMap;
use ::std::iter::Peekable;

use self::response::*;

pub struct SlackRequest {
    token: String,
}

impl SlackRequest {
    pub fn request<R: serde::de::DeserializeOwned>(api: &str, argument: HashMap<String, String>) -> Result<R, Box<Error>> {
        let mut uri = String::from("https://slack.com/api/");
        uri.push_str(api);
        uri.push('?');
        for (key, val) in &argument {
            uri.push_str(key);
            uri.push('=');
            uri.push_str(val);
            uri.push('&');
        }
        uri.pop();

        let resp = reqwest::get(&uri)?;

        let parsed: R = serde_json::from_reader(resp)?;

        Ok(parsed)
    }

    pub fn rtm_connect(&self) -> Result<Peekable<SlackRTM>, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: RTMConnect = SlackRequest::request("rtm.connect", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        let (socket, _) = tungstenite::connect(url::Url::parse(&parsed.url)?)?;

        Ok(SlackRTM { socket }.peekable())
    }

    pub fn users_list(&self) -> Result<Vec<User>, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: UsersList = SlackRequest::request("users.list", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.members)
    }

    pub fn users_info(&self, id: &str) -> Result<User, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        hm.insert("user".to_owned(), id.to_owned());
        let parsed: UsersInfo = SlackRequest::request("users.info", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.user)
    }

    pub fn channels_list(&self) -> Result<Vec<Channel>, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: ChannelsList = SlackRequest::request("channels.list", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.channels)
    }

    pub fn channels_info(&self, id: &str) -> Result<Channel, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        hm.insert("channel".to_owned(), id.to_owned());
        let parsed: ChannelsInfo = SlackRequest::request("channels.info", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.channel)
    }
}

pub struct Slack {
    pub request: SlackRequest,
    user_hashmap: HashMap<String, User>,
    channel_hashmap: HashMap<String, Channel>,
}

impl Slack {
    pub fn new(token: &str) -> Result<Self, Box<Error>> {
        Ok(Slack { request: SlackRequest { token: token.to_owned() }, user_hashmap: HashMap::new(), channel_hashmap: HashMap::new() })
    }

    fn make_user_hashmap(&mut self) -> Result<(), Box<Error>> {
        let users = self.request.users_list()?;
        for u in users {
            self.user_hashmap.insert(u.id.clone(), u);
        }
        Ok(())
    }

    fn make_channel_hashmap(&mut self) -> Result<(), Box<Error>> {
        let channels = self.request.channels_list()?;
        for c in channels {
            self.channel_hashmap.insert(c.id.clone(), c);
        }
        Ok(())
    }

    pub fn get_user(&mut self, id: &str) -> Result<User, Box<Error>> {
        use std::collections::hash_map::Entry::*;
        if self.user_hashmap.keys().len() == 0 {
            self.make_user_hashmap()?;
        }
        Ok(match self.user_hashmap.entry(id.to_owned()) {
            Occupied(u) => u.get().clone(),
            Vacant(entry) => {
                let u = self.request.users_info(id)?;
                entry.insert(u).clone()
            }
        })
    }

    pub fn get_channel(&mut self, id: &str) -> Result<Channel, Box<Error>> {
        use std::collections::hash_map::Entry::*;
        if self.channel_hashmap.keys().len() == 0 {
            self.make_channel_hashmap()?;
        }
        Ok(match self.channel_hashmap.entry(id.to_owned()) {
            Occupied(c) => c.get().clone(),
            Vacant(entry) => {
                let c = self.request.channels_info(id)?;
                entry.insert(c).clone()
            }
        })
    }

    pub fn raw_message_to_message(&mut self, m: RawMessage) -> Result<Message, Box<Error>> {
        let c = self.get_channel(&m.channel)?;
        let u = self.get_user(&m.user)?;
        Ok(Message{ channel: c, user: u, text: m.text.clone() })
    }
}

pub struct SlackRTM {
    socket: tungstenite::WebSocket<tungstenite::client::AutoStream>,
}

impl SlackRTM {
    fn parse(content: tungstenite::Message) -> Option<RawMessage> {
        if let tungstenite::Message::Text(t) = content {
            serde_json::from_str(&t).ok()
        } else {
            None
        }
    }
}

impl Iterator for SlackRTM {
    type Item = RawMessage;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.socket.read_message();
            if line.is_err() {
                return None;
            }
            let m = line.unwrap();

            let p = SlackRTM::parse(m);

            if p.is_some() {
                return p;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RawMessage {
    pub channel: String,
    pub user: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub channel: Channel,
    pub user: User,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

mod response;
