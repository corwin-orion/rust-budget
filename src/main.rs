use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs, io};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Transaction {
    amount: f64,
    date: NaiveDate,
    recurrence: Option<(String, usize)>, // (weekly, biweekly, monthly), occurrences
    note: String, // Additional field to store notes for transactions
}

#[derive(Debug, Serialize, Deserialize)]
struct BudgetState {
    balance: f64,
    transactions: Vec<Transaction>,
}

impl BudgetState {
    fn new(balance: f64) -> Self {
        Self {
            balance,
            transactions: Vec::new(),
        }
    }

    fn add_transaction(&mut self, amount: f64, date: NaiveDate, recurrence: Option<(String, usize)>, note: String) {
        self.transactions.push(Transaction { amount, date, recurrence, note });
    }

    fn delete_transaction(&mut self, index: usize) {
        if index < self.transactions.len() {
            self.transactions.remove(index);
            println!("Transaction deleted successfully.");
        } else {
            println!("Invalid index. No transaction deleted.");
        }
    }

    fn list_transactions(&self) {
        println!("{:<5}  {:<10}  {:<10}  {:<13}  {}", "Index", "Date", "Amount", "Recurrence", "Note");
        println!("{}", "-".repeat(60));
        for (i, t) in self.transactions.iter().enumerate() {
            let recurrence = match &t.recurrence {
                Some((period, count)) => format!("{} ({})", period, count),
                None => "One-time".to_string(),
            };
            println!("{:<5}  {:<10}  {:<10}  {:<13}  {}", i, t.date.to_string(), t.amount, recurrence, t.note);
        }
    }

    fn forecast(&self) {
        let mut balance = self.balance;
        let mut events: VecDeque<Transaction> = VecDeque::new();
        let mut month_balances = vec![];
        let mut current_date = Local::now().date_naive();
        // let mut zero_hit = false;

        // Populate transaction queue
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
                    events.push_back(Transaction { amount: t.amount, date, recurrence: None, note: t.note.clone() });
                }
            }
        }

        events.make_contiguous().sort_by_key(|t| t.date);

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
                // zero_hit = true;
                // break;
            }
        }

        for (date, bal) in month_balances {
            println!("{}: {:.2}", date.format("%Y-%m"), bal);
        }
        
        // if zero_hit {
        //     println!("Balance reaches zero/negative within the displayed period.");
        // }
    }

    fn save_to_file(&self, filename: &str) {
        let data = serde_json::to_string(self).expect("Failed to serialize");
        fs::write(filename, data).expect("Failed to write file");
    }

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
        println!("1. Add transaction\n2. View transactions\n3. Delete transaction\n4. View forecast\n5. Exit");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        
        match choice.trim() {
            "1" => {
                println!("Enter amount (positive for credit, negative for debit): ");
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).expect("Failed to read input");
                let amount: f64 = amount.trim().parse().expect("Invalid amount");
                
                println!("Enter date (YYYY-MM-DD): ");
                let mut date = String::new();
                io::stdin().read_line(&mut date).expect("Failed to read input");
                let date = NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").expect("Invalid date format");
                
                println!("Enter a note for this transaction: ");
                let mut note = String::new();
                io::stdin().read_line(&mut note).expect("Failed to read input");
                
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
                
                budget.add_transaction(amount, date, recurrence, note.trim().to_string());
                budget.save_to_file(filename);
            }
            "2" => budget.list_transactions(),
            "3" => {
                println!("Enter transaction index to delete: ");
                let mut index = String::new();
                io::stdin().read_line(&mut index).expect("Failed to read input");
                if let Ok(idx) = index.trim().parse::<usize>() {
                    budget.delete_transaction(idx);
                    budget.save_to_file(filename);
                }
            }
            "4" => budget.forecast(),
            "5" => break,
            _ => println!("Invalid option, try again."),
        }
    }
}
