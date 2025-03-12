use raita::{echo::EchoPayload, main_loop, state::State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    main_loop::<EchoPayload>(state)
}
