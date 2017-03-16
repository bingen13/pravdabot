//! Module: model.
//! This module contains the game model: data structures and functions
//! to handle them more or less independently of the communication and control parts.

/// Roles for the game.
pub enum Role {
    Worker,
    Saboteur,
    Commissar,
    Chekist,
    Militya,
    Cosmopolitan,
    Spy,
    Mastermind,
    Stalin,
}

/// Teams. Spies start neutral. They win if they survive to the end.
pub enum Team {
    Soviet,
    Foreign,
    Opposition,
}

/// Gun state.
pub enum Gun {
    Loaded,
    Unloaded,
    Broken,
}

/// Player structure.
pub struct Player {
    pub nick: String,
    pub role: Role,
    pub turn_actions: u8,
    pub game_actions: u8,
    pub alive: bool,
    pub day_voter: bool,
    pub night_voter: bool,
    pub real_team: Team,
    pub apparent_team: Team,
}

impl Player {
    /// Method to define new players.
    pub fn new(nick: String, r: Role) -> Player {
        let p = Player {
            nick: nick,
            alive: true,
            day_voter: true,
            night_voter: match r {
                Role::Worker => false,
                _ => true,
            },
            turn_actions: 1,
            game_actions: match r {
                Role::Cosmopolitan |
                Role::Mastermind |
                Role::Spy |
                Role::Worker |
                Role::Saboteur |
                Role::Commissar |
                Role::Militya => 0,
                Role::Chekist => 8,
                Role::Stalin => 1,
            },
            real_team: match r {
                Role::Saboteur | Role::Mastermind => Team::Opposition,
                Role::Spy => Team::Foreign,
                _ => Team::Soviet,
            },
            apparent_team: match r {
                Role::Cosmopolitan | Role::Saboteur | Role::Mastermind => Team::Opposition,
                Role::Spy => Team::Foreign,
                _ => Team::Soviet,
            },
            role: r,
        };
        return p;
    }

    /// Outputs printout of the given player.
    pub fn to_string(&self) -> String {
        let role = match self.role {
            Role::Worker => "glorious Soviet worker",
            Role::Saboteur => "social fascist Trotskyite saboteur",
            Role::Cosmopolitan => "rootless cosmopolitan Mensch",
            Role::Stalin => "general secretary of the Party",
            Role::Spy => "capitalist agent of influence",
            Role::Mastermind => "dangerous Trotskyite theoretician",
            Role::Chekist => "uncompromising CHEKA member",
            Role::Militya => "self-sacrificing people's militsioner",
            Role::Commissar => "cunning political commissar",
        };
        let team_r = match self.real_team {
            Team::Soviet => "loyal citizen of the Union",
            Team::Foreign => "foreign meddler",
            Team::Opposition => "treasonous opposition member",
        };
        let team_a = match self.apparent_team {
            Team::Soviet => "loyal citizen of the Union",
            Team::Foreign => "foreign meddler",
            Team::Opposition => "treasonous opposition member",
        };
        let mut result = format!("You, {}, are a {} and look like a {}, while being a {}. ",
                                 self.nick,
                                 role,
                                 team_a,
                                 team_r);
        result = result +
                 &format!("You have {} game actions and {} turn actions left. ",
                          self.game_actions,
                          self.turn_actions);
        result = result +
                 match self.alive {
            true => "You are still alive. ",
            false => "You are no longer alive. ",
        };
        result = result +
                 match self.day_voter {
            true => "You are a full voting member of the Soviet. ",
            false => "You have lost their voting rights at the Soviet. ",
        };
        result = result +
                 match self.night_voter {
            true => "You are busy at night.",
            false => "You sleep at night.",
        };
        return result;
    }

    fn kill(&mut self) -> &Player {
        assert!(self.alive == true);
        self.alive = false;
        return self;
    }

    fn is_alive(&self) -> bool {
        return self.alive;
    }

    fn is_day_voter(&self) -> bool {
        return self.alive && self.day_voter;
    }
}

/// Phases of play.
pub enum Phase {
    Day(u8),
    Night(u8),
    Inactive,
    Starting(u8, u8),
}

// The game.

/// This structure contains either a vector with nick strings before game starts,
/// or a vector of Players after it does.
pub enum Participants {
    Joiners(Vec<String>),
    Players(Vec<Player>),
}

/// Recipients to send a message to. Either a channel string or a vector of nicks.
pub enum Recipients {
    Channel(String),
    Nicks(Vec<String>),
}

/// A message from the game, with its Recipients and its content.
pub struct GameMessage {
    pub recipients: Recipients,
    pub content: String,
}

impl GameMessage {
    /// Create a public GameMessage.
    fn public(ch: String, content: String) -> GameMessage {
        let r = Recipients::Channel(ch);
        let gm = GameMessage {
            recipients: r,
            content: content,
        };
        return gm;
    }
}

/// A game event.
#[derive(Clone)]
pub enum GameEvent {
    Join(String),
    Leave(String),
    Msg(String),
    Night(u8, u8),
    Day(u8, u8),
    Quit,
    None,
    Notice(String, String),
    Begin,
}

/// A game event and the messages it generates.
pub struct GameReaction {
    pub event: GameEvent,
    pub msg: Vec<GameMessage>,
}

impl GameReaction {
    fn new(e: &GameEvent) -> GameReaction {
        let gr = GameReaction {
            event: e.clone(),
            msg: Vec::new(),
        };
        return gr;
    }

    fn add(mut self, msg: GameMessage) -> GameReaction {
        self.msg.push(msg);
        return self;
    }
}


/// The game state.
pub struct Game {
    pub gun: Gun,
    pub players: Participants,
    pub phase: Phase,
    pub channel: String,
    pub log: Vec<GameReaction>,
    pub pending: Vec<GameReaction>,
}

impl Game {
    pub fn new(ch: &String) -> Game {
        let s = Game {
            gun: Gun::Loaded,
            channel: ch.clone(),
            players: Participants::Joiners(Vec::new()),
            phase: Phase::Inactive,
            log: Vec::new(),
            pending: Vec::new(),
        };
        return s;
    }

    pub fn start(&mut self) {
        println!("To be done. Game starting..."); // TODO
    }

    /// Place pending reactions into log.
    pub fn clean_up(mut self: Game) -> Game {
        if self.pending.len() > 0 {
            {
                let m = &mut self.pending;
                self.log.append(m);
            }
            self.pending = Vec::new();
        }
        return self;
    }

    pub fn process(self, event: GameEvent) -> Game {
        match event {
            GameEvent::Join(_) => process_join(self, event),
            GameEvent::Leave(_) => process_leave(self, event),
            _ => {
                println!("Unimplemented event!");
                self
            }
        }
    }
}

/// Process joins.
fn process_join(mut g: Game, e: GameEvent) -> Game {
    let mut gr = GameReaction::new(&e);
    if let GameEvent::Join(ref nick) = e {
        let nick = nick.clone();
        match g.phase {
            Phase::Inactive => {
                g.phase = Phase::Starting(10, 10);
                g.players = Participants::Joiners(vec![nick.clone()]);
                let gm = GameMessage::public(g.channel.clone(),
                                             format!("{} starting new game!", nick));
                gr = gr.add(gm);
                g.pending.push(gr);
                g
            }
            Phase::Starting(_, _) => {
                if let Participants::Joiners(ref mut p) = g.players {
                    if !p.contains(&nick) {
                        let gm = GameMessage::public(g.channel.clone(),
                                                     format!("{} joins the game.", nick));
                        p.push(nick);
                        gr = gr.add(gm);
                        g.pending.push(gr);
                    } else {
                        let gm = GameMessage::public(g.channel.clone(),
                                                     format!("{} already joined.", nick));
                        gr = gr.add(gm);
                        g.pending.push(gr);
                    }
                }
                g
            }
            _ => {
                println!("Unimplemented phase for join!");
                g
            }
        }
    } else {
        g
    }
}

fn process_leave(mut g: Game, e: GameEvent) -> Game {
    let mut gr = GameReaction::new(&e);
    if let GameEvent::Leave(ref nick) = e {
        let nick = nick.clone();
        match g.phase {
            Phase::Starting(_, _) => {
                if let Participants::Joiners(ref mut p) = g.players {
                    if !p.contains(&nick) {
                        let gm = GameMessage::public(g.channel.clone(),
                                                     format!("{} hasn't joined yet.", nick));
                        gr = gr.add(gm);
                    } else {
                        let index = p.iter().position(|it| it == &nick).unwrap();
                        p.remove(index);
                        let gm = GameMessage::public(g.channel.clone(),
                                                     format!("{} has left the game.", nick));
                        gr = gr.add(gm);
                        if p.len() == 0 {
                            g.phase = Phase::Inactive;
                            let gm = GameMessage::public(g.channel.clone(),
                                                         "No players left. Game cancelled."
                                                             .to_string());
                            gr = gr.add(gm);
                        }
                    }
                    g.pending.push(gr);
                }
                g
            }
            _ => {
                println!("Leave event unimplementted for this phase.");
                g
            }
        }
    } else {
        g
    }
}
