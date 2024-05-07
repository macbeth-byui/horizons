use serde::de::DeserializeOwned;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use regex::Regex;
use crate::data::config::Config;
use crate::macros::err;

pub async fn json_api_get<T>(config : &Config, rel_url : &str) -> Result<Vec<T>, String> 
    where T : DeserializeOwned
{
    // println!("======================\n{}",rel_url);
    let client = reqwest::Client::new();
    let mut all_results = Vec::<T>::new();
    let mut url = format!("{}/{}&per_page=100",config.general.server, rel_url);
    loop {

        // println!("{}",url);
        let res = client.get(url.clone())
            .header("Authorization", format!("Bearer {}", config.general.token))
            .send()
            .await
            .map_err(|e| err!(format!("API Request Failure\n{}",url),e))?;
        
        let headers = res.headers().clone();
        // Note that text() consumes the response
        let data = res.text()
            .await
            .map_err(|e| err!(format!("API Response Parsing Failure\n{}",url),e))?;
        // println!("{}", data);
        // println!("{:?}", headers);
        let results: Vec<T> = serde_json::from_str(&data)
        .map_err(|e| err!(format!("API JSON Conversion Failure\n{}",url),e))?;
        all_results.extend(results);

        match headers.get("link") {
            Some(value) => {
                let links = value.to_str()
                    .map_err(|e| err!(format!("API Header Parsing Failure\n{}",url),e))?;
                // println!("Links: {}", links);
                let re = Regex::new("<([^<]+)>; rel=\"next\",").unwrap();
                let Some(next) = re.captures(links) else {
                    // println!("No match.");
                    break;
                };
                // println!("Match: {}", &next[1]);
                url = next[1].to_string();
            },
            None => break
        }

    }
    Ok(all_results)
}

pub async fn json_api_get_single<T>(config : &Config, rel_url : &str) -> Result<T, String> 
    where T : DeserializeOwned
{
    // println!("======================\n{}",rel_url);
    let client = reqwest::Client::new();
    let url = format!("{}/{}&per_page=200",config.general.server, rel_url);
    let res = client.get(url.clone())
        .header("Authorization", format!("Bearer {}", config.general.token))
        .send()
        .await
        .map_err(|e| err!(format!("API Request Failure\n{}",url),e))?;
        
    let data = res.text()
        .await
        .map_err(|e| err!(format!("API Response Parsing Failure\n{}",url),e))?;
    // println!("{}", data);
    let result: T = serde_json::from_str(&data)
    .map_err(|e| err!(format!("API JSON Conversion Failure\n{}",url),e))?;
    Ok(result)
}

pub async fn connect_database(config : &Config) -> Result<Pool<Postgres>, String> {
    PgPoolOptions::new()
                .max_connections(config.general.postgres_pool as u32)
                .connect(&config.general.postgres)
                .await 
                .map_err(|e| err!("Postgress DB Connect Failure",e))
}

