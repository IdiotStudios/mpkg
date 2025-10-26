use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, sync::{Arc, Mutex}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String, // store hashed in production
}

#[derive(Clone)]
pub struct Accounts {
    pub users: Arc<Mutex<HashMap<String, User>>>,
    pub accounts_file: String,
}

impl Accounts {
    pub fn new(accounts_file: &str) -> Self {
        let users = Self::load_accounts(accounts_file);
        Self {
            users: Arc::new(Mutex::new(users)),
            accounts_file: accounts_file.to_string(),
        }
    }

    fn load_accounts(accounts_file: &str) -> HashMap<String, User> {
        if let Ok(data) = fs::read_to_string(accounts_file) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    pub fn save_accounts(&self) -> Result<(), std::io::Error> {
        let users = self.users.lock().unwrap();
        let data = serde_json::to_string_pretty(&*users)?;
        fs::write(&self.accounts_file, data)?;
        Ok(())
    }

    pub fn register_user(&self, username: String, password: String) -> Result<(), String> {
        let mut users = self.users.lock().unwrap();
        if users.contains_key(&username) {
            return Err("Username already exists".to_string());
        }
        users.insert(username.clone(), User { username, password });
        self.save_accounts().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        let users = self.users.lock().unwrap();
        if let Some(user) = users.get(username) {
            user.password == password
        } else {
            false
        }
    }
}
