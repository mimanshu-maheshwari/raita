use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

use crate::{message::Message, state::State, Node};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniqueIdPayload {
    Generate,
    GenerateOk { id: ulid::Ulid },
}

impl Node<UniqueIdPayload> for Message<UniqueIdPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
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
