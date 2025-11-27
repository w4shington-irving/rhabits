use chrono::{Datelike, Local, NaiveDate};
use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, MoveTo};
use crossterm::terminal::{Clear, ClearType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use terminal_size::{terminal_size, Width};
use std::io;
use std::io::{stdout, Write};
use prettytable::{Table, Row, Cell};
use prettytable::Attr; // for bold, italic, etc.
use directories_next::ProjectDirs;


#[derive(Serialize, Deserialize, Debug)]
struct Habit {
    name: String,
    streak: u32,
    history: Vec<String>, // store dates as YYYY-MM-DD
}


#[derive(Parser)] 
#[command(
    name = "habit-tracker",
    about = "A simple visual habit tracker",
    override_usage = "habit-tracker <COMMAND> [HABIT] [DATE]"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all habits
    List,
    /// Print the graph with your habit's history
    Graph {
        name: String,
    },
    /// Mark a day (or days) as done, leave empty to mark today
    Mark {
        /// Name of the habit
        name: String,
        dates: Vec<String>,
    },
    /// Unmark marked day (or days), leave empty to unmark today 
    Unmark {
        /// Name of the habit
        name: String,
        dates: Vec<String>,
    },
    /// Add a new habit
    Add {
        /// Name of the habit
        name: String,
    },
    /// Remove a habit
    Remove {
        name: String,
    },
}


fn get_habits_path() -> io::Result<PathBuf> {
    
    let proj_dirs = ProjectDirs::from("", "w4shington-irving", "habit-tracker")
        .expect("Failed to get project directories");

    let data_dir = proj_dirs.data_dir();    // ~/.local/share/habit-tracker/
    let file_path = data_dir.join("habits.json");

    
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }

    
    if !file_path.exists() {
        fs::write(&file_path, "[]")?; // start with empty array
    }

    Ok(file_path)
}

fn load_data(habits_path: &PathBuf) -> io::Result<Vec<Habit>> {
    if let Ok(contents) = fs::read_to_string(habits_path) {
        let habits: Vec<Habit> = serde_json::from_str(&contents).unwrap_or_default();
        Ok(habits)
    } else {
        Ok(Vec::new())
    }
}

fn save_data(habits_path: &PathBuf, habits: &Vec<Habit>) -> io::Result<()> {
    let json = serde_json::to_string_pretty(habits).unwrap();
    fs::write(habits_path, json)
}

fn check_streak(habits: &mut Vec<Habit>) {
    let today = Local::now().date_naive();
    for habit in habits {
        if let Some(last_entry) = habit.history.last() {
            let date = NaiveDate::parse_from_str(&last_entry.as_str(), "%Y-%m-%d").unwrap();
            if (today - date).num_days() > 1 {
                habit.streak = 0;
            } else if date == today {
                habit.streak += 1;
            }
        }   
    }

}

fn mark_habit(habits: &mut Vec<Habit>, name: &str, dates: Vec<String>) {
    
    if let Some(habit) = habits.iter_mut().find(|h| h.name == name) {
        
        if dates.is_empty() {
            println!("Marking today as done!");
            let current_date = Local::now().date_naive();
            habit.history.push(current_date.to_string());
        } else {
            println!("Marking: {:?}", dates);
            habit.history.extend(dates.iter().cloned());
        }

        habit.history.sort();
    } else {
        println!("Habit not found.");
    }
}

fn unmark_habit(habits: &mut Vec<Habit>, name: &str, dates: Vec<String>) {
    
    if let Some(habit) = habits.iter_mut().find(|h| h.name == name) {
        
        if dates.is_empty() {
            println!("Unmarking today");
            let current_date_string = Local::now().date_naive().to_string();
            habit.history.retain(|x| x != &current_date_string);
        } else {
            println!("Unmarking: {:?}", dates);
            habit.history.retain(|x| !dates.contains(x));
        }
        
        habit.history.sort();
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

        let difference = current_date-date;
        let calc_x = 2 * (width as i32 / 2) - 2*((difference.num_days() as i32+weekday as i32-1)/7+1);
        
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

fn list_habits(habits: Vec<Habit>) {
    // Create the table
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Habit").with_style(Attr::Bold),
        Cell::new("Streak").with_style(Attr::Bold),
        Cell::new("Last Entry").with_style(Attr::Bold),
    ]));

    for habit in habits {
        table.add_row(Row::new(vec![
            Cell::new(&habit.name),
            Cell::new(&habit.streak.to_string()),
            Cell::new(habit.history.last().map(|s| s.as_str()).unwrap_or("")),
        ]));
    }
    table.printstd();


    
}


fn main() {
    
    let cli = Cli::parse();

    let habits_path = get_habits_path().unwrap();
    let mut habits = load_data(&habits_path).expect("Failed to load data");

    check_streak(&mut habits);

    let _ = save_data(&habits_path, &habits);

    
    match &cli.command {
        Commands::List => {
            
            list_habits(habits);
        }
        Commands::Graph { name } => {
            if let Some(habit) = habits.iter_mut().find(|h| h.name == *name) {
                print_graph(&habit);
            }
        }
        Commands::Mark { name, dates} => {
            mark_habit(&mut habits, name, dates.to_vec());
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Unmark { name, dates} => {
            unmark_habit(&mut habits, name, dates.to_vec());
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Add { name } => {
            add_habit(&mut habits, name);
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Remove { name } => {
            habits.retain(|h| h.name != *name);
            let _ = save_data(&habits_path, &habits);
        }
        
        
    }
    
}

/* To-do
- Add failsafe for malformed dates
- Add default habit
- Multiple habits graphing
- Waybar module
 */