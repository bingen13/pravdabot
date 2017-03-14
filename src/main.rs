//! Pravdabot, a 1930s Soviet themed mafia game IRC bot.

extern crate irc;
use irc::client::prelude::*;
use irc::client::data::command::Command;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};

#[cfg(test)]
mod test;

pub mod model;
use model::*;

/// Function to send messages.
/// It takes a target string, a message string, and an IrcServer object.
pub fn send(c: &String, m: &String, s: IrcServer) {
    s.send_privmsg(c, m).unwrap();
}

/// Processes incoming messages.
/// Takes a Message and a channel sender to issue events to.
pub fn process_cmd(msg: Message, tx: &Sender<GameEvent>) {
    match msg.prefix {
        Some(ref s) => print!("Message from {}: ", s),
        _ => (),
    }
    match msg.command {
        Command::PRIVMSG(ref s1, ref s2) => println!("to {} containing {}", s1, s2),
        _ => print!("{}", msg.to_string()),
    }
    let mstr = msg.to_string();
    if mstr.contains("exitnow") {
        println!("Received Quit command, sending IRC quit event.");
        tx.send(GameEvent::Notice("".to_string(),
                                  "Owner asked me to quit, do vstrechi!".to_string()))
          .unwrap();
        tx.send(GameEvent::Quit).unwrap()
    }
}

fn main() {

    println!("Welcome to CCCP. Building datastructures...");
    let my_server = IrcServer::new("pravda.json").unwrap();
    let s = my_server.clone();
    let my_chan = &my_server.config().clone().channels.unwrap()[0];
    s.identify().unwrap();

    let (tx, rx) = channel();

    let tx2 = tx.clone();

    let _ = thread::spawn(move || {
        thread::sleep(Duration::new(10, 0));
        tx.send(GameEvent::None).unwrap();
        thread::sleep(Duration::new(300, 0));
        tx.send(GameEvent::Quit).unwrap();
    });

    let s2 = my_server.clone();

    let _ = thread::spawn(move || {
        for msg in s2.iter() {
            match msg {
                Ok(m_r) => process_cmd(m_r, &tx2),
                _ => break,
            }
        }
    });


    loop {
        let event = rx.recv().unwrap();
        println!("Got event!");
        match event {
            GameEvent::Quit => {
                println!("Quit event received! Quitting...");
                thread::sleep(Duration::new(1, 0));
                s.send_quit(&"Pravda goes bye-bye!".to_string()).unwrap();
                break;
            }
            GameEvent::Notice(ref str1, ref str2) => {
                if str1 != "" {
                    send(str1, str2, s.clone());
                } else {
                    send(my_chan, str2, s.clone());
                }
            }
            _ => (),
        }
    }

}
