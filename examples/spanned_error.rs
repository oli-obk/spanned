use spanned::{Error, Spanned};

fn main() -> Result<(), Error> {
    parse().map_err(|err| err.wrap_str(Spanned::here("woosh")))?;
    Ok(())
}

fn parse() -> Result<(), Error> {
    Err(Error::str("kaboom"))?;
    Ok(())
}
