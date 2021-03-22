use rand::Rng;

use std::convert::TryInto;
use std::option::Option;
use std::sync::{Arc, Mutex, MutexGuard};

use std::collections::hash_map::DefaultHasher;

use crate::CahError;
use tokio::sync::mpsc::Receiver;

use crate::network::{Operation, Packet, PacketType};
use std::hash::{Hash, Hasher};

lazy_static! {
    static ref WHITE_CARDS: Vec<Card> = {
        let text = include_str!("white-cards.json");
        serde_json::from_str(text).unwrap()
    };
}

lazy_static! {
    static ref BLACK_CARDS: Vec<Card> = {
        let text = include_str!("black-cards.json");
        serde_json::from_str(text).unwrap()
    };
}

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
    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn get_text(&self) -> &str {
        &self.text
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
/// represents a client interacting with the game
pub struct User {
    sender: ws::Sender,
    white_cards: Vec<u16>,
    black_cards: Vec<u16>,
    username: String,
    hash: u16,
}
impl User {
    pub fn score(&self) -> usize {
        self.black_cards.len()
    }
    pub fn new(username: String, sender: ws::Sender) -> Self {
        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);

        User {
            sender,
            white_cards: Vec::with_capacity(7),
            black_cards: Vec::new(),
            hash: hasher.finish() as u16,
            username,
        }
    }
    pub fn send_packet(&self, packet: &Packet) -> Result<(), CahError> {
        Ok(self
            .sender
            .send(ws::Message::Text(serde_json::to_string(packet).unwrap()))?)
    }
    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn get_id(&self) -> u16 {
        self.hash
    }
    pub fn add_white_card(&mut self, card: u16) {
        self.white_cards.push(card);
    }
    pub fn add_black_card(&mut self, card: u16) {
        self.black_cards.push(card);
    }
    pub fn get_white_cards(&self) -> &[u16] {
        &self.white_cards
    }
}
pub struct Game {
    users: Vec<User>,
    //draw_white: Vec<Card>,
    //draw_black: Vec<Card>,
    discard: Vec<u16>,
    // black cards that have already been used
    discard_black: Vec<u16>,
    group_cards: Vec<JudgeCard>,
    judge: u16,
    hash: u16,
    current_black: Option<u16>,
}
impl Game {
    pub fn load_cards(&mut self, user_index: usize) -> Result<(), CahError> {
        for _ in 0..7 {
            let card_index = self.draw_white();
            // I should make fix the case when this fails, but other still connected
            self.users[user_index].send_packet(&Packet::new(
                self.hash,
                PacketType::Game,
                Operation::DrawWhite,
                WHITE_CARDS[card_index as usize].get_text().to_string(),
                self.users[user_index].get_username().to_string(),
            ))?;
        }
        let black_card = self.current_black();
        self.users[user_index].send_packet(&Packet::new(
            self.hash,
            PacketType::Game,
            Operation::DrawBlack,
            BLACK_CARDS[black_card as usize].get_text().to_string(),
            self.users[user_index].get_username().to_string(),
        ))?;
        Ok(())
    }
    pub async fn handle(
        hash: u16,
        mut receiver: Receiver<(Packet, ws::Sender)>,
        create_packet: Packet,
        sender: ws::Sender,
    ) -> Result<(), crate::CahError> {
        let mut game = Game::new(hash);
        let mut user_index = 0;
        game.users
            .push(User::new(create_packet.get_username().to_string(), sender));
        game.load_cards(user_index).unwrap();
        user_index += 1;
        loop {
            let (packet, out_sender) = receiver.recv().await.unwrap();
            // main game loop here
            match packet.get_task() {
                Operation::SendMessage => {
                    for user in game.users.iter_mut() {
                        if user.get_username() != packet.get_username() {
                            user.send_packet(&Packet::new(
                                game.hash,
                                PacketType::Message,
                                Operation::SendMessage,
                                packet.get_data(),
                                packet.get_username().to_string(),
                            ))
                            .unwrap();
                        }
                    }
                }
                Operation::CreateUser => {
                    game.users.push(User::new(
                        create_packet.get_username().to_string(),
                        out_sender,
                    ));
                    game.load_cards(user_index).unwrap();
                    user_index += 1;
                }
                _ => {
                    unimplemented!();
                }
            }
        }
    }
    pub fn new(hash: u16) -> Self {
        Game {
            users: Vec::new(),
            discard: Vec::new(),
            discard_black: Vec::new(),
            group_cards: Vec::new(),
            judge: 0,
            hash,
            current_black: None,
        }
    }
    /*pub fn get_discard(&self) -> Vec<Card> {
        self.discard.clone()
    }
    pub fn get_all_white(&self) -> Vec<Card> {
        self.draw_white.clone()
    }*/
    pub fn draw_white(&mut self) -> u16 {
        if self.discard.len() >= (WHITE_CARDS.len() * 5) * 4 {
            self.discard.clear();
        }
        let mut rng = rand::thread_rng();
        let mut hash: u16 = rng.gen::<u16>() % WHITE_CARDS.len() as u16;
        while self.discard_black.contains(&hash) {
            hash = rng.gen::<u16>() % WHITE_CARDS.len() as u16;
        }
        hash
    }
    pub fn draw_black(&mut self) -> u16 {
        if self.discard_black.len() >= (BLACK_CARDS.len() / 5) * 4 {
            self.discard_black.clear();
        }
        let mut rng = rand::thread_rng();
        let mut hash: u16 = rng.gen::<u16>() % BLACK_CARDS.len() as u16;
        while self.discard_black.contains(&hash) {
            hash = rng.gen::<u16>() % BLACK_CARDS.len() as u16;
        }
        hash
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
    pub fn current_black(&mut self) -> u16 {
        match &self.current_black {
            Some(card) => *card,
            None => {
                let card_index = self.draw_black();
                self.current_black = Some(card_index);
                card_index
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
        BLACK_CARDS.len() - self.discard_black.len()
    }
    pub fn count_white(&self) -> usize {
        WHITE_CARDS.len() - self.discard.len()
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
    pub async fn submit_card(&mut self, username: String, card_text: String) {
        self.group_cards.push(JudgeCard::new(username, card_text));
        println!("group cards len: {}", self.group_cards.len());
        println!("users len {}", self.users.len());
        if self.users.len() - 1 == self.group_cards.len() {
            for user in self.users.iter_mut() {
                if user.get_id() == self.judge {
                    for card in self.group_cards.iter() {
                        // create and send the packet to the judge
                        let packet: Packet = Packet::new(
                            self.hash,
                            PacketType::Game,
                            Operation::SubmitCard,
                            card.get_text().to_string(),
                            card.get_username().to_string(),
                        );
                        user.send_packet(&packet).unwrap();
                    }
                }
            }
            self.group_cards.clear();
        }
    }
    pub async fn change_judge(&mut self) {
        match self.search_users_by_id(self.judge) {
            Some(user) => {
                for card in user.get_white_cards().iter() {
                    let packet: Packet = Packet::new(
                        user.get_id(),
                        PacketType::Game,
                        Operation::DrawWhite,
                        WHITE_CARDS[*card as usize].get_text(),
                        user.get_username().to_string(),
                    );
                    user.send_packet(&packet).unwrap();
                }
            }
            None => panic!("user stopped existing for some reason"),
        }
        let user_count: u16 = self.users.len().try_into().unwrap();
        if self.judge == user_count {
            self.judge = 0;
        } else {
            self.judge += 1;
        }
    }
}
