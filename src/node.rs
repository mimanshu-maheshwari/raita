use crate::{message::Event, GeneratedPayload, InitPayload, Message, State};
use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{stdin, stdout, BufRead, BufReader, Write},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub trait Node<Payload>
where
    Payload: DeserializeOwned + Serialize,
{
    fn step(&self, writer: &mut impl Write, state: &mut State) -> anyhow::Result<()>;
}

#[inline(always)]
pub fn start<Payload>(mut state: State) -> anyhow::Result<()>
where
    Payload: Sized + DeserializeOwned + Serialize + Send + 'static,
    Event<Payload>: Node<Payload>,
{
    let (tx, rx) = mpsc::channel();

    let mut stdout = stdout().lock();

    let stdin = stdin().lock();
    let mut input_buffer = String::new();
    let mut reader = BufReader::new(stdin);

    reader.read_line(&mut input_buffer)?;
    let init_message: Message<InitPayload> = serde_json::from_str(&input_buffer)?;

    // we'll start gossiping when we get init message.
    // we can implement raft concensus algorithm as well only the two phase commit part.
    init_message.step(&mut stdout, &mut state)?;

    let interval_tx = tx.clone();
    let _interval_handler = interval_event_generator(interval_tx);

    drop(init_message);
    drop(input_buffer);
    drop(reader);

    // thread for stdin
    let stdin_tx = tx.clone();
    let stdin_handler = stdin_handler(stdin_tx);

    for message in rx {
        message.step(&mut stdout, &mut state)?;
    }

    stdin_handler
        .join()
        .expect("Stdin thread panicked")
        .context("stdin thread err")?;
    Ok(())
}

fn interval_event_generator<Payload>(
    interval_tx: Sender<Event<Payload>>,
) -> JoinHandle<Result<(), anyhow::Error>>
where
    Payload: Sized + DeserializeOwned + Serialize + Send + 'static,
    GeneratedPayload: Sized + Send + 'static,
    Event<Payload>: Node<Payload>,
{
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            if interval_tx.send(Event::GeneratedMessage).is_err() {
                break;
            }
        }
        let _ = interval_tx.send(Event::EndOfFile);
        Ok::<(), anyhow::Error>(())
    })
}

fn stdin_handler<Payload>(stdin_tx: Sender<Event<Payload>>) -> JoinHandle<Result<(), anyhow::Error>>
where
    Payload: Sized + DeserializeOwned + Serialize + Send + 'static,
    Event<Payload>: Node<Payload>,
{
    thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        let mut input_buffer = String::new();
        let mut reader = BufReader::new(stdin);

        input_buffer.clear();
        while let Ok(bytes) = reader.read_line(&mut input_buffer) {
            if bytes == 0 {
                break;
            }
            let message: Message<Payload> = serde_json::from_str(&input_buffer)?;
            if stdin_tx.send(Event::ReceivedMessage(message)).is_err() {
                return Ok::<_, anyhow::Error>(());
            }
            input_buffer.clear();
        }
        stdin_tx
            .send(Event::EndOfFile)
            .expect("Failed to send EOF event.");
        Ok::<(), anyhow::Error>(())
    })
}
