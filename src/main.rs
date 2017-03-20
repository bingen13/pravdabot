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
        Command::PRIVMSG(ref s1, ref s2) => {
            let nick = msg.source_nickname().unwrap();
            println!("to {} containing {}", s1, s2);
            let cmd = s2.trim();
            if cmd.starts_with('!') {
                // We're getting a command.
                let cmd_words = cmd.split_whitespace().collect::<Vec<_>>();
                match cmd_words[0].to_lowercase().as_str() {
                    "!join" => {
                        if s1.starts_with("#") {
                            tx.send(GameEvent::Join(nick.to_string())).unwrap();
                        } else {
                            tx.send(GameEvent::Notice(nick.to_string(),
                                                      "This command must be issued in public."
                                                          .to_string()))
                              .unwrap();
                        }
                    }
                    _ => println!("Unimplemented command: {}", cmd_words[0]),
                }
            }
        }
        Command::NICK(_) | Command::QUIT(_) | Command::PART(_, _) => {
            let nick = msg.source_nickname().unwrap();
            tx.send(GameEvent::Leave(nick.to_string())).unwrap();
        }
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

/// Deliver takes a GameReaction and delivers all its mesages out.
fn deliver(gr: &GameReaction, tx: &Sender<GameEvent>) {
    for i in gr.msg.iter() {
        match i.recipients {
            Recipients::Channel(ref s) => {
                tx.send(GameEvent::Notice(s.clone(), i.content.clone())).unwrap();
            }
            Recipients::Nicks(ref v) => {
                for j in v.iter() {
                    tx.send(GameEvent::Notice(j.clone(), i.content.clone())).unwrap();
                }
            }
        }
    }
}


fn main() {

    println!("Welcome to CCCP. Building datastructures...");
    let my_server = IrcServer::new("pravda.json").unwrap();
    let s = my_server.clone();
    let my_chan = &my_server.config().clone().channels.unwrap()[0];
    let mut my_game = Game::new(&my_chan.clone());
    s.identify().unwrap();

    let (tx, rx) = channel();

    let tx2 = tx.clone();
    let tx3 = tx.clone();

    // This thread ticks every second.
    let _ = thread::spawn(move || {
        // Give the game time to connect.
        thread::sleep(Duration::new(30, 0));
        loop {
            thread::sleep(Duration::new(1, 0));
            tx.send(GameEvent::Tick).unwrap();
        }
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
            _ => {
                // Here we must process the game event. Game::process does the
                // relevant mutations, but we also require a function to send
                // the messages to the IRC server here.
                my_game = my_game.process(event.clone());
                for i in my_game.pending.iter() {
                    deliver(i, &tx3);
                }
                my_game = my_game.clean_up();
            }
        }
    }

}
