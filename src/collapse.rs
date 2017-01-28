use std::io::{BufRead, BufReader, Write};
use std::fs;
use std::path::Path;

use regex::Regex;

use super::Result;
use actor::Actor;

fn collapse_into<W: Write>(actor: &Actor, from: &Path, to: &mut W) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\s*([^\s]+) %cellsplit<\d+>$").unwrap();
    }

    for line in BufReader::new(actor.open(from)?).lines().skip(1) {
        let line = actor.perform(|| line, "read line from", from)?;
        if let Some(caps) = RE.captures(&line) {
            if let Some(scriptname) = caps.get(1) {
                let scriptpath = from.with_file_name(format!("{}.m", scriptname.as_str()));
                collapse_into(actor, &scriptpath, to)?;
            }
        } else {
            actor.writeln(to, &line, "script file")?;
        }
    }

    Ok(())
}


pub fn collapse<P: AsRef<Path>>(actor: Actor, path: P) -> Result<()> {
    let outpath = actor.perform(|| fs::canonicalize(path.as_ref()), "canonicalize", path.as_ref())?;
    let genpath = outpath.with_file_name(format!("{}_gen.m", outpath.file_stem().ok_or("no filename")?.to_str().ok_or("non-Unicode filename")?));

    collapse_into(&actor, &genpath, &mut actor.create(&outpath)?)
}

