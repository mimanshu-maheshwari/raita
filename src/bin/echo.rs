use raita::{node, EchoPayload, State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    node::start::<EchoPayload, ()>(state)
}
