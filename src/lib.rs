use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::time::Duration;


// for parsing json
#[derive(Debug, Deserialize)]
pub struct Accounts {
    accounts: Vec<Account>
}

#[derive(Debug, Deserialize)]
pub struct Account {
    name: String,
    balance: Mutex<f64>,
}



#[derive(Debug, Deserialize)]
pub struct Record {
    from: String,
    to: String,
    amount: f64,
    fee: f64
}

// for parsing json
#[derive(Debug, Deserialize)]
pub struct Requests {
    requests: Vec<Request>
}

#[derive(Debug, Deserialize)]
pub struct Request {
    records: Vec<Record>    
}

#[derive(Debug)]
pub struct System {
    requests: Vec<Request>,
    accounts: HashMap<String, Arc<Account>>
}

fn transfer(from: &Arc<Account>, to: &Arc<Account>, amount: f64, fee: f64) -> Result<(), String> {
    let wait_duration = Duration::from_millis(100);

    loop {
        let (first, second) = if from.name < to.name { (from, to) } else {(to, from)};
        let try_lock_first = first.balance.try_lock();
        if try_lock_first.is_err() {
            std::thread::sleep(wait_duration);
            continue;
        }
        let try_lock_second = second.balance.try_lock();
        if try_lock_second.is_err() {
            std::thread::sleep(wait_duration);
            continue;
        }

        let (mut from_balance_guard, mut to_balance_guard) = if from.name < to.name { 
            (try_lock_first.unwrap(), try_lock_second.unwrap()) 
        } else { 
            (try_lock_second.unwrap(), try_lock_first.unwrap()) 
        };

        if *from_balance_guard < amount + fee {
            return Err(format!("Insufficient funds, {} needs to transfter (amount + fee: {}) to {}, {}.balance: {}", from.name, 
                amount + fee, to.name, from.name, from_balance_guard));
        }

        *from_balance_guard -= amount + fee;
        *to_balance_guard += amount;
        
        println!("Transfer amount: {} (fee: {}) from {} to {}. {}.balance {}, {}.balance {}.", amount, fee, from.name, to.name, from.name, 
            from_balance_guard, to.name, to_balance_guard);

        return Ok(())
    }
}

impl System {
    pub fn new(accounts: Accounts, requests: Requests) -> Self {
            
        let mut accounts_map = HashMap::new();
        for a in accounts.accounts {
            accounts_map.insert(String::from(&a.name), Arc::new(a));
        }
        
        let request_vec = requests.requests;
        Self {
            requests: request_vec,
            accounts: accounts_map
        }
    }

    pub fn run(&self) {
        self.requests.par_iter().for_each(|request| {
            request.records.iter().for_each(|record| {
                let from_account = self.accounts.get(&record.from).unwrap();
                let to_account = self.accounts.get(&record.to).unwrap();
                transfer(&from_account, &to_account, record.amount, record.fee).unwrap_or_else(|err| {
                    eprintln!("Failed to transfer: {}", err);
                });
            });
        });
    }
}


// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_success() {
        let account_from = Account {
            name: "A".to_string(),
            balance: Mutex::new(1000.0),
        };
        let account_to = Account {
            name: "B".to_string(),
            balance: Mutex::new(0.0),
        };

        let account_from_arc = &Arc::new(account_from);
        let account_to_arc = &Arc::new(account_to);
        let result = transfer(
            account_from_arc,
            account_to_arc,
            100.0,
            10.0,
        );

        assert!(result.is_ok());
        
        let account_from_balance = *account_from_arc.balance.lock().unwrap();
        let account_to_balance = *account_to_arc.balance.lock().unwrap();
        assert_eq!(account_from_balance, 890.0);
        assert_eq!(account_to_balance, 100.0);
    }

    #[test]
    fn test_transfer_insufficient_funds() {
        let account_from = Account {
            name: "Alice".to_string(),
            balance: Mutex::new(100.0),
        };
        let account_to = Account {
            name: "Bob".to_string(),
            balance: Mutex::new(500.0),
        };

        let result = transfer(
            &Arc::new(account_from),
            &Arc::new(account_to),
            200.0,
            10.0,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Insufficient funds, Alice needs to transfter (amount + fee: 210) to Bob, Alice.balance: 100"
        );
    }
}
