use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<Payload> {
    #[serde(rename = "src")]
    source: String,
    #[serde(rename = "dest")]
    destination: String,
    body: Body<Payload>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    message_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Echo {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
        in_reply_to: Option<usize>,
    },
}
