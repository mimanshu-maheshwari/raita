use raita::{main_loop, state::State, unique_id::UniqueIdPayload};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    main_loop::<UniqueIdPayload>(state)
}
