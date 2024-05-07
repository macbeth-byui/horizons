use std::collections::HashMap;
use crate::data::config::Config;
use crate::shell::Shell;
use crate::modules::module::{ModuleTrait, ModuleType};
use crate::modules::current_mod::CurrentMod;
use crate::data::connections::connect_database;


pub async fn run() {
    let config = match Config::load_config() {
        Ok(config) => config,
        Err(e) => { println!("{}",e); return; }
    };

    Shell::logo();

    // When you clone the pool, it creates another reference to a shared resource.
    let database = match connect_database(&config).await {
        Ok(database) => database,
        Err(e) => { println!("{}",e); return; }
    };

    let mut modules = HashMap::<ModuleType, Box<dyn ModuleTrait>>::new();
    modules.insert(ModuleType::Current, Box::new(CurrentMod::new(config.clone(), database.clone())));
    
    let mut shell = match Shell::new(modules).await {
        Ok(shell) => shell,
        Err(e) => { println!("{}",e); return; }
    };

    if let Err(e) = shell.run().await {
        println!("{}",e);
    }
}
