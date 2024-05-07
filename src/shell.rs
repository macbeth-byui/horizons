use std::io::{self, Write};
use std::collections::HashMap;
use console::Style;
use crate::modules::module::{ModuleInfo, ModuleTrait, ModuleType};
use crate::macros::err;


pub struct Shell {
    modules : HashMap<ModuleType, Box<dyn ModuleTrait>>,
    module_info : ModuleInfo,
    selected : ModuleType
}

impl Shell {

    pub async fn new(modules : HashMap<ModuleType, Box<dyn ModuleTrait>>) -> Result<Self,String> {
        let module_info = ModuleInfo::new()?;
        Ok(Self { modules, module_info, selected : ModuleType::Current } )
    }


    pub async fn run(&mut self) -> Result<(), String> {
        println!();
        loop {
            let module = self.modules.get_mut(&self.selected)
                .ok_or(err!("Unable to find module",""))?;

            print!("{}> ", module.get_name());
            
            let mut buffer = String::new();
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut buffer).unwrap();
            println!();

            let parsed = buffer
                .split_whitespace()
                .collect::<Vec<&str>>();

            if !module.process_cmd(parsed.clone()).await? {
                if let Some(command) = parsed.first() {
                    match *command {
                        "refresh" => {
                            module.refresh().await?;
                            self.module_info.update_refresh(&self.selected)?;
                        }
                        "help" => {
                            module.help();
                            Shell::help();
                        }
                        "exit" => break,
                        "module" => {
                            if let Some(selected) = self.module_cmd(parsed) {
                                self.selected = selected;
                            }
                        }
                        _ => println!("Invalid Command")
                    }
                }
            }
            println!();
        }
        Ok(())
    }

    fn module_cmd(&self, parsed : Vec<&str>) -> Option<ModuleType> {
        if let Some(module) = parsed.get(1) {
            match *module {
                "current" => {
                    println!("Switched to Module Current");
                    Some(ModuleType::Current)
                }
                _ => { println!("Invalid Module"); None }
            };
        }
        else {
            for (module_type, module) in self.modules.iter() {
                let refresh = self.module_info.get_module_refresh(module_type);
                println!("{} - {}", module.get_name(), refresh.unwrap_or("Not Refreshed".to_string()));
            }
            println!();
        }
        None
    }

    fn help() {
        println!("refresh : reload data for current module");
        println!("help : show module specific and general command");
        println!("exit : close the program")
    }

    pub fn logo() {
        let yellow = Style::new().color256(226);
        let red = Style::new().color256(52);
        let orange = Style::new().color256(130);
        let gold = Style::new().color256(220);
        let green = Style::new().color256(22);
        println!();
        println!("    {}{}{}     ",
            red.apply_to("-"), 
            orange.apply_to("\\ \\ \\/ / /"),
            red.apply_to("-")
        );
        println!("   {}{}{}    ",
            red.apply_to("-"),
            orange.apply_to("\\ \\ \\/\\/ / /"),
            red.apply_to("-")
        );
        println!("  {}{} {} {}{}   ",
            red.apply_to("-"),
            orange.apply_to("\\ \\"),
            yellow.apply_to("******"),
            orange.apply_to("/ /"),
            red.apply_to("-")
        );
        println!("   {}{} {} {} {} {}{}    ",
            red.apply_to("-"),
            orange.apply_to("\\"),
            yellow.apply_to("*"),
            gold.apply_to("~~~~"),
            yellow.apply_to("*"),
            orange.apply_to("/"),
            red.apply_to("-")
        );
        println!("  {}{} {} {}{}   ",
            red.apply_to("---"),
            yellow.apply_to("*"),
            gold.apply_to("~~~~~~"),
            yellow.apply_to("*"),
            red.apply_to("---")
        );
        println!("{}{} {} {}{}",
            green.apply_to("===="),
            yellow.apply_to("*"),
            gold.apply_to("~~~~~~~~"),
            yellow.apply_to("*"),
            green.apply_to("===="),
        );
        println!("      Horizons       ");
        println!(); 
    }

}