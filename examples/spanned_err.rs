use spanned::Spanned;

fn main() -> Result<(), Spanned<&'static str>> {
    Err(Spanned::here("kaboom"))?;
    Ok(())
}
