use chrono::{NaiveDate, Datelike, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use terminal_size::{terminal_size, Width};
use crossterm::{cursor::MoveTo, cursor::Hide, ExecutableCommand};
use crossterm::cursor;
use std::io;
use std::io::{stdout, Write};
use std::{thread, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
struct Habit {
    name: String,
    streak: u32,
    history: Vec<String>, // store dates as YYYY-MM-DD
}

fn load_data(path: &str) -> io::Result<Vec<Habit>> {
    if let Ok(contents) = fs::read_to_string(path) {
        let habits: Vec<Habit> = serde_json::from_str(&contents).unwrap_or_default();
        Ok(habits)
    } else {
        Ok(Vec::new())
    }
}
/*
fn save_data(path: &str, habits: &Vec<Habit>) -> io::Result<()> {
    let json = serde_json::to_string_pretty(habits).unwrap();
    fs::write(path, json)
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

fn add_habit(habits: &mut Vec<Habit>, name: &str) {
    habits.push(Habit {
        name: name.to_string(),
        streak: 0,
        history: Vec::new(),
    });

}
*/
fn main() {
    
    let mut stdout = stdout();
    let mut width = 0;
    
    execute!(stdout(), Clear(ClearType::All)).unwrap();

    
    if let Some((Width(w), _)) = terminal_size() {
        width = w;
        for _y in 0..7 {    
            for _x in 0..w/2 {
                print!(" ");
            } print!("\n");
        }
    } else {
       println!("Couldn't get terminal size.");
    }

    let path = "/home/washington/Documents/habit-tracker/habits.json";
    let habits = load_data(path).expect("Failed to load data");
    
    let habit = &habits[0];
    let current_date = Local::now().date_naive();
    let current_week = current_date.iso_week().week();
    let current_weekday = current_date.weekday().number_from_monday();
       
    for i in current_weekday..7 {
        stdout.execute(MoveTo(2*(width/2)-2, i as u16)).unwrap();
        print!("  ");
    }
    
    for day in habit.history.iter().rev() {
        let mut date = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
        let mut week = date.iso_week().week();
        let mut weekday = date.weekday().number_from_monday();

        let mut difference_week = current_week as u16- week as u16;
        
        let mut position_x = 2*(width/2) - 2*difference_week-2;
        let mut position_y = weekday as u16 -1;
        
        
        if position_x >= 0 {
            stdout.execute(MoveTo(position_x, position_y)).unwrap();
            print!(" ");
        } else {
            break;
        }
        
        stdout.flush().unwrap();
    
    }

    stdout.execute(Hide).unwrap();

    while true {
        thread::sleep(Duration::from_secs(10));
    }
    
}

