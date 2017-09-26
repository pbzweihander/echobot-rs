extern crate regex;

use std::error::Error;
use std::io::{BufReader, Lines};
use std::io::prelude::*;
use std::net::TcpStream;
use std::iter::Peekable;

use self::regex::Regex;

pub struct IRC {
    stream: TcpStream,
    reader: Peekable<Lines<BufReader<TcpStream>>>,
}

impl IRC {
    pub fn new(uri: &str, nickname: &str) -> Result<Self, Box<Error>> {
        let strm = TcpStream::connect(uri)?;
        let mut stream = strm.try_clone()?;
        let reader = BufReader::new(strm).lines().peekable();

        stream.write_all(
            format!("USER {} 0 * :zweihander-bot\n", nickname)
                .as_bytes(),
        )?;
        stream.write_all(format!("NICK {}\n", nickname).as_bytes())?;
        stream.flush()?;

        Ok(IRC { stream, reader })
    }

    pub fn try_clone(&mut self) -> Result<IRC, Box<Error>> {
        let stream1 = self.stream.try_clone()?;
        let stream2 = stream1.try_clone()?;
        let reader = BufReader::new(stream1).lines().peekable();

        Ok(IRC {
            stream: stream2,
            reader,
        })
    }

    pub fn write(&mut self, content: &str) -> Result<(), Box<Error>> {
        self.stream.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn writeln(&mut self, content: &str) -> Result<(), Box<Error>> {
        let mut s = content.to_owned();
        s.push('\n');
        self.write(&s)
    }

    pub fn privmsg(&mut self, channel: &str, message: &str) -> Result<(), Box<Error>> {
        self.writeln(&format!("PRIVMSG {} :{}", channel, message))?;
        Ok(())
    }

    pub fn join_multi(&mut self, channels: &[&str]) -> Result<(), Box<Error>> {
        for c in channels {
            self.join(c)?;
        }
        Ok(())
    }

    pub fn join(&mut self, channel: &str) -> Result<(), Box<Error>> {
        self.writeln(&format!("JOIN {}", channel))?;
        Ok(())
    }

    pub fn peek(&mut self) -> Option<&Result<String, ::std::io::Error>> {
        self.reader.peek()
    }

    fn parse_privmsg(content: String) -> Option<Message> {
        let r = Regex::new(r"^:(.+?)!.+ PRIVMSG (#.+?) :(.+)").unwrap();
        let caps = r.captures(&content);
        if caps.is_none() {
            return None;
        }
        let caps = caps.unwrap();

        let user = match caps.get(1) {
            Some(s) => s.as_str(),
            None => {
                return None;
            }
        };
        let channel = match caps.get(2) {
            Some(s) => s.as_str(),
            None => {
                return None;
            }
        };
        let text = match caps.get(3) {
            Some(s) => s.as_str(),
            None => {
                return None;
            }
        };

        Some(Message {
            channel: channel.to_owned(),
            user: user.to_owned(),
            text: text.to_owned(),
        })
    }
}

impl Iterator for IRC {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.reader.next().and_then(|l| l.ok());
            if line.is_none() {
                return None;
            }
            let line: String = line.unwrap();
            if line.contains("PING") {
                self.writeln(&line.replace("PING", "PONG")).ok();
            } else {
                let line = IRC::parse_privmsg(line);
                if line.is_some() {
                    return line;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub channel: String,
    pub user: String,
    pub text: String,
}
