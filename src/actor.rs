use std::fs::{File, OpenOptions};
use std::io::{self, Write};
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

    pub fn open<P: AsRef<Path>>(&self, name: P) -> Result<File> {
        self.perform(|| File::open(name.as_ref()), "open", name.as_ref())
    }

    pub fn create<P: AsRef<Path>>(&self, name: P) -> Result<File> {
        self.perform(|| File::create(name.as_ref()), "open", name.as_ref())
    }

    pub fn open_options<P: AsRef<Path>>(&self, options: &mut OpenOptions, name: P) -> Result<File> {
        self.perform(|| options.open(name.as_ref()), "open", name.as_ref())
    }

    pub fn write<W: Write, P: AsRef<Path>>(&self, to: &mut W, line: &str, name: P) -> Result<()> {
        self.perform(|| write!(to, "{}", line), "write line to", name)
    }

    pub fn writeln<W: Write, P: AsRef<Path>>(&self, to: &mut W, line: &str, name: P) -> Result<()> {
        self.perform(|| writeln!(to, "{}", line), "write line to", name)
    }
}

