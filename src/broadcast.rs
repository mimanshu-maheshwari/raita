use crate::{
    message::{Event, Message},
    state::State,
    Node,
};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

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
    Gossip {
        messages: HashSet<u32>,
    },
    GossipOk {
        messages: HashSet<u32>,
    },
}

#[derive(Debug, Clone)]
pub enum GeneratedPayload {
    Gossip,
}

impl Node<BroadcastPayload> for Message<BroadcastPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match &self.body.payload {
            BroadcastPayload::Broadcast { message } => {
                state.add_message(*message);
                let reply = Message::reply(state, self, BroadcastPayload::BroadcastOk);
                reply.write(writer)?;
            }
            BroadcastPayload::Read => {
                let reply = Message::reply(
                    state,
                    self,
                    BroadcastPayload::ReadOk {
                        messages: state.messages.clone(),
                    },
                );
                reply.write(writer)?;
            }
            BroadcastPayload::Topology { topology } => {
                state.update_topology(topology);
                let reply = Message::reply(state, self, BroadcastPayload::TopologyOk);
                reply.write(writer)?;
            }
            BroadcastPayload::Gossip { messages: _ } => todo!(),
            BroadcastPayload::GossipOk { messages: _ } => todo!(),
            BroadcastPayload::BroadcastOk
            | BroadcastPayload::ReadOk { .. }
            | BroadcastPayload::TopologyOk => {}
        }
        Ok(())
    }
}

impl Node<BroadcastPayload, GeneratedPayload> for Event<BroadcastPayload, GeneratedPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => bail!("Unexpected message EOF"),
            Event::GeneratedMessage(message) => match message.body.payload {
                GeneratedPayload::Gossip => {}
            },
            Event::ReceivedMessage(received_message) => match &received_message.body.payload {
                BroadcastPayload::Broadcast { message } => {
                    state.add_message(*message);
                    let reply =
                        Message::reply(state, received_message, BroadcastPayload::BroadcastOk);
                    reply.write(writer)?;
                }
                BroadcastPayload::Read => {
                    let reply = Message::reply(
                        state,
                        received_message,
                        BroadcastPayload::ReadOk {
                            messages: state.messages.clone(),
                        },
                    );
                    reply.write(writer)?;
                }
                BroadcastPayload::Topology { topology } => {
                    state.update_topology(topology);
                    let reply =
                        Message::reply(state, received_message, BroadcastPayload::TopologyOk);
                    reply.write(writer)?;
                }

                BroadcastPayload::Gossip { messages: _ } => todo!(),
                BroadcastPayload::GossipOk { messages: _ } => todo!(),
                BroadcastPayload::BroadcastOk
                | BroadcastPayload::ReadOk { .. }
                | BroadcastPayload::TopologyOk => {}
            },
        }
        Ok(())
    }
}
