use crate::State;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone)]
pub enum Event<Payload> {
    ReceivedMessage(Message<Payload>),
    GeneratedMessage,
    EndOfFile,
}

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
    #[inline(always)]
    pub fn new(source: String, destination: String, body: Body<Payload>) -> Self {
        Self {
            source,
            destination,
            body,
        }
    }
    #[inline(always)]
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
    #[inline(always)]
    pub fn write(&self, writer: &mut impl Write) -> anyhow::Result<()> {
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
    #[inline(always)]
    pub fn new(message_id: Option<usize>, payload: Payload, in_reply_to: Option<usize>) -> Self {
        Self {
            message_id,
            payload,
            in_reply_to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Body, Message};
    use crate::State;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum TestPayload {
        Ping,
        Pong { value: u32 },
    }

    #[test]
    fn reply_reuses_request_message_id_and_increments_state_counter() {
        let request = Message::new(
            "client".to_owned(),
            "node-a".to_owned(),
            Body::new(Some(7), TestPayload::Ping, None),
        );
        let mut state = State::default();

        let reply = Message::reply(&mut state, &request, TestPayload::Pong { value: 99 });

        assert_eq!(reply.source, "node-a");
        assert_eq!(reply.destination, "client");
        assert_eq!(reply.body.message_id, Some(0));
        assert_eq!(reply.body.in_reply_to, Some(7));
        assert_eq!(reply.body.payload, TestPayload::Pong { value: 99 });
        assert_eq!(state.message_track_id, 1);
    }
}
