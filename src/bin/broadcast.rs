use raita::{node, BroadcastPayload, GeneratedPayload, State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    node::start::<BroadcastPayload>(state)
}
