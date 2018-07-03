extern crate env_logger;
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate markov;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use failure::Error;
use std::env;

mod api;
mod gen;

fn main() -> Result<(), Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    gen::generate_names_cli()?;

    Ok(())
}
