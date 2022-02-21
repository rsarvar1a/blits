
use lits::*;
use utils::*;

fn main() -> Result<()>
{
    log::initialize(".", "client");
    Tetromino::initialize();

    Ok(())
}
