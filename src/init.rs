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
        match self.body().payload() {
            InitPayload::Init { node_id, node_ids } => {
                state.set_node_id(node_id);
                state.set_declared_nodes(node_ids);
                let reply = Message::reply(state, self, InitPayload::InitOk);
                reply.write(writer)?;
            }
            InitPayload::InitOk => bail!("Unexpected message Init Ok"),
        }
        Ok(())
    }
}
