#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum PacketType{
    Admin,
    Message,
    Game,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Operation{
    DrawWhite,
    DrawBlack,
    SendMessage,
    CreateUser,
    SubmitCard,
    SelectWinner,
    StartGame,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Packet{
    gameid: u16,
    kind: PacketType,
    task: Operation,
    data: String,
}
