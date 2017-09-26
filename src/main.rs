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

    let (irc_message_tx, irc_message_rx): (mpsc::Sender<irc::Message>,
                                           mpsc::Receiver<irc::Message>) = mpsc::channel();
    let (slack_message_tx, slack_message_rx): (mpsc::Sender<slack::Message>,
                                               mpsc::Receiver<slack::Message>) = mpsc::channel();

    let irc_channel = config.irc.channel.clone();
    let irc_channel2 = irc_channel.clone();
    let slack_channel = config.slack.channel.clone();
    let slack_channel2 = slack_channel.clone();

    let mut irc: IRC = IRC::new(&config.irc.server, &config.irc.nickname)?;

    let mut slack: Slack = Slack::new(&config.slack.token)?;
    let rtm: Peekable<slack::SlackRTM> = slack.request.rtm_connect()?;

    irc.join(&irc_channel)?;
    irc.privmsg(&irc_channel, "hi!")?;

    let slack2 = slack.clone();
    let mut irc2 = irc.try_clone()?;

    let irc_rx_thread = thread::Builder::new()
        .name("IRC Receiver Thread".to_owned())
        .spawn(move || for m in irc {
            println!("IRC: <{}> {}", m.user, m.text);
            if irc_channel == m.channel {
                irc_message_tx.send(m).unwrap();
            }
        })?;

    let irc_tx_thread = thread::Builder::new()
        .name("IRC Transmitter Thread".to_owned())
        .spawn(move || for m in slack_message_rx.iter() {
            irc2.privmsg(&irc_channel2, &format!("<{}> {}", m.user.name, m.text))
                .unwrap();
            println!("Slk -> IRC: <{}> {}", m.user.name, m.text);
        })?;

    let slack_rx_thread = thread::Builder::new()
        .name("Slack Receiver Thread".to_owned())
        .spawn(move || for m in rtm {
            let parsed = slack.raw_message_to_message(m).unwrap();
            println!("Slk: <{}> {}", parsed.user.name, parsed.text);
            if slack_channel == parsed.channel.name {
                slack_message_tx.send(parsed).unwrap();
            }
        })?;

    let slack_tx_thread = thread::Builder::new()
        .name("Slack Transmitter Thread".to_owned())
        .spawn(move || for m in irc_message_rx.iter() {
            slack2
                .request
                .chat_post_message(&slack_channel2, &format!("<{}> {}", m.user, m.text))
                .unwrap();
            println!("IRC -> Slk: <{}> {}", m.user, m.text);
        })?;

    irc_rx_thread.join().ok();
    irc_tx_thread.join().ok();
    slack_rx_thread.join().ok();
    slack_tx_thread.join().ok();

    Ok(())
}

fn main() {
    if let Err(e) = init() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
