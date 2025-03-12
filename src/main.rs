use raita::{echo::EchoPayload, main_loop};

fn main() -> anyhow::Result<()> {
    main_loop::<EchoPayload>()
}
