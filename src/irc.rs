extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate tungstenite;
extern crate url;

use std::error::Error;
use std::io::{BufReader, Lines};
use std::io::prelude::*;
use std::net::TcpStream;

pub struct IRC {
    stream: TcpStream,
    reader: Lines<BufReader<TcpStream>>,
}

impl IRC {
    pub fn new(uri: &str, nickname: &str) -> Result<Self, Box<Error>> {
        let strm = TcpStream::connect(uri)?;
        let mut stream = strm.try_clone()?;
        let reader = BufReader::new(strm).lines();

        stream.write(&format!("USER {} 0 * :zweihander-bot\n", nickname).into_bytes())?;
        stream.write(&format!("NICK {}\n", nickname).into_bytes())?;
        stream.flush()?;

        Ok(IRC { stream, reader })
    }

    pub fn write(&mut self, content: &str) -> Result<(), Box<Error>> {
        self.stream.write(content.as_bytes())?;
        Ok(())
    }

    pub fn join_multi(&mut self, channels: &[&str]) -> Result<(), Box<Error>> {
        for c in channels {
            self.join(c)?;
        }
        Ok(())
    }

    pub fn join(&mut self, channel: &str) -> Result<(), Box<Error>> {
        self.write(&format!("JOIN {}\n", channel))?;
        Ok(())
    }
}

impl Iterator for IRC {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.reader.next().and_then(|l| l.ok());

        line
    }
}
