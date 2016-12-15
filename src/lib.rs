#[macro_use] extern crate error_type;
#[macro_use] extern crate unborrow;
extern crate slug;

use std::borrow::Cow;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

error_type! {
    #[derive(Debug)]
    pub enum Error {
        Io(io::Error) {
            cause;
        },

        Message(Cow<'static, str>) {
            desc (e) &**e;
            from (s: &'static str) s.into();
            from (s: String) s.into();
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn try_create<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<File> {
    OpenOptions::new()
                .write(true)
                .create(true).truncate(true)
                .create_new(!overwrite)
                .open(path)
                .map_err(Into::into)
}

fn slugify(s: &str, limit: usize) -> String {
    let slug = slug::slugify(s.trim()).replace('-', "_");
    slug[..slug.char_indices().nth(limit).map_or(slug.len(), |(i,_)| i)].to_string()
}

fn start_cell<W: Write>(inpath: &Path, file: &mut W, pat: &str, n: i32, overwrite: bool, title: &str) -> Result<File> {
    let name = pat.replace("{}", &format!("{}_{}", &n.to_string(), slugify(title.trim_left_matches('%'), 20)));
    let mut cell = try_create(&name, overwrite)?;
    writeln!(cell, "% part {} of {}", n, inpath.display())?;
    writeln!(file, "{}", Path::new(&name).file_stem().unwrap().to_str().unwrap())?;
    Ok(cell)
}

fn write_cell(cells: &mut Vec<File>, outfile: &mut File, inpath: &Path, outpat: &str, outs: i32, overwrite: bool,
              indent: &str, line: &str) -> Result<()> {
    let trimline = line.trim_left();
    writeln!(cells.last_mut().unwrap_or(outfile), "{}", line)?;
    write!(cells.last_mut().unwrap_or(outfile), "{}{}", &line[..(line.len() - trimline.len())], indent)?;
    unborrow!(cells.push(start_cell(inpath, cells.last_mut().unwrap_or(outfile), outpat, outs, overwrite, trimline)?));
    Ok(())
}

// TODO name files with slugified section headers
pub fn expand<P: AsRef<Path>>(path: P, overwrite: bool) -> Result<()> {
    let inpath = fs::canonicalize(path.as_ref())?;
    let outpat = {
        let mut outpat = inpath.clone();
        let mut stem = outpat.file_stem().ok_or(Error::from("no filename"))?
                             .to_str().ok_or(Error::from("non-Unicode filename"))?
                             .to_string();
        stem.push_str("_{}.");
        stem.push_str(outpat.extension().and_then(|s| s.to_str()).unwrap_or(""));
        outpat.set_file_name(stem);
        outpat.to_str().ok_or(Error::from("non-Unicode path"))?
              .to_string()
    };
    let infile = BufReader::new(File::open(&inpath)?);
    let mut outfile = try_create(outpat.replace("{}", "gen"), overwrite)?;

    println!("Processing cell mode file\n\t\t{}\n\tinto\n\t\t{}\n\tand\n\t\t{}",
             inpath.display(), outpat.replace("{}", "gen"), outpat.replace("{}", "*"));

    let mut outs = 1;
    let mut cells = vec![start_cell(&inpath, &mut outfile, &outpat, 0, overwrite, "")?];
    for line in infile.lines() {
        let line = line?;
        let trimline = line.trim_left();

        // FIXME users could use semicolons to put multiple statements on one line
        if trimline.starts_with("%%") {
            println!("Found cell break: {}", trimline);

            cells.pop().unwrap();
            write_cell(&mut cells, &mut outfile, &inpath, &outpat, outs, overwrite, "", &line)?;
            outs += 1;
            writeln!(cells.last_mut().unwrap(), "{}", &line[1..])?;
        } else if trimline == "end" {
            cells.pop().unwrap();
            writeln!(cells.last_mut().unwrap_or(&mut outfile), "{}", line)?;
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
            return Err("switch is unimplemented".into());
        } else {
            // FIXME trim left if inside block inside cell
            writeln!(cells.last_mut().unwrap(), "{}", line)?;
        }
    }

    Ok(())
}

pub fn collapse<P: AsRef<Path>>(_path: P, _overwrite: bool) -> Result<()> {
    Err("collapsing is not yet implemented".into())
}

