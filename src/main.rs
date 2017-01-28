#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;

use std::process;

use error_chain::ChainedError;

extern crate cellsplit;
use cellsplit::actor::Actor;

fn main() {
    let matches = clap_app!(cellsplit =>
        (version: crate_version!())
        (author:  crate_authors!())
        (about:   crate_description!())
        (@setting ColoredHelp)
        (@setting SubcommandRequiredElseHelp)
        (@subcommand expand =>
            (about: "Split a cell mode file into individually executable scripts")
            (@setting ColoredHelp)
            (@arg INPUT: +required "The cell mode file")
            (@arg overwrite: -o --overwrite "Overwrite output files")
        )
        (@subcommand collapse =>
            (about: "Recombine individual scripts into a complete cell mode file")
            (@setting ColoredHelp)
            (@arg OUTPUT: +required "The generated cell mode file")
        )
    ).get_matches();

    let actor = Actor::new();

    if let Some(matches) = matches.subcommand_matches("expand") {
        let infile = matches.value_of("INPUT").unwrap();
        if let Err(err) = cellsplit::expand(actor, infile, matches.is_present("overwrite")) {
            println!("{}", ChainedError::display(&err));
            process::exit(1);
        } else {
            println!("Complete.");
        }
    } else if let Some(matches) = matches.subcommand_matches("collapse") {
        let outfile = matches.value_of("OUTPUT").unwrap();
        if let Err(err) = cellsplit::collapse(actor, outfile) {
            println!("{}", ChainedError::display(&err));
            process::exit(1);
        } else {
            println!("Complete.");
        }
    } else {
        // unreachable because of the SubcommandRequiredElseHelp setting
        unreachable!();
    }
}

