/// Simple WebSocket server with error handling. It is not necessary to setup logging, but doing
/// so will allow you to see more details about the connection by using the RUST_LOG env variable.
extern crate ws;

use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::convert::TryInto;

use ws::listen;

use cah_backend::game::{Game, User};
use cah_backend::network::{Packet, Operation};

fn main() {
    // Setup logging
    let all_games: Arc<Mutex<Vec<Arc<Mutex<Game>>>>> = Arc::new(Mutex::new(Vec::new()));

    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:3012", |out| {
        let (tx, rx) = channel();
        let games = all_games.clone();
        thread::spawn(move || {
            let mut packet: Packet = rx.recv().unwrap();
            loop{
                if packet.get_task() == Operation::StartGame {
                    let mut game_vec = games.lock().unwrap();
                    let game_instance = Game::new(game_vec.len().try_into().unwrap());
                    game_vec.push(Arc::new(Mutex::new(game_instance)));
                    break;
                }
                if packet.get_task() == Operation::CreateUser{
                    let mut game_vec = games.lock().unwrap();
                    let mut found_game = false;
                    for game in game_vec.iter(){
                        let mut game = game.lock().unwrap();
                        if game.get_hash() == packet.get_gameid().parse::<u16>().unwrap(){    
                            game.add_user(User::new(packet.get_data()));
                            found_game = true;
                        }
                    }
                    if !found_game{
                        //game wasn't found let the user no that
                    }else{
                        break;
                    }
                }
            }
            //main game loop
            println!("about to enter second loop ");
            while packet.get_task() != Operation::EndGame {
                
                out.send(ws::Message::text(serde_json::to_string(&packet).unwrap()))
                    .unwrap();
                packet = rx.recv().unwrap();
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
