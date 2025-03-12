use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

use crate::{
    message::{Body, Message},
    state::State,
    Node,
};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastPayload {}

impl Node<BroadcastPayload> for Message<BroadcastPayload> {
    fn step(&self, writer: &mut StdoutLock, state: &mut State) -> anyhow::Result<()> {
        Ok(())
    }
}
