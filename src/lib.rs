pub mod echo;
pub mod init;
pub mod message;

use init::InitPayload;
use message::Message;
use serde::de::DeserializeOwned;
use std::io::{stdin, stdout, BufRead, BufReader, StdoutLock};

pub trait Node<Payload> {
    fn step(&self, writer: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<Payload>() -> anyhow::Result<()>
where
    Payload: Sized + DeserializeOwned,
    Message<Payload>: Node<Payload>,
{
    let stdin = stdin().lock();
    let mut stdout = stdout().lock();
    let mut input_buffer = String::new();
    let mut reader = BufReader::new(stdin);
    reader.read_line(&mut input_buffer)?;
    let init_message: Message<InitPayload> = serde_json::from_str(&input_buffer)?;
    init_message.step(&mut stdout)?;
    input_buffer.clear();
    while let Ok(bytes) = reader.read_line(&mut input_buffer) {
        if bytes == 0 {
            break;
        }
        let message: Message<Payload> = serde_json::from_str(&input_buffer)?;
        message.step(&mut stdout)?;
        input_buffer.clear();
    }
    Ok(())
}
