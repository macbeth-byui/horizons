use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::macros::err;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub general : GeneralConfig,
    pub current_config : CurrentConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub server : String,
    pub token : String,
    pub postgres : String,
    pub postgres_pool : i32
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurrentConfig {
    pub exclude_zero_grades : bool,
    pub courses : Vec<(String, i32)>,
}

impl Config {

    pub fn load_config() -> Result<Self,String> {
        let file = File::open("config.toml")
            .map_err(|e| err!("TOML File Failure",e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::<u8>::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| err!("TOML File Read Failure",e))?;
        let contents = String::from_utf8(buffer)
            .map_err(|e| err!("TOML Parsing Failure", e))?;
        toml::from_str(&contents)
            .map_err(|e| err!("TOML Parsing Failure",e))

        // TODO: Validate Config
    }
}
