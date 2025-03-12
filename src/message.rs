use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<Payload> {
    #[serde(rename = "src")]
    source: String,
    #[serde(rename = "dest")]
    destination: String,
    body: Body<Payload>,
}

impl<Payload> Message<Payload> {
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

impl<Payload> Body<Payload> {
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
