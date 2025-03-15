use crate::{
    message::{Event, Message},
    state::State,
    Node,
};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniqueIdPayload {
    Generate,
    GenerateOk { id: ulid::Ulid },
}
impl Node<UniqueIdPayload> for Message<UniqueIdPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self.body.payload {
            UniqueIdPayload::Generate => {
                Message::reply(
                    state,
                    self,
                    UniqueIdPayload::GenerateOk {
                        id: ulid::Ulid::new(),
                    },
                )
                .write(writer)?;
            }
            UniqueIdPayload::GenerateOk { .. } => bail!("Recieved generated ok"),
        }
        Ok(())
    }
}
impl Node<UniqueIdPayload> for Event<UniqueIdPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => bail!("Unexpected message EOF"),
            Event::GeneratedMessage(message) => bail!("Unexpected message {message:?}"),
            Event::ReceivedMessage(message) => {
                match &message.body.payload {
                    UniqueIdPayload::Generate => {
                        Message::reply(
                            state,
                            message,
                            UniqueIdPayload::GenerateOk {
                                id: ulid::Ulid::new(),
                            },
                        )
                        .write(writer)?;
                    }
                    UniqueIdPayload::GenerateOk { .. } => bail!("Recieved generated ok"),
                }
                Ok(())
            }
        }
    }
}
