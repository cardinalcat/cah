/// Simple WebSocket server with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;

use std::boxed::Box;
use std::convert::TryInto;
use std::option::Option;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;

use ws::listen;

use cah_backend::game::{Game, User};
use cah_backend::network::{Operation, Packet, PacketType};

fn main() {
    // Setup logging
    let all_games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(Mutex::new(Vec::new()));

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("0.0.0.0:3012", |out| {
        let (tx, rx) = channel();
        let games = all_games.clone();
        thread::spawn(move || {
            let mut game_lock: Arc<Option<Arc<Mutex<Game>>>> = Arc::new(None);
            loop {
                let mut packet: Packet = match rx.recv() {
                    Ok(pack) => pack,
                    Err(e) => {
                        println!("error: {}", e);
                        break;
                    }
                };
                let gameid = packet.get_gameid().parse::<u16>().unwrap();
                if packet.get_task() == Operation::StartGame {
                    let mut game_vec = games.lock().unwrap();
                    let mut game_instance = Game::new(game_vec.len().try_into().unwrap());

                    let pack: Packet = Packet::new(
                        game_instance.get_hash(),
                        PacketType::Game,
                        Operation::StartGame,
                        game_instance.get_hash().to_string(),
                        packet.get_username(),
                    );
                    
                    out.send(ws::Message::text(serde_json::to_string(&pack).unwrap()));
                    game_vec.push(Arc::new(Mutex::new(game_instance)));
                }
                if packet.get_task() == Operation::SubmitCard{
                    match &*game_lock {
                        Some(game_temp) => {
                            let mut game = game_temp.lock().unwrap();
                            game.submit_card(packet.get_data());
                        },
                        None => continue,
                    }
                }
                let game_vec = games.lock().unwrap();
                if packet.get_task() == Operation::CreateUser {
                    match Game::search_mutex(&game_vec, packet.get_gameid().parse::<u16>().unwrap())
                    {
                        Some(mut game) => {
                            let mut temp_game = game.lock().unwrap();
                            let user = User::new(packet.get_data(), out.clone(), temp_game.count_users().try_into().unwrap());
                            match temp_game.add_user(user){
                                Ok(_) => (),
                                Err(e) => {
                                    let err = Packet::new(
                                        temp_game.get_hash(),
                                        PacketType::Error,
                                        Operation::CreateUserError,
                                        e,
                                        packet.get_username(),
                                    );
                                   out.send(ws::Message::text(
                                        serde_json::to_string(&err).unwrap(),
                                    )); 
                                },
                            }
                            let drawblack: Packet = Packet::new(
                                temp_game.get_hash(),
                                PacketType::Game,
                                Operation::DrawBlack,
                                temp_game.current_black().get_text(),
                                packet.get_username(),
                            );
                            let pack: Packet = Packet::new(
                                temp_game.get_hash(),
                                PacketType::Game,
                                Operation::CreateUser,
                                temp_game.get_hash().to_string(),
                                packet.get_username(),
                            );
                            out.send(ws::Message::text(
                                serde_json::to_string(&drawblack).unwrap(),
                            ));
                            out.send(ws::Message::text(
                                serde_json::to_string(&pack).unwrap(),
                            ));
                            game_lock = Arc::new(Some(game.to_owned()));
                        }
                        None => continue,
                    }
                }
                if packet.get_task() == Operation::DrawWhite {
                    match &*game_lock {
                        Some(game_temp) => {
                            let mut game = game_temp.lock().unwrap();
                            let hash = game.get_hash();
                            let white_card = game.draw_white();
                            //have draw_white() auto add the card to the user struct
                            let found_user = game.search_users_mut(packet.get_username());
                            if let Some(user) = found_user {
                                user.add_white_card(white_card.clone());
                                let data = white_card.get_text();
                                let packet = Packet::new(
                                    hash,
                                    PacketType::Game,
                                    Operation::DrawWhite,
                                    data,
                                    packet.get_username(),
                                );
                                user.send_packet(packet).unwrap();
                            }
                        }
                        None => continue,
                    }
                }
                
            }
        });

        move |msg: ws::Message| {
            
            let content = msg.clone();
            let packet: Packet = serde_json::from_str(msg.into_text().unwrap().as_str()).unwrap();
            
            match tx.send(packet) {
                Ok(_) => (),
                Err(e) => println!("error: {}", e),
            }
            Ok(())
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    };
}
