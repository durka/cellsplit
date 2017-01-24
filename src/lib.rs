#[macro_use] extern crate error_chain;
#[macro_use] extern crate unborrow;
extern crate slug;

use std::io;

mod expand;
mod collapse;

pub use expand::expand;
pub use collapse::collapse;

error_chain! {
    foreign_links {
        Io(io::Error);
    }
}

