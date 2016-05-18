use toml::{self, Table};
use std::fs::File;
use std::io::{Read};

lazy_static! {
    static ref GLOBAL_CONFIG: Table = get_config();
}

pub fn get_config() -> Table {
    let mut input = String::new();
    File::open("./conf/config.toml").and_then(|mut f| {
        f.read_to_string(&mut input)
    }).unwrap();
    toml::Parser::new(&input).parse().unwrap()
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_config_num<'a>(section: &'a str, attr_name: &'a str) -> i64 {
        let config = &GLOBAL_CONFIG;
        let sec = config.get(section).unwrap();

        let num = sec.lookup(&attr_name).unwrap().as_integer().unwrap();
        num as i64
    }

    pub fn get_config_str<'a>(section: &'a str, attr_name: &'a str) -> String {
        let config = &GLOBAL_CONFIG;
        let sec = config.get(section).unwrap();
        
        let astr = sec.lookup(&attr_name).unwrap().as_str().unwrap();
        astr.to_owned()
    }

}

