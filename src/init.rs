use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Write};

use crate::{
    message::{Event, Message},
    state::State,
    Node,
};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: HashSet<String>,
    },
    InitOk,
}

impl Node<InitPayload> for Message<InitPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match &self.body.payload {
            InitPayload::Init { node_id, node_ids } => {
                state.node_id = node_id.clone();
                state.declared_nodes = node_ids.clone();
                Message::reply(state, self, InitPayload::InitOk).write(writer)?;
            }
            InitPayload::InitOk => bail!("Unexpected message Init Ok"),
        }
        Ok(())
    }
}

impl Node<InitPayload> for Event<InitPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => bail!("Unexpected message EOF"),
            Event::GeneratedMessage(message) => bail!("Unexpected message {message:?}"),
            Event::ReceivedMessage(message) => {
                match &message.body.payload {
                    InitPayload::Init { node_id, node_ids } => {
                        state.node_id = node_id.clone();
                        state.declared_nodes = node_ids.clone();
                        Message::reply(state, message, InitPayload::InitOk).write(writer)?;
                    }
                    InitPayload::InitOk => bail!("Unexpected message Init Ok"),
                }
                Ok(())
            }
        }
    }
}
