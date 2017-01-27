use std::io;
use std::path::Path;

use ResultExt;
use super::{Result, ErrorKind};

pub struct Actor;

impl Actor {
    pub fn new() -> Self {
        Actor
    }

    pub fn perform<R, F: FnOnce() -> io::Result<R>, P: AsRef<Path>>(&self, f: F, descr: &'static str, target: P) -> Result<R> {
        f().chain_err(|| ErrorKind::IoFailed(descr, target.as_ref().to_owned()))
    }
}

