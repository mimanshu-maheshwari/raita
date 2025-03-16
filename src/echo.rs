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

impl Node<EchoPayload> for Event<EchoPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => bail!("Unexpected message EOF"),
            Event::GeneratedMessage => {}
            Event::ReceivedMessage(message) => match &message.body.payload {
                EchoPayload::Echo { echo } => {
                    Message::reply(
                        state,
                        message,
                        EchoPayload::EchoOk {
                            echo: echo.to_owned(),
                        },
                    )
                    .write(writer)?;
                }
                EchoPayload::EchoOk { .. } => {}
            },
        }
        Ok(())
    }
}
