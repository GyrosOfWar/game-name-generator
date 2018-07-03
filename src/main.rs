extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate itertools;
extern crate markov;
extern crate rayon;
extern crate serde_json;
extern crate rand;

use markov::Chain;
use failure::Error;
use reqwest::Url;
use std::env::{self, VarError};
use std::io::BufWriter;
use std::fs::File;

const API_URL: &'static str = "https://www.giantbomb.com/api";

#[derive(Serialize, Deserialize, Debug)]
pub struct GbResponse<T> {
    pub error: String,
    pub limit: usize,
    pub offset: usize,
    pub number_of_page_results: usize,
    pub number_of_total_results: usize,
    pub status_code: usize,
    pub version: Option<String>,
    pub results: Vec<T>,
}

impl<T> GbResponse<T> {
    pub fn has_error(&self) -> bool {
        self.status_code != 1
    }

    pub fn get_error(&self) -> Option<&str> {
        match self.error.as_ref() {
            "OK" => None,
            e => Some(e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub name: String,
    pub aliases: Option<String>,
}

impl Game {
    pub fn alias_list(&self) -> Vec<&str> {
        self.aliases
            .as_ref()
            .map(|a| a.split('\n').collect())
            .unwrap_or(vec![])
    }
}

pub struct GbApi {
    api_key: String,
}

impl GbApi {
    pub fn new(api_key: String) -> Self {
        GbApi { api_key }
    }

    pub fn from_env() -> Result<Self, VarError> {
        let api_key = env::var("GB_API_KEY")?;
        Ok(GbApi { api_key })
    }

    pub fn games(&self, offset: usize, limit: usize) -> Result<GbResponse<Game>, Error> {
        let mut url = Url::parse(API_URL).unwrap();
        url.set_path("api/games/");
        url.query_pairs_mut().append_pair("api_key", &self.api_key);
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());
        url.query_pairs_mut()
            .append_pair("offset", &offset.to_string());
        url.query_pairs_mut()
            .append_pair("field_list", "name,aliases");
        url.query_pairs_mut().append_pair("format", "json");
        info!("Making request to URL {}", url);
        let mut response = reqwest::get(url)?;
        response.json().map_err(From::from)
    }

    pub fn all_games(&self) -> Result<Vec<Game>, Error> {
        use itertools::Itertools;
        use std::thread;
        use std::time::Duration;

        let num_games = self.games(0, 1)?.number_of_total_results;
        info!("found {} games", num_games);
        let offsets: Vec<_> = (0..num_games).step(100).collect();
        Ok(offsets
            .into_iter()
            .filter_map(|offset| {
                thread::sleep(Duration::from_millis(1050));
                match self.games(offset, 100) {
                    Ok(resp) => Some(resp.results),
                    Err(e) => {
                        warn!("Failed to fetch: {}", e);
                        None
                    }
                }
            })
            .flat_map(|e| e)
            .collect())
    }
}

#[allow(unused)]
fn fetch_games() -> Result<(), Error> {
    let api = GbApi::from_env()?;
    let games = api.all_games()?;
    let mut file = BufWriter::new(File::create("data/games.json")?);
    serde_json::to_writer(&mut file, &games)?;

    Ok(())
}

#[allow(unused)]
fn save_chain() -> Result<Chain<String>, Error> {
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

#[allow(unused)]
fn load_chain() -> Result<Chain<String>, Error> {
    Chain::load("data/chain.bin").map_err(From::from)
}

fn generate_names() -> Result<(), Error> {
    use std::io;
    use rand::Rng;

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

fn main() -> Result<(), Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    generate_names()?;

    Ok(())
}
