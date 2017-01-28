use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use regex::Regex;

use super::Result;
use actor::Actor;

fn try_create<P: AsRef<Path>>(actor: &Actor, path: P, overwrite: bool) -> Result<File> {
    actor.open_options(OpenOptions::new()
                   .write(true)
                   .create(true).truncate(true)
                   .create_new(!overwrite),
               path.as_ref())
}

fn slugify(s: &str, limit: usize) -> String {
    let slug = ::slug::slugify(s.trim()).replace('-', "_");
    slug[..slug.char_indices().nth(limit).map_or(slug.len(), |(i,_)| i)].to_string()
}

fn start_cell<W: Write>(actor: &Actor, inpath: &Path, file: &mut W, pat: &str, n: i32, overwrite: bool, title: &str) -> Result<File> {
    let name = pat.replace("{}", &format!("{}_{}", slugify(title.trim_left_matches('%'), 20), n));
    let mut cell = try_create(actor, &name, overwrite)?;
    actor.writeln(&mut cell, &format!("% part {} of {}", n, inpath.display()), &name)?;
    actor.writeln(file, &format!("{} %cellsplit<{}>", Path::new(&name).file_stem().unwrap().to_str().unwrap(), n), "script file")?;
    Ok(cell)
}

fn write_cell(actor: &Actor, cells: &mut Vec<File>, outfile: &mut File, inpath: &Path, outpat: &str, outs: i32, overwrite: bool,
              indent: &str, line: &str) -> Result<()> {
    let trimline = line.trim_left();
    actor.writeln(cells.last_mut().unwrap_or(outfile), line, "script file")?;
    actor.write(cells.last_mut().unwrap_or(outfile), &format!("{}{}", &line[..(line.len() - trimline.len())], indent), "script file")?;
    unborrow!(cells.push(start_cell(actor, inpath, cells.last_mut().unwrap_or(outfile), outpat, outs, overwrite, trimline)?));
    Ok(())
}

fn delete_from(actor: &Actor, file: &Path) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\s*([^\s]+) %cellsplit<\d+>$").unwrap();
    }

    for line in BufReader::new(actor.open(file)?).lines() {
        let line = actor.perform(|| line, "read line from", file)?;
        if let Some(caps) = RE.captures(&line) {
            if let Some(scriptname) = caps.get(1) {
                let scriptpath = file.with_file_name(format!("{}.m", scriptname.as_str()));
                delete_from(actor, &scriptpath)?;
                actor.perform(|| fs::remove_file(&scriptpath), "delete", &scriptpath)?;
            }
        }
    }

    Ok(())
}

pub fn expand<P: AsRef<Path>>(actor: Actor, path: P, overwrite: bool) -> Result<()> {
    let inpath = actor.perform(|| fs::canonicalize(path.as_ref()), "canonicalize", path.as_ref())?;
    let outpat = {
        let mut outpat = inpath.clone();
        let mut stem = outpat.file_stem().ok_or("no filename")?
                             .to_str().ok_or("non-Unicode filename")?
                             .to_string();
        stem.push_str("_{}.");
        stem.push_str(outpat.extension().and_then(|s| s.to_str()).unwrap_or(""));
        outpat.set_file_name(stem);
        outpat.to_str().ok_or("non-Unicode path")?
              .to_string()
    };

    let genname = outpat.replace("{}", "gen");
    println!("Processing cell mode file\n\t\t{}\n\tinto\n\t\t{}\n\tand\n\t\t{}",
             inpath.display(), genname, outpat.replace("{}", "*"));

    // see if there are old files to delete
    if overwrite && Path::new(&genname).exists() {
        println!("Deleting old scripts from {}", genname);
        delete_from(&actor, Path::new(&genname))?;
    }

    let infile = BufReader::new(actor.open(&inpath)?);
    let mut outfile = try_create(&actor, genname, overwrite)?;

    let mut outs = 1;
    let mut cells = vec![start_cell(&actor, &inpath, &mut outfile, &outpat, 0, overwrite, "")?];
    for line in infile.lines() {
        let line = actor.perform(|| line, "read line from", &inpath)?;
        let trimline = line.trim_left();

        // FIXME users could use semicolons to put multiple statements on one line
        if trimline.starts_with("%%") {
            println!("Found cell break: {}", trimline);

            cells.pop().unwrap();
            write_cell(&actor, &mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "", &line)?;
            outs += 1;
        } else if trimline == "end" {
            cells.pop().unwrap();
            actor.writeln(cells.last_mut().unwrap_or(&mut outfile), &line, "script file")?;
        } else if trimline.starts_with("for")
               || trimline.starts_with("while")
               || trimline.starts_with("parfor")
               || trimline.starts_with("if")
               || trimline.starts_with("try") {
            write_cell(&actor, &mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "    ", &line)?;
            outs += 1;
        } else if trimline.starts_with("else")
               || trimline.starts_with("elseif")
               || trimline.starts_with("catch") {
            cells.pop().unwrap();
            write_cell(&actor, &mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "    ", &line)?;
            outs += 1;
        } else if trimline.starts_with("switch") {
            Err("switch is unimplemented")?
        } else {
            // FIXME trim left if inside block inside cell
            actor.writeln(cells.last_mut().unwrap(), &line, "script file")?;
        }
    }

    Ok(())
}

