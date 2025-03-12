use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

use crate::{
    message::{Body, Message},
    Node,
};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: Option<usize>,
    },
}

impl Node<InitPayload> for Message<InitPayload> {
    fn step(
        &self,
        // input: &mut Message<InitPayload>,
        writer: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match self.body().payload() {
            InitPayload::Init { node_id, node_ids } => {
                let reply = Message::reply(
                    &self,
                    Body::new(
                        self.body().message_id(),
                        InitPayload::InitOk {
                            in_reply_to: self.body().message_id(),
                        },
                    ),
                );
                serde_json::to_writer(&mut *writer, &reply)?;
                writer.write_all(b"\r")?;
            }
            InitPayload::InitOk { .. } => bail!("Unexpected message Init Ok"),
        }
        Ok(())
    }
}
