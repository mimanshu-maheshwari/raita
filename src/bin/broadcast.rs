use raita::{node, BroadcastPayload, State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    node::start::<BroadcastPayload>(state)
}
