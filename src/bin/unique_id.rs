use raita::{main_loop, State, UniqueIdPayload};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    main_loop::<UniqueIdPayload>(state)
}
