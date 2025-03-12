use raita::{main_loop, BroadcastPayload, State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    main_loop::<BroadcastPayload>(state)
}
