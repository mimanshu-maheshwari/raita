use crate::{
    message::{Body, Event, Message},
    state::State,
    Node,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastPayload {
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<u32>,
    },
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk,
    Gossip {
        messages: HashSet<u32>,
    },
}

#[derive(Debug, Clone)]
pub enum GeneratedPayload {
    Gossip,
}

fn send_gossip_to_neighbors(
    writer: &mut impl Write,
    state: &mut State,
    skip_node: Option<&str>,
) -> anyhow::Result<()> {
    let neighbors: Vec<String> = state.neighborhood.iter().cloned().collect();
    for neighbor in neighbors {
        if skip_node.is_some_and(|skip| skip == neighbor) {
            continue;
        }

        let messages = state.messages_unknown_to(&neighbor);
        if messages.is_empty() {
            continue;
        }

        let message_id = state.get_and_increment();
        Message::new(
            state.node_id.clone(),
            neighbor.clone(),
            Body::new(
                Some(message_id),
                BroadcastPayload::Gossip {
                    messages: messages.clone(),
                },
                None,
            ),
        )
        .write(writer)?;
        state.mark_known(&neighbor, messages.iter());
    }

    Ok(())
}

fn apply_gossip(state: &mut State, source: &str, messages: &HashSet<u32>) -> bool {
    let previous_len = state.messages.len();
    state.add_messages(messages.iter());
    state.mark_known(source, messages.iter());
    state.messages.len() > previous_len
}

impl Node<BroadcastPayload> for Message<BroadcastPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match &self.body.payload {
            BroadcastPayload::Broadcast { message } => {
                state.add_message(*message);
                let reply = Message::reply(state, self, BroadcastPayload::BroadcastOk);
                reply.write(writer)?;
                send_gossip_to_neighbors(writer, state, None)?;
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
            BroadcastPayload::Gossip { messages } => {
                if apply_gossip(state, &self.source, messages) {
                    send_gossip_to_neighbors(writer, state, Some(&self.source))?;
                }
            }
            BroadcastPayload::BroadcastOk
            | BroadcastPayload::ReadOk { .. }
            | BroadcastPayload::TopologyOk => {}
        }
        Ok(())
    }
}

impl Node<BroadcastPayload> for Event<BroadcastPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => {}
            Event::GeneratedMessage => {
                send_gossip_to_neighbors(writer, state, None)?;
            }
            Event::ReceivedMessage(received_message) => match &received_message.body.payload {
                BroadcastPayload::Broadcast { message } => {
                    state.add_message(*message);
                    let reply =
                        Message::reply(state, received_message, BroadcastPayload::BroadcastOk);
                    reply.write(writer)?;
                    send_gossip_to_neighbors(writer, state, None)?;
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
                BroadcastPayload::Gossip { messages } => {
                    if apply_gossip(state, &received_message.source, messages) {
                        send_gossip_to_neighbors(writer, state, Some(&received_message.source))?;
                    }
                }
                BroadcastPayload::BroadcastOk
                | BroadcastPayload::ReadOk { .. }
                | BroadcastPayload::TopologyOk => {}
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BroadcastPayload;
    use crate::{
        message::{Body, Event, Message},
        state::State,
        Node,
    };
    use std::collections::{HashMap, HashSet};

    fn parse_messages(output: &[u8]) -> Vec<Message<BroadcastPayload>> {
        let text = String::from_utf8(output.to_vec()).expect("valid utf8 output");
        text.split('\r')
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("valid message json"))
            .collect()
    }

    #[test]
    fn generated_gossip_only_sends_unknown_messages() {
        let mut state = State {
            node_id: "n1".to_owned(),
            neighborhood: HashSet::from(["n2".to_owned(), "n3".to_owned()]),
            messages: HashSet::from([1, 2, 3]),
            known: HashMap::from([
                ("n2".to_owned(), HashSet::from([1, 3])),
                ("n3".to_owned(), HashSet::new()),
            ]),
            ..State::default()
        };
        let mut output = Vec::new();

        Event::<BroadcastPayload>::GeneratedMessage
            .step(&mut output, &mut state)
            .expect("gossip generation succeeds");

        let messages = parse_messages(&output);
        assert_eq!(messages.len(), 2);

        let gossip_to_n2 = messages
            .iter()
            .find(|message| message.destination == "n2")
            .expect("message to n2");
        let gossip_to_n3 = messages
            .iter()
            .find(|message| message.destination == "n3")
            .expect("message to n3");

        assert_eq!(
            gossip_to_n2.body.payload,
            BroadcastPayload::Gossip {
                messages: HashSet::from([2]),
            }
        );
        assert_eq!(
            gossip_to_n3.body.payload,
            BroadcastPayload::Gossip {
                messages: HashSet::from([1, 2, 3]),
            }
        );
        assert_eq!(state.known["n2"], HashSet::from([1, 2, 3]));
        assert_eq!(state.known["n3"], HashSet::from([1, 2, 3]));
    }

    #[test]
    fn received_gossip_marks_sender_known_and_forwards_to_other_neighbors() {
        let mut state = State {
            node_id: "n1".to_owned(),
            neighborhood: HashSet::from(["n2".to_owned(), "n3".to_owned()]),
            known: HashMap::from([
                ("n2".to_owned(), HashSet::new()),
                ("n3".to_owned(), HashSet::new()),
            ]),
            ..State::default()
        };
        let mut output = Vec::new();
        let event = Event::ReceivedMessage(Message::new(
            "n2".to_owned(),
            "n1".to_owned(),
            Body::new(
                Some(7),
                BroadcastPayload::Gossip {
                    messages: HashSet::from([9, 11]),
                },
                None,
            ),
        ));

        event.step(&mut output, &mut state).expect("gossip is handled");

        let messages = parse_messages(&output);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].destination, "n3");
        assert_eq!(
            messages[0].body.payload,
            BroadcastPayload::Gossip {
                messages: HashSet::from([9, 11]),
            }
        );
        assert_eq!(state.messages, HashSet::from([9, 11]));
        assert_eq!(state.known["n2"], HashSet::from([9, 11]));
        assert_eq!(state.known["n3"], HashSet::from([9, 11]));
    }

    #[test]
    fn broadcast_event_replies_and_fans_out_immediately() {
        let mut state = State {
            node_id: "n1".to_owned(),
            neighborhood: HashSet::from(["n2".to_owned()]),
            known: HashMap::from([("n2".to_owned(), HashSet::new())]),
            ..State::default()
        };
        let mut output = Vec::new();
        let event = Event::ReceivedMessage(Message::new(
            "client".to_owned(),
            "n1".to_owned(),
            Body::new(
                Some(3),
                BroadcastPayload::Broadcast { message: 42 },
                None,
            ),
        ));

        event
            .step(&mut output, &mut state)
            .expect("broadcast is handled");

        let messages = parse_messages(&output);
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().any(|message| matches!(
            message.body.payload,
            BroadcastPayload::BroadcastOk
        )));
        assert!(messages.iter().any(|message| {
            message.destination == "n2"
                && message.body.payload
                    == BroadcastPayload::Gossip {
                        messages: HashSet::from([42]),
                    }
        }));
    }
}
