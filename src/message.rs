use std::io::{StdoutLock, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::State;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<Payload> {
    #[serde(rename = "src")]
    pub source: String,
    #[serde(rename = "dest")]
    pub destination: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload>
where
    Payload: DeserializeOwned + Serialize,
{
    pub fn new(source: String, destination: String, body: Body<Payload>) -> Self {
        Self {
            source,
            destination,
            body,
        }
    }
    pub fn reply(state: &mut State, request: &Message<Payload>, payload: Payload) -> Self {
        let body = Body::new(
            Some(state.get_and_increment()),
            payload,
            request.body.message_id,
        );
        Self {
            source: request.destination.clone(),
            destination: request.source.clone(),
            body,
        }
    }
    pub fn write(&self, writer: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *writer, self)?;
        writer.write_all(b"\r")?;
        writer.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub message_id: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
    pub in_reply_to: Option<usize>,
}

impl<Payload> Body<Payload>
where
    Payload: DeserializeOwned + Serialize,
{
    pub fn new(message_id: Option<usize>, payload: Payload, in_reply_to: Option<usize>) -> Self {
        Self {
            message_id,
            payload,
            in_reply_to,
        }
    }
}
