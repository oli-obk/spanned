use spanned::Spanned;

fn main() -> anyhow::Result<()> {
    Err(Spanned::here("kaboom"))?;
    Ok(())
}
