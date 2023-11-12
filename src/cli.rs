use std::{time::Duration, thread};

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use strum::IntoEnumIterator;
use colored::Colorize;

use crate::{farm::{Farm, Crop, Field}, util};

fn print_header(name: Option<&str>) {
    let name = match name {
        Some(name) => format!("{}'s", name),
        None => "your".to_string(),
    };
    println!("{}", format!("Welcome to {} farm!", name).bold().bright_green().underline());
}

fn format_money(money: f64) -> String {
    format!("{}",format!("${:.2}", money).bold().bright_green())
}

pub fn run() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    print_header(None);
    println!("{}: New game", "1".bold());
    println!("{}: Load game", "2".bold());
    let mut farm = if input(2) == 1 {
        println!("Starting new game...");
        println!("Enter your name:");
        let name = loop {
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => (),
                Err(_) => {println!("Unable to read input"); continue},
            };
            break input.trim().to_string()
        };
        let farm = Farm::new(name);
        println!("New game started");
        farm
    } else {
        println!("Loading game...");
        let farm = Farm::load_from_path("save.json".to_string());
        println!("Game loaded");
        farm
    };
    
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        print_header(Some(&farm.name));
        println!("Balance: {}", format_money(farm.money));
        print_menu();
        match input(8) {
            0 => {
                println!("Do you want to save the game?\n{}: Back\n{}: Yes\n{}: No", "0".bold(), "1".bold(), "2".bold());
                let input = input(2);
                if input == 0 { continue }
                if input == 1 {
                    println!("Saving game...");
                    wait();
                    farm.save_to_path("save.json".to_string());
                    println!("Game saved");
                }
                println!("Goodbye!");
                break
            },
            1 => {
                println!("{}", "Your farm (enter any number to go back):".bold().underline());
                print_farm(&farm);
                let _input = input(u32::MAX);
                continue 
            },
            2 => {
                if farm.fields.is_empty() {
                    println!("No fields to plant");
                } else {
                    println!("{}", "Pick a field to plant".bold().underline());
                    print_fields(&farm);
                    let input = input(farm.fields.len() as u32);
                    if input == 0 { continue }
                    let id = input - 1;
                    match farm.plant_field(id) {
                        Ok(_) => println!("Field planted, it will be ready in {}s", format!("{}", farm.fields[id as usize].time_to_farm(util::timestamp())/1000).bold().bright_magenta()),
                        Err(e) => println!("{}", e),
                    }
                }
                wait()
            },
            3 => {
                if farm.fields.is_empty() {
                    println!("No fields to farm");
                } else {
                    println!("{}", "Pick a field to farm".bold().underline());
                    print_fields(&farm);
                    let input = input(farm.fields.len() as u32);
                    if input == 0 { continue }
                    let id = input - 1;
                    match farm.farm_field(id) {
                        Ok(payout) => println!("Field farmed, you received ${payout:.2}"),
                        Err(e) => println!("{}", e),
                    }
                }
                wait()
            },
            4 => {
                print_shop();
                let input = input(Crop::iter().count() as u32);
                if input == 0 { continue }
                let id = input - 1;
                let crop = Crop::iter().nth(id as usize).unwrap();
                match farm.buy_field(crop) {
                    Ok(_) => println!("Field bought"),
                    Err(e) => println!("{}", e),
                }
                wait()
            },
            5 => {
                println!("{}", "Pick a field to level up".bold().underline());
                print_fields(&farm);
                let input = input(farm.fields.len() as u32);
                if input == 0 { continue }
                let id = input - 1;
                match farm.level_up_field(id) {
                    Ok(_) => println!("Field leveled up"),
                    Err(e) => println!("{}", e),
                }
                wait()
            },
            6 => {
                if farm.fields.is_empty() {
                    println!("No fields to sell");
                } else {
                    println!("{}", "Pick a field to sell".bold().underline());
                    print_fields(&farm);
                    let input = input(farm.fields.len() as u32);
                    if input == 0 { continue }
                    let id = input - 1;
                    match farm.sell_field(id) {
                        Ok(price) => println!("Field sold, you received {}", format_money(price)),
                        Err(e) => println!("{}", e),
                    }
                }
                wait()
            }
            7 => {
                println!("Saving game...");
                thread::sleep(Duration::from_secs(2));
                farm.save_to_path("save.json".to_string());
                println!("Game saved");
                wait()
            },
            8 => {
                println!("Loading game...");
                thread::sleep(Duration::from_secs(2));
                farm = Farm::load_from_path("save.json".to_string());
                println!("Game loaded");
                wait()
            },
            _ => unreachable!(),
        }
    }
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn wait() {
    enable_raw_mode().unwrap();
    thread::sleep(Duration::from_secs_f32(1.5));
    disable_raw_mode().unwrap();
}

fn print_menu() {
    println!(
"
Pick an option:
{}: Exit
{}: View farm
{}: Plant field
{}: Harvest field
{}: Buy new field
{}: Level up field
{}: Sell field
{}: Save game
{}: Load game
","0".bold(), "1".bold(), "2".bold(), "3".bold(), "4".bold(), "5".bold(), "6".bold(), "7".bold(), "8".bold()
    )
}

fn print_farm(farm: &Farm) {
    let field_string = farm.fields.iter().map(|f| 
        if f.planted() {
            format!("{} field, level {}, ready to harvest {}, earnings {} per harvest", f.crop, f.level.to_string().red().bold(), 
            {
                let time_to_farm = f.time_to_farm(util::timestamp());
                if time_to_farm > 0 {
                    format!("in {}s", time_to_farm/1000).bold().bright_magenta()
                } else {
                    "NOW".bold().bright_magenta()
                }
            }
            , format_money(f.earnings()))
        } else {
            format!("{} field, level {}, price to plant {}, earnings {} per harvest", f.crop, f.level.to_string().red().bold(), format_money(f.crop.get_planting_price()), format_money(f.earnings()))
        }
    ).collect::<Vec<String>>().join("\n  ");
    println!("Fields: [\n  {}\n]", field_string)
}

fn print_shop() {
    let fields_string = Farm::available_crops().iter().enumerate().map(|(i, c)| 
        format!("{}: {} field for {}, earnings per harvest {}, max level {}", format!("{}", i+1).bold(), c, format_money(Field::calculate_price(*c)), format_money(c.payout()), c.get_max_level().to_string().red().bold())
    ).collect::<Vec<String>>().join("\n");
    println!("{}", "Pick a field to buy:".bold().underline());
    println!("{}{}\n{}", "0".bold(), ": Back", fields_string)
}

fn print_fields(farm: &Farm) {
    let fields_string = farm.fields.iter().enumerate().map(|(i, f)| 
        if f.planted() {
            format!("{}: {} field, level {}, ready to harvest {}, price to level up {}", 
                format!("{}", i+1).bold(), 
                f.crop, f.level.to_string().red().bold(), 
                {
                    let time_to_farm = f.time_to_farm(util::timestamp());
                    if time_to_farm > 0 {
                        format!("in {}s", time_to_farm/1000).bold().bright_magenta()
                    } else {
                        "NOW".bold().bright_magenta()
                    }
                }, 
                format_money(f.level_up_price().unwrap_or(f64::INFINITY))
            )
        } else {
            format!("{}: {} field, level {}, price to plant {}, price to level up {}", format!("{}", i+1).bold(), f.crop, f.level.to_string().red().bold() , format_money(f.crop.get_planting_price()), format_money(f.level_up_price().unwrap_or(f64::INFINITY)))
        }
    ).collect::<Vec<String>>().join("\n");
    println!("{}{}\n{}", "0".bold(), ": Back", fields_string)
}

fn input(max: u32) -> u32 {
    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(_) => {println!("Unable to read input"); continue},
        };

        let input = input.trim().parse();
        let input: u32 = match input {
            Ok(input) => input,
            _ => {println!("Input must be a number"); continue}
        };
        if !(0..=max).contains(&input) {
            println!("Input must be a number in 0 to {max}"); continue
        }
        return input
    }
}
