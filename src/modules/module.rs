use std::fmt;
use std::io::{self,Write};
use serde::{Serialize,Deserialize};
use std::fs::File;
use std::io::{BufReader, Read, BufWriter};
use console::Term;
use async_trait::async_trait;
use chrono::prelude::{DateTime, Local};
use crate::macros::err;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleType {
    Current,
}

#[derive(Deserialize, Serialize)]
pub struct ModuleInfo {
    pub current_mod_refresh : Option<String>
}

impl ModuleInfo {

    pub fn new() -> Result<Self, String> {
        match ModuleInfo::load_module_info() {
            Ok(module_info) => Ok(module_info),
            Err(_) => {
                let module_info = ModuleInfo { 
                    current_mod_refresh: None
                };
                module_info.save_module_info()?;
                Ok(module_info)
            } 
        }
        
    }

    fn load_module_info() -> Result<Self, String> {
        let file = File::open("refresh.toml")
            .map_err(|e| err!("TOML File Failure",e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::<u8>::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| err!("TOML File Read Failure",e))?;
        let contents = String::from_utf8(buffer)
            .map_err(|e| err!("TOML Parsing Failure", e))?;
        toml::from_str(&contents)
            .map_err(|e| err!("TOML Parsing Failure",e))
    }

    pub fn update_refresh(&mut self, module : &ModuleType) -> Result<(),String> {
        let dt = Local::now().format("%d/%m/%Y %H:%M").to_string();        
        match module {
            ModuleType::Current => self.current_mod_refresh = Some(dt),
        };
        self.save_module_info()
    }

    fn save_module_info(&self) -> Result<(),String> {
        let file = File::create("refresh.toml")
            .map_err(|e| err!("TOML File Failure",e))?;
        let mut writer = BufWriter::new(file);
        let buffer = toml::to_string_pretty(self)
            .map_err(|e| err!("TOML Parsing Failure",e))?;
        writer.write_all(buffer.as_bytes())
            .map_err(|e| err!("TOML File Read Failure",e))?;
        Ok(())
    }

    pub fn get_module_refresh(&self, module : &ModuleType) -> Option<String> {
        match module {
            ModuleType::Current => self.current_mod_refresh.clone()
        }
    }
    
}

#[async_trait]
pub trait ModuleTrait {
    fn get_name(&self) -> String;
    async fn process_cmd(&mut self, cmd : Vec<&str>) -> Result<bool,String>;
    async fn refresh(&mut self) -> Result<(),String>;
    fn help(&self);
}


pub fn print_vec<T>(vec : &Vec<T>) where T : fmt::Display {
    for item in vec {
        println!("{}", item);
    }
}

pub fn print_progress_bar(mut curr : u32, total : u32) {
    if curr > total {
        curr = total;
    }
    let mut progress = 0;
    if total != 0 {
       progress = (curr as f32 * (10.0 / total as f32)) as i32;
    }
    let term = Term::stdout();
    let _ = term.clear_line();
    print!("[");
    for _ in 0..progress {
        print!("*");
    }
    for _ in progress..10 {
        print!(" ");
    }
    let mut pct = 0;
    if total != 0 {
        pct = ((curr as f32 / total as f32) * 100.0) as u32;
    }
    print!("] {}%", pct);
    if progress == 10 {
        print!(" Completed.");
    }
    let _ = io::stdout().flush();
}
