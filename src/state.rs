use serenity::model::id::UserId;
use serenity::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::collections::{HashSet, HashMap};

const COUNT_TIMEOUT: u64 = 60*60*1000;

pub type StateResult<T> = Result<T,&'static str>;

#[derive(Default)]
#[derive(Serialize, Deserialize)]
pub struct State {
    banned: HashSet<UserId>,
    admins: HashSet<UserId>,
    count: HashMap<UserId, u64>,
    count_cooldown: HashMap<UserId, u64>,
    custom_cmds: HashMap<String, String>,
    #[serde(skip)]
    file: Option<String>,
    #[serde(skip)]
    dirty: bool,
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
    
    pub fn count_up(&mut self, user: UserId) -> u64 {
        use std::time::*;
        use std::convert::TryInto;
        let cooldown = *self.count_cooldown.entry(user).or_insert(0);
        let ctime: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_millis().try_into().unwrap();
        if ctime > (cooldown + COUNT_TIMEOUT) {
            *self.count.entry(user).or_insert(0) += 1;
            self.dirty = true;
            self.count_cooldown.insert(user, ctime);
            0
        } else {
            cooldown + COUNT_TIMEOUT - ctime
        }
    }

    pub fn get_count(&mut self, user: UserId) -> u64 {
        *self.count.entry(user).or_insert(0)
    }

    pub fn get_count_all(&mut self) -> Vec<(UserId, u64)> {
        let mut sorted = self.count.iter()
            .map(|(a,b)| (*a,*b))
            .collect::<Vec<(UserId, u64)>>();
        sorted.sort_by_key(|(_,a)| u64::MAX-*a);
        sorted
    }

    pub fn add_cmd(&mut self, cmd: &str, text: &str) {
        self.custom_cmds.insert(cmd.to_owned(), text.to_owned());
        self.dirty = true;
    }

    pub fn rm_cmd(&mut self, cmd: &str) {
        self.custom_cmds.remove(cmd);
        self.dirty = true;
    }

    pub fn run_custom_cmd(&self, cmd: &str) -> Option<&String> {
        self.custom_cmds.get(cmd)
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
