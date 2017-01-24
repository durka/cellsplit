use std::path::Path;

use super::Result;

pub fn collapse<P: AsRef<Path>>(_path: P, _overwrite: bool) -> Result<()> {
    Err("collapsing is not yet implemented")?
}

