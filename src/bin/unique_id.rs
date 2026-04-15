use raita::{node, State, UniqueIdPayload};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    node::start::<UniqueIdPayload, _>(state)
}
