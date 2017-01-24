#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate unborrow;
extern crate regex;
extern crate slug;

use std::path::PathBuf;

mod expand;
mod collapse;

pub use expand::expand;
pub use collapse::collapse;

error_chain! {
    errors {
        IoFailed(action: &'static str, name: PathBuf) {
            description("IO error")
            display("failed to {} \"{}\"", action, name.display())
        }
    }
}

