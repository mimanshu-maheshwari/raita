use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::{stdin, stdout, BufRead, Write},
    str::FromStr,
};

type Result<RET> = std::result::Result<RET, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    fn new(src: String, dest: String, body: Body<Payload>) -> Self {
        Self { src, dest, body }
    }
}

impl<Payload> FromStr for Message<Payload>
where
    Payload: DeserializeOwned,
{
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str::<Message<Payload>>(s)?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Body<Payload> {
    msg_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

impl<Payload> Body<Payload> {
    fn new(msg_id: Option<usize>, payload: Payload) -> Self {
        Self { msg_id, payload }
    }
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

trait Process<Payload> {
    fn step(&self, writer: &mut dyn Write, message: &Message<Payload>) -> Result<()>;
}

impl Process<Init> for Init {
    fn step(&self, writer: &mut dyn Write, message: &Message<Init>) -> Result<()> {
        let init_ok = Init::InitOk {
            in_reply_to: message.body.msg_id.unwrap_or(1),
        };
        let message = Message::new(
            message.dest.clone(),
            message.src.clone(),
            Body::new(message.body.msg_id, init_ok),
        );
        serde_json::to_writer(&mut *writer, &message)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    // read data from server and print to std out
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    // let mut stderr = stderr().lock();
    let mut buf = String::new();
    while let Ok(bytes) = stdin.read_line(&mut buf) {
        if bytes == 0 {
            break;
        }
        let message: Message<Init> = serde_json::from_str(&buf)?;
        message.body.payload.step(&mut stdout, &message)?;
        // stderr.write_all(format!("{message:?}").as_bytes())?;
        // stderr.flush()?;
    }
    Ok(())
}
