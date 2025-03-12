use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

use crate::{message::Message, state::State, Node};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl Node<EchoPayload> for Message<EchoPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
        match self.body().payload() {
            EchoPayload::Echo { echo } => {
                let reply = Message::reply(state, self, EchoPayload::EchoOk { echo: echo.clone() });
                reply.write(writer)?;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }
}
