use crate::{message::Message, state::State, Node};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl Node<EchoPayload> for Message<EchoPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match &self.body.payload {
            EchoPayload::Echo { echo } => {
                Message::reply(
                    state,
                    self,
                    EchoPayload::EchoOk {
                        echo: echo.to_owned(),
                    },
                )
                .write(writer)?;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }
}
