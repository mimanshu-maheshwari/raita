use crate::{
    message::{Event, Message},
    node::HasCommonState,
    Node, State,
};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Write};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: HashSet<String>,
    },
    InitOk,
}

fn apply_init(
    message: &Message<InitPayload>,
    writer: &mut impl Write,
    state: &mut State,
) -> anyhow::Result<()> {
    match &message.body.payload {
        InitPayload::Init { node_id, node_ids } => {
            state.node_id = node_id.clone();
            state.declared_nodes = node_ids.clone();
            state.update_neighborhood();
            state.create_known(node_ids);
            Message::reply(state, message, InitPayload::InitOk).write(writer)?;
        }
        InitPayload::InitOk => bail!("Unexpected message Init Ok"),
    }
    Ok(())
}

impl<S> Node<InitPayload, S> for Message<InitPayload>
where
    S: HasCommonState,
{
    fn step(&self, writer: &mut impl Write, state: &mut S) -> anyhow::Result<()> {
        apply_init(self, writer, state.state())
    }
}

impl<S> Node<InitPayload, S> for Event<InitPayload>
where
    S: HasCommonState,
{
    fn step(&self, writer: &mut impl Write, state: &mut S) -> anyhow::Result<()> {
        match self {
            Event::EndOfFile => bail!("Unexpected message EOF"),
            Event::GeneratedMessage => {}
            Event::ReceivedMessage(message) => apply_init(message, writer, state.state())?,
        };
        Ok(())
    }
}
