use trading_system::{System, Accounts, Requests};
use std::{fs, error::Error};

fn read_json() -> Result<(Accounts, Requests), Box<dyn Error>> {
    let account_data = fs::read_to_string("resources/accounts.json")?;

    let a: Accounts = serde_json::from_str(&account_data)?;
    
    let request_data = fs::read_to_string("resources/requests.json")?;

    let r: Requests = serde_json::from_str(&request_data)?;

    Ok((a, r))
}

fn main() {
    let (a, r) = read_json().expect("Parsing json failed");
    let s: System = System::new(a, r);
    s.run();
}
