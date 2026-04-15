use crate::{
    message::{Body, Event, Message},
    node::HasCommonState,
    Node, State,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

const KEY_DOES_NOT_EXIST: u32 = 20;
const PRECONDITION_FAILED: u32 = 22;
const SEQ_KV_NODE: &str = "seq-kv";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CounterPayload {
    Add {
        delta: u64,
    },
    AddOk,
    Read {
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },
    ReadOk {
        value: u64,
    },
    Cas {
        key: String,
        from: u64,
        to: u64,
        create_if_not_exists: bool,
    },
    CasOk,
    Error {
        code: u32,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClientRequest {
    source: String,
    destination: String,
    request_message_id: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingCounterRpc {
    AddRead {
        client: ClientRequest,
        delta: u64,
    },
    AddCas {
        client: ClientRequest,
        delta: u64,
    },
    ReadShard {
        aggregate_id: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingRead {
    client: ClientRequest,
    remaining: usize,
    total: u64,
}

#[derive(Debug, Default)]
pub struct CounterState {
    pub inner: State,
    pending_rpcs: HashMap<usize, PendingCounterRpc>,
    pending_reads: HashMap<usize, PendingRead>,
}

impl HasCommonState for CounterState {
    fn state(&mut self) -> &mut State {
        &mut self.inner
    }
}

impl CounterState {
    fn counter_key(&self, node_id: &str) -> String {
        format!("g-counter::{node_id}")
    }

    fn remember_pending(&mut self, message_id: usize, pending: PendingCounterRpc) {
        self.pending_rpcs.insert(message_id, pending);
    }

    fn take_pending(&mut self, in_reply_to: Option<usize>) -> Option<PendingCounterRpc> {
        in_reply_to.and_then(|message_id| self.pending_rpcs.remove(&message_id))
    }
}

fn client_request(message: &Message<CounterPayload>) -> ClientRequest {
    ClientRequest {
        source: message.source.clone(),
        destination: message.destination.clone(),
        request_message_id: message.body.message_id,
    }
}

fn reply_to_client(
    writer: &mut impl Write,
    state: &mut CounterState,
    client: &ClientRequest,
    payload: CounterPayload,
) -> anyhow::Result<()> {
    Message::new(
        client.destination.clone(),
        client.source.clone(),
        Body::new(
            Some(state.inner.get_and_increment()),
            payload,
            client.request_message_id,
        ),
    )
    .write(writer)
}

fn send_kv_read(
    writer: &mut impl Write,
    state: &mut CounterState,
    key: String,
    pending: PendingCounterRpc,
) -> anyhow::Result<()> {
    let message_id = state.inner.get_and_increment();
    state.remember_pending(message_id, pending);
    Message::new(
        state.inner.node_id.clone(),
        SEQ_KV_NODE.to_owned(),
        Body::new(
            Some(message_id),
            CounterPayload::Read { key: Some(key) },
            None,
        ),
    )
    .write(writer)
}

fn send_kv_cas(
    writer: &mut impl Write,
    state: &mut CounterState,
    key: String,
    from: u64,
    to: u64,
    create_if_not_exists: bool,
    pending: PendingCounterRpc,
) -> anyhow::Result<()> {
    let message_id = state.inner.get_and_increment();
    state.remember_pending(message_id, pending);
    Message::new(
        state.inner.node_id.clone(),
        SEQ_KV_NODE.to_owned(),
        Body::new(
            Some(message_id),
            CounterPayload::Cas {
                key,
                from,
                to,
                create_if_not_exists,
            },
            None,
        ),
    )
    .write(writer)
}

fn handle_read_completion(
    writer: &mut impl Write,
    state: &mut CounterState,
    aggregate_id: usize,
    contribution: u64,
) -> anyhow::Result<()> {
    let mut should_reply = None;

    if let Some(pending) = state.pending_reads.get_mut(&aggregate_id) {
        pending.remaining -= 1;
        pending.total += contribution;

        if pending.remaining == 0 {
            should_reply = Some((pending.client.clone(), pending.total));
        }
    }

    if let Some((client, total)) = should_reply {
        state.pending_reads.remove(&aggregate_id);
        reply_to_client(writer, state, &client, CounterPayload::ReadOk { value: total })?;
    }

    Ok(())
}

fn restart_add(
    writer: &mut impl Write,
    state: &mut CounterState,
    client: ClientRequest,
    delta: u64,
) -> anyhow::Result<()> {
    let key = state.counter_key(&state.inner.node_id);
    send_kv_read(writer, state, key, PendingCounterRpc::AddRead { client, delta })
}

impl Node<CounterPayload, CounterState> for Message<CounterPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut CounterState) -> anyhow::Result<()> {
        match &self.body.payload {
            CounterPayload::Add { delta } => {
                restart_add(writer, state, client_request(self), *delta)?;
            }
            CounterPayload::Read { key: None } => {
                let client = client_request(self);
                let aggregate_id = state.inner.get_and_increment();
                let nodes: Vec<String> = state.inner.declared_nodes.iter().cloned().collect();

                if nodes.is_empty() {
                    reply_to_client(writer, state, &client, CounterPayload::ReadOk { value: 0 })?;
                    return Ok(());
                }

                state.pending_reads.insert(
                    aggregate_id,
                    PendingRead {
                        client,
                        remaining: nodes.len(),
                        total: 0,
                    },
                );

                for node in nodes {
                    let key = state.counter_key(&node);
                    send_kv_read(writer, state, key, PendingCounterRpc::ReadShard { aggregate_id })?;
                }
            }
            CounterPayload::ReadOk { value } if self.source == SEQ_KV_NODE => {
                match state.take_pending(self.body.in_reply_to) {
                    Some(PendingCounterRpc::AddRead { client, delta }) => {
                        let key = state.counter_key(&state.inner.node_id);
                        send_kv_cas(
                            writer,
                            state,
                            key,
                            *value,
                            value + delta,
                            false,
                            PendingCounterRpc::AddCas { client, delta },
                        )?;
                    }
                    Some(PendingCounterRpc::ReadShard { aggregate_id }) => {
                        handle_read_completion(writer, state, aggregate_id, *value)?;
                    }
                    Some(PendingCounterRpc::AddCas { .. }) | None => {}
                }
            }
            CounterPayload::CasOk => match state.take_pending(self.body.in_reply_to) {
                Some(PendingCounterRpc::AddCas { client, .. }) => {
                    reply_to_client(writer, state, &client, CounterPayload::AddOk)?;
                }
                Some(PendingCounterRpc::AddRead { .. })
                | Some(PendingCounterRpc::ReadShard { .. })
                | None => {}
            },
            CounterPayload::Error { code, text } => match state.take_pending(self.body.in_reply_to) {
                Some(PendingCounterRpc::AddRead { client, delta }) => {
                    if *code == KEY_DOES_NOT_EXIST {
                        let key = state.counter_key(&state.inner.node_id);
                        send_kv_cas(
                            writer,
                            state,
                            key,
                            0,
                            delta,
                            true,
                            PendingCounterRpc::AddCas { client, delta },
                        )?;
                    } else {
                        reply_to_client(
                            writer,
                            state,
                            &client,
                            CounterPayload::Error {
                                code: *code,
                                text: text.clone(),
                            },
                        )?;
                    }
                }
                Some(PendingCounterRpc::AddCas { client, delta }) => {
                    if *code == PRECONDITION_FAILED {
                        restart_add(writer, state, client, delta)?;
                    } else {
                        reply_to_client(
                            writer,
                            state,
                            &client,
                            CounterPayload::Error {
                                code: *code,
                                text: text.clone(),
                            },
                        )?;
                    }
                }
                Some(PendingCounterRpc::ReadShard { aggregate_id }) => {
                    if *code == KEY_DOES_NOT_EXIST {
                        handle_read_completion(writer, state, aggregate_id, 0)?;
                    } else if let Some(pending) = state.pending_reads.remove(&aggregate_id) {
                        reply_to_client(
                            writer,
                            state,
                            &pending.client,
                            CounterPayload::Error {
                                code: *code,
                                text: text.clone(),
                            },
                        )?;
                    }
                }
                None => {}
            },
            CounterPayload::AddOk
            | CounterPayload::ReadOk { .. }
            | CounterPayload::Read { key: Some(..) }
            | CounterPayload::Cas { .. } => {}
        }

        Ok(())
    }
}

impl Node<CounterPayload, CounterState> for Event<CounterPayload> {
    fn step(&self, writer: &mut impl Write, state: &mut CounterState) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile | Event::GeneratedMessage => {}
            Event::ReceivedMessage(message) => message.step(writer, state)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CounterPayload, CounterState, KEY_DOES_NOT_EXIST};
    use crate::{
        message::{Body, Event, Message},
        Node,
    };
    use std::collections::HashSet;

    fn parse_messages(output: &[u8]) -> Vec<Message<CounterPayload>> {
        let text = String::from_utf8(output.to_vec()).expect("valid utf8 output");
        text.split('\r')
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("valid json"))
            .collect()
    }

    #[test]
    fn add_request_starts_with_seq_kv_read() {
        let mut state = CounterState::default();
        state.inner.node_id = "n1".to_owned();
        let mut output = Vec::new();

        Event::ReceivedMessage(Message::new(
            "c1".to_owned(),
            "n1".to_owned(),
            Body::new(Some(9), CounterPayload::Add { delta: 4 }, None),
        ))
        .step(&mut output, &mut state)
        .expect("add should be handled");

        let messages = parse_messages(&output);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].destination, "seq-kv");
        assert_eq!(
            messages[0].body.payload,
            CounterPayload::Read {
                key: Some("g-counter::n1".to_owned()),
            }
        );
    }

    #[test]
    fn missing_key_add_retries_with_create_if_missing_cas_and_replies() {
        let mut state = CounterState::default();
        state.inner.node_id = "n1".to_owned();
        let mut output = Vec::new();

        Event::ReceivedMessage(Message::new(
            "c1".to_owned(),
            "n1".to_owned(),
            Body::new(Some(9), CounterPayload::Add { delta: 4 }, None),
        ))
        .step(&mut output, &mut state)
        .expect("add request should be accepted");

        output.clear();
        Event::ReceivedMessage(Message::new(
            "seq-kv".to_owned(),
            "n1".to_owned(),
            Body::new(
                Some(20),
                CounterPayload::Error {
                    code: KEY_DOES_NOT_EXIST,
                    text: "missing".to_owned(),
                },
                Some(0),
            ),
        ))
        .step(&mut output, &mut state)
        .expect("missing key should trigger cas");

        let messages = parse_messages(&output);
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].body.payload,
            CounterPayload::Cas {
                key: "g-counter::n1".to_owned(),
                from: 0,
                to: 4,
                create_if_not_exists: true,
            }
        );

        output.clear();
        Event::ReceivedMessage(Message::new(
            "seq-kv".to_owned(),
            "n1".to_owned(),
            Body::new(Some(21), CounterPayload::CasOk, Some(1)),
        ))
        .step(&mut output, &mut state)
        .expect("cas should reply to client");

        let reply = parse_messages(&output);
        assert_eq!(reply.len(), 1);
        assert_eq!(reply[0].destination, "c1");
        assert_eq!(reply[0].body.payload, CounterPayload::AddOk);
    }

    #[test]
    fn read_request_aggregates_all_node_shards() {
        let mut state = CounterState::default();
        state.inner.node_id = "n1".to_owned();
        state.inner.declared_nodes = HashSet::from([
            "n0".to_owned(),
            "n1".to_owned(),
            "n2".to_owned(),
        ]);
        let mut output = Vec::new();

        Event::ReceivedMessage(Message::new(
            "c9".to_owned(),
            "n1".to_owned(),
            Body::new(Some(42), CounterPayload::Read { key: None }, None),
        ))
        .step(&mut output, &mut state)
        .expect("read request should start shard reads");

        let requests = parse_messages(&output);
        assert_eq!(requests.len(), 3);
        let request_ids: Vec<usize> = requests
            .iter()
            .map(|request| request.body.message_id.expect("kv read should have msg id"))
            .collect();

        output.clear();
        for in_reply_to in request_ids {
            Event::ReceivedMessage(Message::new(
                "seq-kv".to_owned(),
                "n1".to_owned(),
                Body::new(
                    Some(50 + in_reply_to),
                    CounterPayload::ReadOk { value: 2 },
                    Some(in_reply_to),
                ),
            ))
            .step(&mut output, &mut state)
            .expect("read shard should be processed");
        }

        let reply = parse_messages(&output);
        assert_eq!(reply.len(), 1);
        assert_eq!(reply[0].destination, "c9");
        assert_eq!(reply[0].body.payload, CounterPayload::ReadOk { value: 6 });
    }
}
