use raita::{main_loop, EchoPayload, State};

fn main() -> anyhow::Result<()> {
    let state = State::default();
    main_loop::<EchoPayload>(state)
}
