#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum PacketType {
    Admin,
    Message,
    Game,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operation {
    DrawWhite,
    DrawBlack,
    SendMessage,
    CreateUser,
    SubmitCard,
    SelectWinner,
    StartGame,
    EndGame,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Packet {
    gameid: String,
    username: String,
    kind: PacketType,
    task: Operation,
    data: String,
}
impl Packet {
    pub fn new(id: u16, kind: PacketType, task: Operation, data: String, username: String) -> Self {
        Packet {
            gameid: id.to_string(),
            username,
            kind,
            task,
            data,
        }
    }
    pub fn get_task(&self) -> Operation {
        self.task
    }
    pub fn get_data(&self) -> String {
        self.data.clone()
    }
    pub fn get_gameid(&self) -> String {
        println!("gameid in packet.get_gameid() {:?}", self.gameid);
        self.gameid.clone()
    }
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
}
