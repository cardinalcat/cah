extern crate rand;
extern crate ws;
use cah_backend::game::Game;
use rand::Rng;
use ws::listen;

fn main() {
    let mut rng = rand::thread_rng();
    let hash: u16 = rng.gen();
    let mut game = Game::new(hash);
    let mut index: usize = 0;
    let black_card_count = game.count_black();
    while (index < black_card_count) {
        println!(
            "black: {}\n\n, white: {}\n",
            game.draw_black().get_text(),
            game.draw_white().get_text(),
        );
        println!("{}", index);
        index = index + 1;
    }
    /*if let Err(error) = listen("127.0.0.1:3012", |out| {
        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);
            let content = msg.clone();
            if msg.into_text().unwrap() == "start".to_string() {

                out.send(format!("{}", hash));
            } else {
                // Use the out channel to send messages back
                out.send(format!("unknown command {}", content));
            }
            Ok(())
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }*/
}
