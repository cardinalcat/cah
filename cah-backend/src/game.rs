use rand::Rng;
use std::fs::File;
use std::io::Read;
use ws::WebSocket;

use std::option::Option;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::network::{Operation, Packet, PacketType};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Color {
    Black,
    White,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Kind {
    One,
    Two,
    Three,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Card {
    id: u16,
    kind: Kind,
    color: Color,
    text: String,
}
impl Card {
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
}
pub struct User {
    sender: ws::Sender,
    white_cards: Vec<Card>,
    black_cards: Vec<Card>,
    username: String,
    hash: u64,
}
impl User {
    pub fn score(&self) -> usize {
        self.black_cards.len()
    }
    pub fn new(username: String, sender: ws::Sender) -> Self {
        User {
            sender,
            white_cards: Vec::with_capacity(7),
            black_cards: Vec::new(),
            hash: 12,
            username,
        }
    }
    pub fn send_packet(&self, packet: Packet) -> std::result::Result<(), ws::Error> {
        self.sender
            .send(ws::Message::text(serde_json::to_string(&packet).unwrap()))
    }
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
}
pub struct Game {
    users: Vec<User>,
    draw_white: Vec<Card>,
    draw_black: Vec<Card>,
    discard: Vec<Card>,
    hash: u16,
}
impl Game {
    pub fn new(hash: u16) -> Self {
        let mut wcontents = String::new();
        let mut bcontents = String::new();
        File::open("black-cards.json")
            .expect("no black cards")
            .read_to_string(&mut bcontents)
            .unwrap();
        File::open("white-cards.json")
            .expect("no white cards")
            .read_to_string(&mut wcontents)
            .unwrap();
        let mut draw_white: Vec<Card> = Vec::new();
        let mut draw_black: Vec<Card> = Vec::new();
        for card in bcontents.split("|") {
            let card = card.trim();
            if !card.is_empty() {
                draw_black.push(serde_json::from_str(card).unwrap());
            }
        }
        for card in wcontents.split("|") {
            let card = card.trim();
            if !card.is_empty() {
                draw_white.push(serde_json::from_str(card).unwrap());
            }
        }
        println!("draw_black: {}", draw_black.len());
        Game {
            users: Vec::new(),
            draw_black,
            draw_white,
            discard: Vec::new(),
            hash,
        }
    }
    pub fn get_discard(&self) -> Vec<Card> {
        self.discard.clone()
    }
    pub fn get_all_white(&self) -> Vec<Card> {
        self.draw_white.clone()
    }
    pub fn draw_white(&mut self) -> Card {
        if self.draw_white.is_empty() {
            println!("empty");
            self.draw_white.append(&mut self.discard);
            self.discard.clear();
        }
        //println!("draw_white: {}", self.draw_white.len());
        let mut rng = rand::thread_rng();
        let hash: usize = rng.gen::<usize>() % self.draw_white.len();
        self.draw_white.remove(hash)
    }
    pub fn draw_black(&mut self) -> Card {
        let mut rng = rand::thread_rng();
        let hash: usize = rng.gen::<usize>() % self.draw_black.len();
        self.draw_black.remove(hash)
    }
    //game.add_user(User::new(packet.get_data(), out.clone()));
    pub fn search_guard(
        game_vec: &Vec<Arc<Mutex<Self>>>,
        gameid: u16,
    ) -> Option<MutexGuard<'_, Game>> {
        for temp_game in game_vec.iter() {
            let mut game = temp_game.lock().unwrap();
            if game.get_hash() == gameid {
                return Some(game);
            }
        }
        None
    }
    pub fn search_mutex(game_vec: &Vec<Arc<Mutex<Self>>>, gameid: u16) -> Option<Arc<Mutex<Game>>> {
        println!("gameid in search mutex: {}", gameid);
        for temp_game in game_vec.iter() {
            let mut game = temp_game.lock().unwrap();
            if game.get_hash() == gameid {
                return Some(temp_game.clone());
            }
        }
        None
    }
    /*pub fn handle_event(&mut self, packet: &Packet) {
        println!("packet: {:?}", packet);
        match packet.get_task() {
            StartGame => panic!("game start requested multiple times on the same thread"),
            CreateUser => panic!("user should already have been created"),
            DrawWhite => {
                let white_card = self.draw_white();
                println!("card draw: {:?}", white_card);
                let found_user = self.search_users(packet.get_username());
                if let Some(user) = found_user {
                    let data = white_card.get_text();
                    let packet = Packet::new(
                        self.hash,
                        PacketType::Game,
                        Operation::DrawWhite,
                        data,
                        packet.get_username(),
                    );
                    user.send_packet(packet).unwrap();
                }
            }
        }
    }*/
    pub fn count_black(&self) -> usize {
        self.draw_black.len()
    }
    pub fn count_white(&self) -> usize {
        self.draw_white.len()
    }
    pub fn add_user(&mut self, user: User) {
        println!("game id in adding user: {}", self.hash);
        self.users.push(user);
    }
    pub fn get_hash(&self) -> u16 {
        self.hash
    }
    pub fn search_users(&self, username: String) -> Option<&User> {
        for user in self.users.iter() {
            if user.get_username() == username {
                return Some(user);
            }
        }
        None
    }
}
