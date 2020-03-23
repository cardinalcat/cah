/// Simple WebSocket server with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;

use std::convert::TryInto;
use std::option::Option;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::boxed::Box;

use ws::listen;

use cah_backend::game::{Game, User};
use cah_backend::network::{Operation, Packet};

fn main() {
    // Setup logging
    let all_games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(Mutex::new(Vec::new()));

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:3012", |out| {
        let (tx, rx) = channel();
        let games = all_games.clone();
        thread::spawn(move || {
            let mut packet: Packet = rx.recv().unwrap();
            let mut gameid: u16 = packet.get_gameid().parse::<u16>().unwrap();
            loop {
                if packet.get_task() == Operation::StartGame {
                    let mut game_vec = games.lock().unwrap();
                    let game_instance = Game::new(game_vec.len().try_into().unwrap());
                    game_vec.push(Arc::new(Mutex::new(game_instance)));
                    break;
                }
                let game_vec = games.lock().unwrap();
                if packet.get_task() == Operation::CreateUser {
                    match Game::search_guard(&game_vec, packet.get_gameid().parse::<u16>().unwrap())
                    {
                        Some(mut game) => {
                            game.add_user(User::new(packet.get_data(), out.clone()));
                            break;
                        }
                        None => continue,
                    }
                }
                packet = rx.recv().unwrap();
                gameid = packet.get_gameid().parse::<u16>().unwrap();
            }
            let game_vec = games.lock().unwrap();
            //let mut game: Option<Arc<Mutex<Game>>> = None;
            let mut game_lock: Option<Arc<Mutex<Game>>> =
                match Game::search_mutex(&game_vec, packet.get_gameid().parse::<u16>().unwrap()) {
                    Some(game) => Some(game),
                    None => None,
                };
            std::mem::drop(game_vec);
            //main game loop
            println!("about to enter second loop ");
            while packet.get_task() != Operation::EndGame {
                let game_item = game_lock.as_ref();
                let mut game = game_item.unwrap().lock().unwrap();
                if packet.get_task() != Operation::StartGame && packet.get_task() != Operation::CreateUser{
                    game.handle_event(&packet);
                }
                std::mem::drop(game);
                packet = match rx.recv(){
                    Ok(pack) => packet,
                    Err(e) => break,
                }
            }
        });

        move |msg: ws::Message| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);
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
