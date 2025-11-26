use chrono::{Datelike, Local, NaiveDate};
use crossterm::terminal::{Clear, ClearType};
use serde::{Deserialize, Serialize};
use std::fs;
use clap::{Parser, Subcommand};
use terminal_size::{terminal_size, Width};
use crossterm::{cursor::MoveTo, cursor::Hide, ExecutableCommand};
use std::io;
use std::io::{stdout, Write};
use prettytable::{Table, Row, Cell};
use prettytable::Attr; // for bold, italic, etc.


#[derive(Serialize, Deserialize, Debug)]
struct Habit {
    name: String,
    streak: u32,
    history: Vec<String>, // store dates as YYYY-MM-DD
}


#[derive(Parser)] 
#[command(name = "habit-tracker")]
#[command(about = "A simple habit tracker CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the graph with your habit's history
    Graph {
        name: String,
    },
    /// Mark a habit as done today
    Mark {
        /// Name of the habit
        name: String,
    },
    /// Add a new habit
    Add {
        /// Name of the habit
        name: String,
    },
    /// List all habits
    List,

}

fn load_data(path: &str) -> io::Result<Vec<Habit>> {
    if let Ok(contents) = fs::read_to_string(path) {
        let habits: Vec<Habit> = serde_json::from_str(&contents).unwrap_or_default();
        Ok(habits)
    } else {
        Ok(Vec::new())
    }
}

fn save_data(path: &str, habits: &Vec<Habit>) -> io::Result<()> {
    let json = serde_json::to_string_pretty(habits).unwrap();
    fs::write(path, json)
}

fn check_streak(habits: &mut Vec<Habit>) {
    let today = Local::now().date_naive();
    for habit in habits {
        if let Some(last_entry) = habit.history.last() {
            let last_str: &str = last_entry.as_str(); // convert &String -> &str
            let date = NaiveDate::parse_from_str(last_str, "%Y-%m-%d").unwrap();
            if (today - date).num_days() > 1 {
                habit.streak = 0;
            }
        }   
    }

}
fn mark_habit(habits: &mut Vec<Habit>, name: &str) {
    let today = Local::now().date_naive();

    if let Some(habit) = habits.iter_mut().find(|h| h.name == name) {
        let today_str = today.to_string();
        if !habit.history.contains(&today_str) {
            habit.history.push(today_str);
            habit.streak += 1;
            println!("Habit '{}' marked! Streak: {}", habit.name, habit.streak);
        } else {
            println!("Habit '{}' is already marked today.", habit.name);
        }
    } else {
        println!("Habit not found.");
    }
}

fn add_habit(habits: &mut Vec<Habit>, name: &str) {
    habits.push(Habit {
        name: name.to_string(),
        streak: 0,
        history: Vec::new(),
    });

}

fn print_graph(habit: &Habit) {

    let mut stdout = stdout();
    let width: u16;
    
    let current_date = Local::now().date_naive();
    let current_weekday = current_date.weekday().number_from_monday();

     if let Some((Width(w), _)) = terminal_size() {
       
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(MoveTo(0, 0)).unwrap();
        width = w;
        for _y in 0..7 {    
            for _x in 0..width/2 {
                print!(" ");
            } print!("\n");
        }
        
        
    } else {
       println!("Couldn't get terminal size.");
       std::process::exit(1);
    }

    
    
    // Mark completed days
    for day in habit.history.iter().rev() {
        let date = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
        let weekday = date.weekday().number_from_monday();

        let difference = current_date - date;
        
        // compute using signed arithmetic so we can detect negative positions safely
        let calc_x = 2 * (width as i32 / 2) - 2 * difference.num_weeks() as i32 - 2;
        if calc_x < 0 {
            break;
        }
        
        let position_x = calc_x as u16;
        let position_y = weekday as u16 -1;   
        
        stdout.execute(MoveTo(position_x, position_y)).unwrap();
        print!(" ");
    }
       
    // Remove upcoming days
    for i in current_weekday..8 {
        stdout.execute(MoveTo(2*(width/2)-2, i as u16)).unwrap();
        print!("  ");
    }

    stdout.execute(MoveTo(0, 8)).unwrap();
    stdout.flush().unwrap();
    stdout.execute(Hide).unwrap();
    
}

fn list_habits(path: &str) {
    
    let habits = load_data(&path).expect("Failed to load data");
    let mut table = Table::new();


    table.add_row(Row::new(vec![
        Cell::new("ID").with_style(Attr::Bold),
        Cell::new("Habit").with_style(Attr::Bold),
        Cell::new("Streak").with_style(Attr::Bold),
    ]));
    let mut i = 0;
    for habit in habits {
        i+=1;
        table.add_row(Row::new(vec![
            Cell::new(&i.to_string()),
            Cell::new(&habit.name),
            Cell::new(&habit.streak.to_string()),
        ]));
    }
    table.printstd();


    
}

fn main() {
    
    let cli = Cli::parse();

    // To-Do: add paths relative to executable directory, make the project portable

    let path = "/home/washington/Documents/habit-tracker/habits.json";
    let mut habits = load_data(path).expect("Failed to load data");

    check_streak(&mut habits);
    let _ = save_data(path, &habits);

    match &cli.command {
        Commands::Mark { name } => {
            mark_habit(&mut habits, name);
            let _ = save_data(path, &habits);
        }
        Commands::Add { name } => {
            add_habit(&mut habits, name);
            let _ = save_data(path, &habits);
        }
        Commands::Graph { name } => {
            for habit in habits {
                if habit.name == *name {
                    print_graph(&habit);
                }
            }
        }
        Commands::List => {
            
            list_habits(path);
        }
    }
    

    
    
    
}

