use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use crate::{message::Message, state::State, Node};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastPayload {
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<u32>,
    },
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk,
}

impl Node<BroadcastPayload> for Message<BroadcastPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
        match self.body().payload() {
            BroadcastPayload::Broadcast { message } => {
                state.add_message(*message);
                let reply = Message::reply(state, self, BroadcastPayload::BroadcastOk);
                reply.write(writer)?;
            }
            BroadcastPayload::BroadcastOk => {}
            BroadcastPayload::Read => {
                let reply = Message::reply(
                    state,
                    self,
                    BroadcastPayload::ReadOk {
                        messages: state.messages().to_vec(),
                    },
                );
                reply.write(writer)?;
            }
            BroadcastPayload::ReadOk { .. } => {}
            BroadcastPayload::Topology { topology } => {
                state.update_topology(topology);
                let reply = Message::reply(state, self, BroadcastPayload::TopologyOk);
                reply.write(writer)?;
            }
            BroadcastPayload::TopologyOk => {}
        }
        Ok(())
    }
}
