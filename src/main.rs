use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::{stderr, stdin, stdout},
    str::FromStr,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

impl<Payload> FromStr for Message<Payload>
where
    Payload: DeserializeOwned,
{
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str::<Message<Payload>>(s)?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Body<Payload> {
    msg_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Init {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read data from server and print to std out
    let stdin = stdin().lock();
    let mut stdout = stdout().lock();
    let mut stderr = stderr();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();
    for input in inputs {
        let input = input?;
    }
    Ok(())
}
