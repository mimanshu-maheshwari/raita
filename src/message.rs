use std::io::{StdoutLock, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<Payload> {
    #[serde(rename = "src")]
    source: String,
    #[serde(rename = "dest")]
    destination: String,
    body: Body<Payload>,
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
    pub fn reply(request: &Message<Payload>, body: Body<Payload>) -> Self {
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

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn body(&self) -> &Body<Payload> {
        &self.body
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    message_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

impl<Payload> Body<Payload>
where
    Payload: DeserializeOwned + Serialize,
{
    pub fn new(message_id: Option<usize>, payload: Payload) -> Self {
        Self {
            message_id,
            payload,
        }
    }

    pub fn message_id(&self) -> Option<usize> {
        self.message_id
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}
