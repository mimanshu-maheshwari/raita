use raita::{node, CounterPayload, CounterState};

fn main() -> anyhow::Result<()> {
    node::start::<CounterPayload, _>(CounterState::default())
}
