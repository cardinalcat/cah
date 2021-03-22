#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref IDMANAGER: Arc<Mutex<IDManager>> = Arc::new(Mutex::new(IDManager::default()));
}

/// this handles main game logic including handiling requests other then ending the game
/// and starting a game
pub mod game;
/// this just defines types used to communicated between the client and the server
/// principally it defines Operation, and PacketType that set out a purpose for each communication
pub mod network;
use crate::game::Game;

use ws::listen;

use crate::network::{Operation, Packet};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

/// keeps track of used, and unused IDs
pub struct IDManager {
    game_count: u16,
    used: Vec<u16>,
}

impl Default for IDManager {
    fn default() -> IDManager {
        Self {
            game_count: 0,
            used: Vec::new(),
        }
    }
}

impl IDManager {
    pub fn get_id(&mut self) -> Result<u16, CahError> {
        if self.game_count == 65535 {
            return match self.used.pop() {
                Some(val) => Ok(val),
                None => Err(CahError::MaxGames),
            };
        }
        self.game_count += 1;
        Ok(self.game_count - 1)
    }
}

/// this is most just a derived error that wraps other error types
#[derive(Debug, Error)]
pub enum CahError {
    #[error(display = "{}", _0)]
    JsonError(#[source] serde_json::Error),
    #[error(display = "{}", _0)]
    Utf8Error(#[source] std::string::FromUtf8Error),
    #[error(display = "{}", _0)]
    WsErr(#[source] ws::Error),
    #[error(display = "no more games can be play on this server at this time")]
    MaxGames,
}

/// this is a convenience type to run function that are too long
/// to look pretty in main
pub struct GameEngine {
    games: GameStore,
}

impl Default for GameEngine {
    fn default() -> GameEngine {
        GameEngine::new()
    }
}

type GameStore = Arc<RwLock<HashMap<String, (Sender<Packet>, JoinHandle<Result<(), CahError>>)>>>;

impl GameEngine {
    pub fn new() -> Self {
        let games: GameStore = Arc::new(RwLock::new(HashMap::new()));
        Self { games }
    }
    pub async fn run(self) -> Result<(), CahError> {
        //let packet_receiver = Arc::new(Mutex::new(packet_receiver));
        //let all_games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(Mutex::new(Vec::new()));
        // Listen on an address and call the closure for each connection

        if let Err(error) = listen("0.0.0.0:3012", |out| {
            //let games = self.games.clone();
            let (switch_send, mut switch_recv) = channel(5);
            tokio::spawn(async move {
                let (msg, games, out) = switch_recv.recv().await.unwrap();
                handle_message(msg, games, out).await.unwrap()
            });
            //let switch_send_clone = switch_send.clone();
            let games = self.games.clone();
            move |msg: ws::Message| {
                // likely a lot of overhead here
                let mut switch_send_clone_two = switch_send.clone();
                let game_list = games.clone();
                let out_send = out.clone();
                tokio::spawn(async move {
                    switch_send_clone_two.send((msg, game_list, out_send)).await
                });
                Ok(())
            }
        }) {
            // Inform the user of failure
            println!("Failed to create WebSocket due to {:?}", error);
        };
        Ok(())
    }
}

/// usually returns Ok(()) even when errors occure so the worker doesn't panic
async fn handle_message(
    msg: ws::Message,
    games: GameStore,
    out: ws::Sender,
) -> Result<(), CahError> {
    //let mut out = out.lock().await;
    let msg_text: String = match msg {
        ws::Message::Text(txt) => txt,
        ws::Message::Binary(bin) => match String::from_utf8(bin) {
            Ok(txt) => txt,
            Err(e) => {
                out.send(Packet::report_error(format!("{}", e))).unwrap();
                return Ok(());
            }
        },
    };
    let packet: Packet = match serde_json::from_str(&msg_text) {
        Ok(packet) => packet,
        Err(e) => {
            out.send(Packet::report_error(format!("{}", e))).unwrap();
            return Ok(());
        }
    };

    match packet.get_task() {
        Operation::StartGame => {
            // I forgot the on who starts the game is also a player, so out also
            // needs to be sent to that game thread
            let (sender, receiver): (Sender<Packet>, Receiver<Packet>) = channel(100);
            let mut hash = IDMANAGER.lock().await;
            //games.write().await.insert(hash.to_string(), sender);
            let hash_value: u16 = match hash.get_id() {
                Ok(id) => id,
                Err(e) => {
                    out.send(Packet::report_error(format!("{}", e)))?;
                    return Ok(());
                }
            };
            let handler = tokio::spawn(async move {
                Game::handle(hash_value, receiver, packet.clone(), out).await
            });
            games.write().await.insert(
                
                hash_value.to_string(),
                (sender, handler),
            );
            Ok(())
        }
        Operation::EndGame => {
            // remove the game object, then wait for a sec for the game loop to clean up
            // if it doesn't then abort it
            Ok(())
        }
        _ => match games.write().await.get_mut(packet.get_gameid()) {
            Some((game, _)) => match game.send(packet).await {
                Ok(_) => Ok(()),
                Err(_e) => {
                    out.send(Packet::report_error(String::from("game has ended")))?;
                    Ok(())
                }
            },
            None => {
                out.send(Packet::report_error(String::from("game doesn't exist")))
                    .unwrap();
                Ok(())
            }
        },
    }
}
