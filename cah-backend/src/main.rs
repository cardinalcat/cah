extern crate ws;
extern crate rand;
use rand::Rng;

use ws::listen;

fn main() {
    // Listen on an address and call the closure for each connection
    if let Err(error) = listen("127.0.0.1:3012", |out| {
        // The handler needs to take ownership of out, so we use move
        move |msg: ws::Message| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);
            let content = msg.clone();
            if msg.into_text().unwrap() == "start".to_string(){
                let mut rng = rand::thread_rng();
                let hash: u16 = rng.gen();
                out.send(format!("{}", hash));
            }else{
            // Use the out channel to send messages back
                out.send(format!("unknown command {}", content));
            }
            Ok(())
        }
    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }
}