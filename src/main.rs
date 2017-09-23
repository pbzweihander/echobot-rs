extern crate echobot;
extern crate serde_json;

use std::error::Error;
use std::fs::File;
use std::env;
use std::thread;
use std::sync::mpsc;
use std::iter::Peekable;

use echobot::slack;
use echobot::slack::Slack;
use echobot::irc;
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

    let (irc_message_tx, irc_message_rx): (mpsc::Sender<irc::Message>, mpsc::Receiver<irc::Message>) = mpsc::channel();
    let (slack_message_tx, slack_message_rx): (mpsc::Sender<slack::Message>, mpsc::Receiver<slack::Message>) = mpsc::channel();

    let irc_channel = config.irc.channel.clone();
    let slack_channel = config.slack.channel.clone();

    let mut irc: IRC = IRC::new(&config.irc.server, &config.irc.nickname)?;

    let mut slack: Slack = Slack::new(&config.slack.token)?;
    let mut rtm: Peekable<slack::SlackRTM> = slack.request.rtm_connect()?;

    irc.join(&irc_channel)?;

    let irc_thread = thread::Builder::new().name("IRC Thread".to_owned()).spawn(move || loop {
        if irc.peek().is_some() {
            let m = irc.next().unwrap();
            println!("IRC: <{}> {}", m.user, m.text);
            if irc_channel == m.channel {
                irc_message_tx.send(m).unwrap();
            }
            for m in slack_message_rx.try_iter() {
                irc.privmsg(&irc_channel, &format!("<{}> {}", m.user.name, m.text)).unwrap();
            }
        }
    })?;

    let slack_thread = thread::Builder::new().name("Slack Thread".to_owned()).spawn(move || loop {
        if rtm.peek().is_some() {
            let m = slack.raw_message_to_message(rtm.next().unwrap()).unwrap();
            println!("Slk: <{}> {}", m.user.name, m.text);
            if slack_channel == m.channel.name {
                slack_message_tx.send(m).unwrap();
            }
            for m in irc_message_rx.try_iter() {
                slack.request.chat_post_message(&slack_channel, &format!("<{}> {}", m.user, m.text)).unwrap();
            }
        }
    })?;

    irc_thread.join().ok();
    slack_thread.join().ok();

    Ok(())
}

fn main() {
    if let Err(e) = init() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
