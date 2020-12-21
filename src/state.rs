use serenity::model::id::UserId;
use serenity::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::collections::HashSet;

pub type StateResult<T> = Result<T,&'static str>;

#[derive(Default)]
#[derive(Serialize, Deserialize)]
pub struct State {
    banned: HashSet<UserId>,
    admins: HashSet<UserId>,
    #[serde(skip)]
    file: Option<String>,
    #[serde(skip)]
    dirty: bool
}

impl TypeMapKey for State { 
    type Value = Self;
}

impl State {
    pub fn load_from_file(name: &str) -> StateResult<Self> {
        let file = match File::open(name) {
            Ok(x) => x,
            Err(_) => return Err("Could not open file")
        };
        match serde_json::from_reader::<File, Self>(file) {
            Ok(mut x) => {
                x.file = Some(name.to_owned());
                Ok(x)
            }
            Err(_) => return Err("Could not parse file")
        }
    }

    pub fn ban(&mut self, user: UserId) -> StateResult<()> {
        if self.admins.contains(&user) {
            return Err("Cannot ban an admin")
        } else if self.banned.contains(&user) {
            return Err("User is already banned")
        } else {
            self.banned.insert(user);
            self.dirty = true;
            Ok(())
        }
    }

    pub fn unban(&mut self, user: UserId) -> StateResult<()> {
        if !self.banned.contains(&user) {
            return Err("User is not banned")
        } else {
            self.banned.remove(&user);
            self.dirty = true;
            Ok(())
        }
    }

    pub fn is_admin(&self, user: UserId) -> bool {
        self.admins.contains(&user)
    }

    pub fn is_banned(&self, user: UserId) -> bool {
        self.banned.contains(&user)
    }

    pub fn force_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn save_if_dirty(&mut self) -> Result<bool,String> {
        if !self.dirty {
            return Ok(false)
        }
        if self.file.is_none() {
            return Err(String::from("No file set"))
        }
        use std::fs::OpenOptions;
        let file = match OpenOptions::new()
            .write(true).truncate(true)
            .open(&(self.file.as_ref().unwrap())) {
            Ok(x) => x,
            Err(e) => return Err(format!("{:?}", e))
        };
        match serde_json::to_writer(file, &self) {
            Ok(_) => (),
            Err(e) => return Err(format!("{:?}", e))
        }
        self.dirty = false;
        Ok(true)
    }
}
