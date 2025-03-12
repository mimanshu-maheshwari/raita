use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

use crate::{
    message::{Body, Message},
    Node,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoPayload {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
        in_reply_to: Option<usize>,
    },
}

impl Node<EchoPayload> for Message<EchoPayload> {
    fn step(
        &self,
        // input: &mut Message<EchoPayload>,
        writer: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match self.body().payload() {
            EchoPayload::Echo { echo } => {
                let reply = Message::reply(
                    &self,
                    Body::new(
                        self.body().message_id(),
                        EchoPayload::EchoOk {
                            echo: echo.clone(),
                            in_reply_to: self.body().message_id(),
                        },
                    ),
                );
                serde_json::to_writer(&mut *writer, &reply)?;
                writer.write_all(b"\r")?;
            }
            EchoPayload::EchoOk { .. } => {}
        }
        Ok(())
    }
}
