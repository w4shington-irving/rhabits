use chrono::{Datelike, Days, Duration, Local, NaiveDate};
use crossterm::ExecutableCommand;
use crossterm::cursor::{Hide, MoveTo};
use crossterm::terminal::{Clear, ClearType, enable_raw_mode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use terminal_size::{terminal_size, Width};
use crossterm::event::{self, Event, KeyCode};
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
#[command(name = "habit-tracker")]
#[command(about = "A simple habit tracker CLI", long_about = None)]
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
    /// Remove a habit
    Remove {
        name: String,
    },
    /// Edit a habit's history
    Edit {
        name: String,
    }
}


fn get_habits_path() -> io::Result<PathBuf> {
    
    let proj_dirs = ProjectDirs::from("", "w4shington-irving", "habit-tracker")
        .expect("Failed to get project directories");

    let data_dir = proj_dirs.data_dir();    // ~/.local/share/HabitTracker/
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
    
    let current_date = NaiveDate::parse_from_str("2025-12-1", "%Y-%m-%d").unwrap();
    //let current_date = Local::now().date_naive();
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

fn edit_habit(habits: &mut Vec<Habit>, name: &str) {
    
    if let Some(habit) = habits.iter_mut().find(|h| h.name == name) {
        let mut stdout = stdout();
        let width: u16;
        
        if let Some((Width(w), _)) = terminal_size() {
           width = w;
        } else {
           println!("Couldn't get terminal size.");
           std::process::exit(1);
        }

        let current_date = Local::now().date_naive();
        let current_weekday = current_date.weekday().number_from_monday();

        let mut selected_date = current_date.clone();

        //print_graph(habit);

        // Read arrow keys 
        enable_raw_mode();
        loop {
            if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key_event)) => {
                        match key_event.code {
                            KeyCode::Up => {
                                selected_date -= Duration::days(1);
                            }
                            KeyCode::Down => {
                                let t = selected_date + Duration::days(1);
                                if t>=current_date {
                                    selected_date = t;
                                }
                            }
                            KeyCode::Left => {
                                selected_date -= Duration::days(7);
                            }
                            KeyCode::Right => {
                                let t = selected_date + Duration::days(7);
                                if t>=current_date {
                                    selected_date = t;
                                }
                            }
                            KeyCode::Esc => {
                                println!("ESC pressed, exiting key loop");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Ok(_) => {} // ignore other events
                    Err(_) => break,
                }
            }
            println!("{}",selected_date);
        }
    } else {
        println!("Habit not found.");
    }
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
        Commands::Mark { name } => {
            mark_habit(&mut habits, name);
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Add { name } => {
            add_habit(&mut habits, name);
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Graph { name } => {
            if let Some(habit) = habits.iter_mut().find(|h| h.name == *name) {
                print_graph(&habit);
            }
        }
        Commands::Remove { name } => {
            habits.retain(|h| h.name != *name);
            let _ = save_data(&habits_path, &habits);
        }
        Commands::Edit { name } => {
            edit_habit(&mut habits, name);
        }
        
    }
    
}

/* To-do
- Add edit mode
- Add default habit
- Multiple habits graphing
- Waybar module
 */