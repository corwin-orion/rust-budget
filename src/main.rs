use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs, io};

// Define a struct to represent a financial transaction
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    amount: f64, // The transaction amount (positive for credit, negative for debit)
    date: NaiveDate, // The date the transaction occurs
    recurrence: Option<(String, usize)>, // Optional recurrence (type and number of occurrences)
}

// Define a struct to represent the budget state
#[derive(Debug, Serialize, Deserialize)]
struct BudgetState {
    balance: f64, // Current account balance
    transactions: Vec<Transaction>, // List of transactions
}

impl BudgetState {
    // Create a new budget state with a given initial balance
    fn new(balance: f64) -> Self {
        Self {
            balance,
            transactions: Vec::new(),
        }
    }

    // Add a new transaction to the budget state
    fn add_transaction(&mut self, amount: f64, date: NaiveDate, recurrence: Option<(String, usize)>) {
        self.transactions.push(Transaction { amount, date, recurrence });
    }

    // Forecasts the budget balance over time
    fn forecast(&self) {
        let mut balance = self.balance;
        let mut events: VecDeque<Transaction> = VecDeque::new(); // Queue for upcoming transactions
        let mut month_balances = vec![]; // Stores monthly balances
        let mut current_date = Local::now().date_naive(); // Get the current date
        let mut zero_hit = false; // Flag to track if balance reaches zero

        // Populate the events queue with transactions and their recurrences
        for t in &self.transactions {
            events.push_back((*t).clone());
            if let Some((ref period, count)) = t.recurrence {
                let mut date = t.date;
                for _ in 0..count {
                    date = match period.as_str() {
                        "weekly" => date + Duration::weeks(1),
                        "biweekly" => date + Duration::weeks(2),
                        "monthly" => date + Duration::days(30),
                        _ => break,
                    };
                    events.push_back(Transaction { amount: t.amount, date, recurrence: None });
                }
            }
        }

        // Sort transactions by date
        events.make_contiguous().sort_by_key(|t| t.date);

        // Forecast up to 12 months or until balance reaches zero
        for _ in 0..12 {
            let next_month = current_date.with_day(1).unwrap() + Duration::days(32);
            current_date = next_month.with_day(1).unwrap();
            
            while let Some(t) = events.front() {
                if t.date >= current_date { break; }
                balance += t.amount;
                events.pop_front();
            }
            
            month_balances.push((current_date, balance));
            
            if balance <= 0.0 {
                zero_hit = true;
                break;
            }
        }

        // Print the forecasted balances
        for (date, bal) in month_balances {
            println!("{}: {:.2}", date.format("%Y-%m"), bal);
        }
        
        if zero_hit {
            println!("Balance reaches zero/negative within the displayed period.");
        }
    }

    // Save budget state to a file
    fn save_to_file(&self, filename: &str) {
        let data = serde_json::to_string(self).expect("Failed to serialize");
        fs::write(filename, data).expect("Failed to write file");
    }

    // Load budget state from a file (or create a new one if the file doesn't exist)
    fn load_from_file(filename: &str) -> Self {
        if let Ok(data) = fs::read_to_string(filename) {
            if let Ok(state) = serde_json::from_str(&data) {
                return state;
            }
        }
        Self::new(0.0)
    }
}

fn main() {
    let filename = "budget_state.json";
    let mut budget = BudgetState::load_from_file(filename);
    
    loop {
        // Display menu options
        println!("1. Add transaction\n2. View forecast\n3. Exit");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        
        match choice.trim() {
            "1" => {
                // Get transaction details from the user
                println!("Enter amount (positive for credit, negative for debit): ");
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).expect("Failed to read input");
                let amount: f64 = amount.trim().parse().expect("Invalid amount");
                
                println!("Enter date (YYYY-MM-DD): ");
                let mut date = String::new();
                io::stdin().read_line(&mut date).expect("Failed to read input");
                let date = NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").expect("Invalid date format");
                
                println!("Is this a recurring transaction? (yes/no)");
                let mut recur = String::new();
                io::stdin().read_line(&mut recur).expect("Failed to read input");
                let recurrence = if recur.trim().eq_ignore_ascii_case("yes") {
                    println!("Enter recurrence type (weekly/biweekly/monthly): ");
                    let mut period = String::new();
                    io::stdin().read_line(&mut period).expect("Failed to read input");
                    
                    println!("Enter number of occurrences: ");
                    let mut count = String::new();
                    io::stdin().read_line(&mut count).expect("Failed to read input");
                    let count: usize = count.trim().parse().expect("Invalid number");
                    Some((period.trim().to_string(), count))
                } else {
                    None
                };
                
                // Add transaction to budget and save state
                budget.add_transaction(amount, date, recurrence);
                budget.save_to_file(filename);
            }
            "2" => budget.forecast(), // Display forecast
            "3" => {
                budget.save_to_file(filename); // Save and exit
                break;
            }
            _ => println!("Invalid option, try again."),
        }
    }
}
