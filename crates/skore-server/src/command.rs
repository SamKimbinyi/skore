use skore_protocol::RespValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
    Ping,
    Keys,
    Clear,
    Len,
    Unknown(String), // Maybe an error?
}
impl Command {}
