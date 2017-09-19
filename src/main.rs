extern crate serde;
extern crate serde_json;
extern crate echobot;

use std::error::Error;
use std::fs::File;
use std::env;

use echobot::slack::Slack;
use echobot::irc::IRC;
use echobot::config::*;

fn init() -> Result<(), Box<Error>> {
    let args: Vec<_> = env::args().collect();
    let config_file_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.json"
    };
    let config_file = File::open(config_file_path)?;
    let config: Config = serde_json::from_reader(config_file)?;

    let mut irc = IRC::new(&config.irc.server, &config.irc.nickname)?;
    irc.join_multi(&config.irc.channels.iter().map(AsRef::as_ref).collect::<Vec<_>>())?;

    for line in irc {
        println!("{}", line);
    }

    // let slack = Slack::new(&config.slack.token)?;
    // let rtm = slack.rtm_connect()?;

    // for m in slack.users_list()? {
    //     println!("{}", m.name);
    // }

    // for c in slack.channels_list()? {
    //     println!("{}", c.name);
    // }

    // for l in rtm {
    //     println!("{}", l.text);
    // }

    Ok(())
}

fn main() {
    if let Err(e) = init() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
