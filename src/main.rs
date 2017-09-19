extern crate serde;
extern crate serde_json;
extern crate echobot;

use echobot::slack::Slack;
use echobot::irc::IRC;
use echobot::config::*;
use std::fs::File;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let config_file_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.json"
    };
    let config_file = File::open(config_file_path).unwrap();
    let config: Config = serde_json::from_reader(config_file).unwrap();

    let mut irc = IRC::new(&config.irc.server, &config.irc.nickname).unwrap();
    irc.join_multi(&config.irc.channels.iter().map(AsRef::as_ref).collect::<Vec<_>>()).unwrap();

    for line in irc {
        println!("{}", line);
    }

    // let slack = Slack::new(&config.slack.token).unwrap();
    // let rtm = slack.rtm_connect().unwrap();

    // for m in slack.users_list().unwrap() {
    //     println!("{}", m.name);
    // }

    // for c in slack.channels_list().unwrap() {
    //     println!("{}", c.name);
    // }

    // for l in rtm {
    //     println!("{}", l.text);
    // }
}
