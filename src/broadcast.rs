use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use crate::{
    message::{Body, Message},
    state::State,
    Node,
};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastPayload {
    Broadcast {
        message: u32,
    },
    BroadcastOk {
        in_reply_to: Option<usize>,
    },
    Read,
    ReadOk {
        messages: Vec<u32>,
        in_reply_to: Option<usize>,
    },
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk {
        in_reply_to: Option<usize>,
    },
}

impl Node<BroadcastPayload> for Message<BroadcastPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
        match self.body().payload() {
            BroadcastPayload::Broadcast { message } => {
                state.add_message(*message);
                let reply = Message::reply(
                    self,
                    Body::new(
                        Some(state.get_and_increment()),
                        BroadcastPayload::BroadcastOk {
                            in_reply_to: self.body().message_id(),
                        },
                    ),
                );
                reply.write(writer)?;
            }
            BroadcastPayload::BroadcastOk { .. } => {}
            BroadcastPayload::Read => {
                let reply = Message::reply(
                    self,
                    Body::new(
                        Some(state.get_and_increment()),
                        BroadcastPayload::ReadOk {
                            messages: state.messages().to_vec(),
                            in_reply_to: self.body().message_id(),
                        },
                    ),
                );
                reply.write(writer)?;
            }
            BroadcastPayload::ReadOk { .. } => {}
            BroadcastPayload::Topology { topology } => {
                state.update_topology(topology);
                let reply = Message::reply(
                    self,
                    Body::new(
                        Some(state.get_and_increment()),
                        BroadcastPayload::TopologyOk {
                            in_reply_to: self.body().message_id(),
                        },
                    ),
                );
                reply.write(writer)?;
            }
            BroadcastPayload::TopologyOk { .. } => {}
        }
        Ok(())
    }
}
