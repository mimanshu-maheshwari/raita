use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::StdoutLock};

use crate::{message::Message, state::State, Node};
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
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
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
