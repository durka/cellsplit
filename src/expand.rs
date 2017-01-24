use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use regex::Regex;

use ResultExt;
use super::{Result, ErrorKind};

fn try_create<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<File> {
    OpenOptions::new()
                .write(true)
                .create(true).truncate(true)
                .create_new(!overwrite)
                .open(path.as_ref())
                .chain_err(|| ErrorKind::IoFailed("create", path.as_ref().to_owned()))
}

fn slugify(s: &str, limit: usize) -> String {
    let slug = ::slug::slugify(s.trim()).replace('-', "_");
    slug[..slug.char_indices().nth(limit).map_or(slug.len(), |(i,_)| i)].to_string()
}

fn start_cell<W: Write>(inpath: &Path, file: &mut W, pat: &str, n: i32, overwrite: bool, title: &str) -> Result<File> {
    let name = pat.replace("{}", &format!("{}_{}", slugify(title.trim_left_matches('%'), 20), n));
    let mut cell = try_create(&name, overwrite)?;
    writeln!(cell, "% part {} of {}", n, inpath.display())
        .chain_err(|| ErrorKind::IoFailed("write to", Path::new(&name).to_owned()))?;
    writeln!(file, "{} %cellsplit<{}>", Path::new(&name).file_stem().unwrap().to_str().unwrap(), n)
        .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
    Ok(cell)
}

fn write_cell(cells: &mut Vec<File>, outfile: &mut File, inpath: &Path, outpat: &str, outs: i32, overwrite: bool,
              indent: &str, line: &str) -> Result<()> {
    let trimline = line.trim_left();
    writeln!(cells.last_mut().unwrap_or(outfile), "{}", line)
        .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
    write!(cells.last_mut().unwrap_or(outfile), "{}{}", &line[..(line.len() - trimline.len())], indent)
        .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
    unborrow!(cells.push(start_cell(inpath, cells.last_mut().unwrap_or(outfile), outpat, outs, overwrite, trimline)?));
    Ok(())
}

fn delete_from(file: &Path) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\s*([^\s]+) %cellsplit<\d+>$").unwrap();
    }

    for line in BufReader::new(File::open(file).chain_err(|| ErrorKind::IoFailed("open", file.to_owned()))?).lines() {
        let line = line.chain_err(|| ErrorKind::IoFailed("read line from", file.to_owned()))?;
        if let Some(caps) = RE.captures(&line) {
            if let Some(scriptname) = caps.get(1) {
                let scriptpath = file.with_file_name(format!("{}.m", scriptname.as_str()));
                delete_from(&scriptpath)?;
                fs::remove_file(&scriptpath).chain_err(|| ErrorKind::IoFailed("delete", scriptpath))?;
            }
        }
    }

    Ok(())
}

pub fn expand<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<()> {
    let inpath = fs::canonicalize(path.as_ref()).chain_err(|| ErrorKind::IoFailed("canonicalize", path.as_ref().to_owned()))?;
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
        delete_from(Path::new(&genname))?;
    }

    let infile = BufReader::new(File::open(&inpath).chain_err(|| ErrorKind::IoFailed("open", inpath.clone()))?);
    let mut outfile = try_create(genname, overwrite)?;

    let mut outs = 1;
    let mut cells = vec![start_cell(&inpath, &mut outfile, &outpat, 0, overwrite, "")?];
    for line in infile.lines() {
        let line = line.chain_err(|| ErrorKind::IoFailed("read line from", inpath.clone()))?;
        let trimline = line.trim_left();

        // FIXME users could use semicolons to put multiple statements on one line
        if trimline.starts_with("%%") {
            println!("Found cell break: {}", trimline);

            cells.pop().unwrap();
            write_cell(&mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "", &line)?;
            outs += 1;
            writeln!(cells.last_mut().unwrap(), "{}", &line[1..])
                .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
        } else if trimline == "end" {
            cells.pop().unwrap();
            writeln!(cells.last_mut().unwrap_or(&mut outfile), "{}", line)
                .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
        } else if trimline.starts_with("for")
               || trimline.starts_with("while")
               || trimline.starts_with("parfor")
               || trimline.starts_with("if")
               || trimline.starts_with("try") {
            write_cell(&mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "    ", &line)?;
            outs += 1;
        } else if trimline.starts_with("else")
               || trimline.starts_with("elseif")
               || trimline.starts_with("catch") {
            cells.pop().unwrap();
            write_cell(&mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "    ", &line)?;
            outs += 1;
        } else if trimline.starts_with("switch") {
            Err("switch is unimplemented")?
        } else {
            // FIXME trim left if inside block inside cell
            writeln!(cells.last_mut().unwrap(), "{}", line)
                .chain_err(|| ErrorKind::IoFailed("write to", Path::new("script file").to_owned()))?;
        }
    }

    Ok(())
}

