#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum PacketType {
    Admin,
    Message,
    Game,
    Error,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operation {
    DrawWhite,
    DrawBlack,
    SendMessage,
    CreateUser,
    DropUser,
    SubmitCard,
    SelectWinner,
    StartGame,
    EndGame,
    CreateUserError,
    ChangeJudge,
    None,
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

    pub fn report_error(error: String) -> ws::Message {
        ws::Message::Text(
            serde_json::to_string(&Packet {
                gameid: String::from("-1"),
                username: String::from("server"),
                kind: PacketType::Error,
                task: Operation::None,
                data: error,
            })
            .unwrap(),
        )
    }

    pub fn get_task(&self) -> Operation {
        self.task
    }
    pub fn get_data(&self) -> String {
        self.data.clone()
    }
    pub fn get_gameid(&self) -> &str {
        &self.gameid
    }
    pub fn get_username(&self) -> &str {
        &self.username
    }
}
