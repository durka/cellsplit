use std::path::Path;

use super::Result;
use actor::Actor;

pub fn collapse<P: AsRef<Path>>(_actor: Actor, _path: P, _overwrite: bool) -> Result<()> {
    Err("collapsing is not yet implemented")?
}

