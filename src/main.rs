use serde::{Deserialize, Serialize};
use std::io::{stderr, stdin, stdout, BufRead, Write};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message<Payload> {
    src: String,
    #[serde[rename = "dest"]]
    dst: String,
    body: Body<Payload>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Body<Payload> {
    #[serde[rename = "msg_id"]]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
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
