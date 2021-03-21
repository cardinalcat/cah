use rand::Rng;
use std::fs::File;
use std::io::Read;

use std::convert::TryInto;
use std::option::Option;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::mpsc::Receiver;

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
pub struct JudgeCard {
    username: String,
    text: String,
}
impl JudgeCard {
    pub fn new(username: String, text: String) -> Self {
        JudgeCard { username, text }
    }
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
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
    black_cards: Vec<String>,
    username: String,
    hash: u16,
}
impl User {
    pub fn score(&self) -> usize {
        self.black_cards.len()
    }
    pub fn new(username: String, sender: ws::Sender, id: u16) -> Self {
        User {
            sender,
            white_cards: Vec::with_capacity(7),
            black_cards: Vec::new(),
            hash: id,
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
    pub fn get_id(&self) -> u16 {
        self.hash
    }
    pub fn add_white_card(&mut self, card: Card) {
        self.white_cards.push(card);
    }
    pub fn add_black_card(&mut self, card_text: String) {
        self.black_cards.push(card_text);
    }
    pub fn get_white_cards(&self) -> Vec<Card> {
        self.white_cards.clone()
    }
}
pub struct Game {
    users: Vec<User>,
    draw_white: Vec<Card>,
    draw_black: Vec<Card>,
    discard: Vec<Card>,
    group_cards: Vec<JudgeCard>,
    judge: u16,
    hash: u16,
    current_black: Option<Card>,
}
impl Game {
    pub async fn handle(hash: u16, mut receiver: Receiver<Packet>) -> Result<(), crate::CahError>{
        let mut game = Game::new(hash);
        loop {
            let packet = receiver.recv().await;
            /// main game loop here
        }
    }
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
        for card in bcontents.split('|') {
            let card = card.trim();
            if !card.is_empty() {
                draw_black.push(serde_json::from_str(card).unwrap());
            }
        }
        for card in wcontents.split('|') {
            let card = card.trim();
            if !card.is_empty() {
                draw_white.push(serde_json::from_str(card).unwrap());
            }
        }
        Game {
            users: Vec::new(),
            draw_black,
            draw_white,
            discard: Vec::new(),
            group_cards: Vec::new(),
            judge: 0,
            hash,
            current_black: None,
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
            self.draw_white.append(&mut self.discard);
            self.discard.clear();
        }

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
        game_vec: &[Arc<Mutex<Self>>],
        gameid: u16,
    ) -> Option<MutexGuard<'_, Game>> {
        for temp_game in game_vec.iter() {
            let game = temp_game.lock().unwrap();
            if game.get_hash() == gameid {
                return Some(game);
            }
        }
        None
    }
    pub fn current_black(&mut self) -> Card {
        match &self.current_black {
            Some(card) => card.clone(),
            None => {
                self.current_black = Some(self.draw_black());
                self.current_black.as_ref().unwrap().clone()
            }
        }
    }
    pub fn search_mutex(game_vec: &[Arc<Mutex<Self>>], gameid: u16) -> Option<Arc<Mutex<Game>>> {
        for temp_game in game_vec.iter() {
            let game = temp_game.lock().unwrap();
            if game.get_hash() == gameid {
                return Some(temp_game.clone());
            }
        }
        None
    }
    pub fn count_black(&self) -> usize {
        self.draw_black.len()
    }
    pub fn count_white(&self) -> usize {
        self.draw_white.len()
    }
    pub fn add_user(&mut self, user: User) -> std::result::Result<(), String> {
        for existing_user in self.users.iter() {
            if existing_user.get_username() == user.get_username() {
                return Err("username in use".to_string());
            }
        }
        self.users.push(user);
        Ok(())
    }
    pub fn get_hash(&self) -> u16 {
        self.hash
    }
    pub fn search_users(&self, username: String) -> Option<&User> {
        for user in self.users.iter() {
            if user.get_username() == username {
                //let usr = *user.clone();
                return Some(user);
            }
        }
        None
    }
    pub fn search_users_mut(&mut self, username: String) -> Option<&mut User> {
        for user in self.users.iter_mut() {
            if user.get_username() == username {
                //let usr = *user.clone();
                return Some(user);
            }
        }
        None
    }
    pub fn search_users_by_id(&self, id: u16) -> Option<&User> {
        for user in self.users.iter() {
            if user.get_id() == id {
                return Some(user);
            }
        }
        None
    }
    pub fn count_users(&self) -> usize {
        self.users.len()
    }
    pub fn submit_card(&mut self, username: String, card_text: String) {
        self.group_cards.push(JudgeCard::new(username, card_text));
        println!("group cards len: {}", self.group_cards.len());
        println!("users len {}", self.users.len());
        if self.users.len() - 1 == self.group_cards.len() {
            for user in self.users.iter() {
                if user.get_id() == self.judge {
                    for card in self.group_cards.iter() {
                        // create and send the packet to the judge
                        let packet: Packet = Packet::new(
                            self.hash,
                            PacketType::Game,
                            Operation::SubmitCard,
                            card.get_text(),
                            card.get_username(),
                        );
                        user.send_packet(packet).unwrap();
                    }
                }
            }
            self.group_cards.clear();
        }
    }
    pub fn change_judge(&mut self) {
        match self.search_users_by_id(self.judge) {
            Some(user) => {
                for card in user.get_white_cards().iter() {
                    let packet: Packet = Packet::new(
                        user.get_id(),
                        PacketType::Game,
                        Operation::DrawWhite,
                        card.get_text(),
                        user.get_username(),
                    );
                    user.send_packet(packet).unwrap();
                }
            }
            None => panic!("user stopped existing for some reason"),
        }
        let user_count: u16 = self.users.len().try_into().unwrap();
        if self.judge == user_count {
            self.judge = 0;
        } else {
            self.judge = self.judge + 1;
        }
    }
}
