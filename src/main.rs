#[macro_use] extern crate clap;

use std::process;

extern crate cellsplit;

fn main() {
    let matches = clap_app!(cellsplit =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@subcommand expand =>
            (@arg INPUT: +required "The cell mode file")
            (@arg overwrite: -o --overwrite "Overwrite output files")
        )
        (@subcommand collapse =>
            (@arg OUTPUT: +required "The generated cell mode file")
            (@arg overwrite: -o --overwrite "Overwrite output file")
        )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("expand") {
        let infile = matches.value_of("INPUT").unwrap();
        if let Err(err) = cellsplit::expand(infile, matches.is_present("overwrite")) {
            println!("ERROR: {}", err);
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("collapse") {
        let outfile = matches.value_of("OUTPUT").unwrap();
        if let Err(err) = cellsplit::collapse(outfile, matches.is_present("overwrite")) {
            println!("ERROR: {}", err);
            process::exit(1);
        }
    } else {
        println!("No subcommand");
        process::exit(1);
    }
}

