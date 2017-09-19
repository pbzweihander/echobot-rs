extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate tungstenite;
extern crate url;

use ::std::error::Error;
use ::std::collections::HashMap;

use self::response::*;

pub struct Slack {
    token: String
}

impl Slack {
    pub fn new(token: &str) -> Result<Self, Box<Error>> {
        Ok(Slack { token: String::from(token) })
    }

    pub fn request<R: serde::de::DeserializeOwned>(&self, api: &str, argument: HashMap<String, String>) -> Result<R, Box<Error>> {
        let mut uri = String::from("https://slack.com/api/");
        uri.push_str(api);
        uri.push('?');
        for (key, val) in argument.iter() {
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

    pub fn rtm_connect(&self) -> Result<SlackRTM, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: RTMConnect = self.request("rtm.connect", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        let (socket, _) = tungstenite::connect(url::Url::parse(&parsed.url)?)?;

        Ok(SlackRTM { socket })
    }

    pub fn users_list(&self) -> Result<Vec<SlackUser>, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: UsersList = self.request("users.list", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.members)
    }

    pub fn channels_list(&self) -> Result<Vec<SlackChannel>, Box<Error>> {
        let mut hm = HashMap::new();
        hm.insert("token".to_owned(), self.token.clone());
        let parsed: ChannelsList = self.request("channels.list", hm)?;

        if !parsed.ok {
            return Err(Box::new(::std::io::Error::new(::std::io::ErrorKind::Other, "Slack Response Error")));
        }

        Ok(parsed.channels)
    }
}

pub struct SlackRTM {
    socket: tungstenite::WebSocket<tungstenite::client::AutoStream>
}

impl SlackRTM {
    fn parse(content: tungstenite::Message) -> Option<SlackMessage> {
        if let tungstenite::Message::Text(t) = content {
            serde_json::from_str(&t).ok()
        } else {
            None
        }
    }
}

impl Iterator for SlackRTM {
    type Item = SlackMessage;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.socket.read_message();
            if line.is_err() {
                return None;
            }
            let m = line.unwrap();
            if m.is_ping() {
                if self.socket.write_message(tungstenite::Message::Pong(vec![])).is_err() {
                    return None;
                }
            }

            let p = SlackRTM::parse(m);

            if p.is_some() {
                return p;
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub user: String,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
}

mod response;
