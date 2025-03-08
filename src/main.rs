type Result<RET> = std::result::Result<RET, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    Ok(())
}
