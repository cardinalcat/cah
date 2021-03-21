#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;
pub mod game;
pub mod network;

use ws::listen;

use crate::network::{Operation, Packet};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum CahError {
    #[error(display = "{}", _0)]
    JsonError(#[source] serde_json::Error),
    #[error(display = "{}", _0)]
    Utf8Error(#[source] std::string::FromUtf8Error),
}

pub struct GameEngine {
    games: Arc<RwLock<HashMap<String, (Sender<Packet>, JoinHandle<Result<(), CahError>>)>>>,
}

impl GameEngine {
    pub fn new() -> Self {
        let games: Arc<
            RwLock<HashMap<String, (Sender<Packet>, JoinHandle<Result<(), CahError>>)>>,
        > = Arc::new(RwLock::new(HashMap::new()));
        Self { games }
    }
    pub async fn run(self) -> Result<(), CahError> {
        //let packet_receiver = Arc::new(Mutex::new(packet_receiver));
        //let all_games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(Mutex::new(Vec::new()));
        // Listen on an address and call the closure for each connection
        if let Err(error) = listen("0.0.0.0:3012", |out| {
            let (packet_sender, mut packet_receiver): (Sender<Packet>, Receiver<Packet>) = channel(100);
            //let packet_receiver = packet_receiver.clone();
            tokio::spawn(async move {
                loop {
                    let packet = packet_receiver.recv().await.unwrap();
                    out.send(ws::Message::Text(serde_json::to_string(&packet).unwrap())).unwrap();
                }
            });
            //let sender = sender.clone();
            //let pkt_sender = Arc::new(Mutex::new(packet_sender));
            let games = self.games.clone();
            move |msg: ws::Message| {
                //let game_list = games_cloned.clone();
                //let pkt_sender = cloned.clone();
                handle_message(msg, packet_sender.clone(), games.clone());
                Ok(())
            }
        }) {
            // Inform the user of failure
            println!("Failed to create WebSocket due to {:?}", error);
        };
        Ok(())
    }
}

async fn handle_message(
    msg: ws::Message,
    mut packet_sender: Sender<Packet>,
    games: Arc<RwLock<HashMap<String, (Sender<Packet>, JoinHandle<Result<(), CahError>>)>>>,
) -> Result<(), CahError> {
    //let mut packet_sender = packet_sender.lock().await;
    let msg_text: String = match msg {
        ws::Message::Text(txt) => txt,
        ws::Message::Binary(bin) => match String::from_utf8(bin) {
            Ok(txt) => txt,
            Err(e) => {
                packet_sender
                    .send(Packet::report_error(format!("{}", e)))
                    .await.unwrap();
                return Err(CahError::from(e));
            }
        },
    };
    let packet: Packet = match serde_json::from_str(&msg_text) {
        Ok(packet) => packet,
        Err(e) => {
            packet_sender.send(Packet::report_error(format!("{}", e))).await.unwrap();
            return Err(CahError::from(e));
        }
    };

    match packet.get_task() {
        Operation::StartGame => Ok(()),
        Operation::EndGame => Ok(()),
        _ => match games.write().await.get_mut(packet.get_gameid()) {
            Some((game, _)) => match game.send(packet).await {
                Ok(_) => Ok(()),
                Err(e) => Ok(packet_sender
                    .send(Packet::report_error(String::from("game has ended")))
                    .await
                    .unwrap()),
            },
            None => Ok(packet_sender
                .send(Packet::report_error(String::from("game doesn't exist")))
                .await
                .unwrap()),
        },
    }
}
