use cah_backend::GameEngine;

#[tokio::main]
async fn main() {
    let engine = GameEngine::new();
    engine.run().await.unwrap();
}
