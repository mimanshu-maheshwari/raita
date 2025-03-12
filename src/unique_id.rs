use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

use crate::{
    message::{Body, Message},
    state::State,
    Node,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniqueIdPayload {
    Generate,
    GenerateOk {
        in_reply_to: Option<usize>,
        id: ulid::Ulid,
    },
}

impl Node<UniqueIdPayload> for Message<UniqueIdPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
        match self.body().payload() {
            UniqueIdPayload::Generate => {
                let reply = Message::reply(
                    self,
                    Body::new(
                        Some(state.get_and_increment()),
                        UniqueIdPayload::GenerateOk {
                            in_reply_to: self.body().message_id(),
                            id: ulid::Ulid::new(),
                        },
                    ),
                );
                reply.write(writer)?;
            }
            UniqueIdPayload::GenerateOk { .. } => bail!("Recieved generated ok"),
        }
        Ok(())
    }
}
