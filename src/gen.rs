use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use api::{Game, GbApi};
use failure::Error;
use markov::Chain;
use rand::{self, Rng};
use serde_json;

const DATA_DIR: &str = "data";
const GAMES_FILE: &str = "games.json";
const CHAIN_FILE: &str = "chain.bin";

pub fn fetch_games_if_needed() -> Result<(), Error> {
    if Path::new(DATA_DIR).join(GAMES_FILE).is_file() {
        info!("games.json already exists, returning");
        Ok(())
    } else {
        let api = GbApi::from_env()?;
        let games = api.all_games()?;
        let mut file = BufWriter::new(File::create("data/games.json")?);
        serde_json::to_writer(&mut file, &games)?;

        Ok(())
    }
}

pub fn load_chain() -> Result<Chain<String>, Error> {
    if Path::new(DATA_DIR).join(CHAIN_FILE).is_file() {
        Chain::load("data/chain.bin").map_err(From::from)
    } else {
        let file = File::open("data/games.json")?;
        let games: Vec<Game> = serde_json::from_reader(&file)?;
        let mut chain = Chain::of_order(3);
        for game in games {
            for word in game.name.split(" ") {
                chain.feed_str(&word);
            }
        }

        chain.save("data/chain.bin")?;

        Ok(chain)
    }
}

pub fn generate_names_cli() -> Result<(), Error> {
    fetch_games_if_needed()?;
    let chain = load_chain()?;
    let mut rng = rand::thread_rng();

    loop {
        for _ in 0..15 {
            let len = rng.gen_range(2, 8);
            let name: Vec<String> = chain.str_iter_for(len).collect();
            println!("{}", name.join(" "));
        }
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
    }
}
