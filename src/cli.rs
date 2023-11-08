use std::{time::Duration, thread};

use strum::IntoEnumIterator;

use crate::{farm::{Farm, Crop, Field}, util};

pub fn run() {
    let mut farm = Farm::new();
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Welcome to the farm!");
    println!("1: New game");
    println!("2: Load game");
    if input(2) == 1 {
        println!("Starting new game...");
        farm = Farm::new();
        println!("New game started");
    } else {
        println!("Loading game...");
        farm.load_from_path("save.json".to_string());
        println!("Game loaded");
    }
    
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("Welcome to the farm!");
        println!("Balance: ${:.2}", farm.money);
        print_menu();
        match input(7) {
            1 => {
                print_farm(&farm)
            },
            2 => {
                if farm.fields.is_empty() {
                    println!("No fields to plant");
                } else {
                    println!("Pick a field to plant");
                    print_fields(&farm);
                    let id = input(farm.fields.len() as u32) - 1;
                    match farm.plant_field(id) {
                        Ok(_) => println!("Field planted"),
                        Err(e) => println!("{}", e),
                    }
                }
            },
            3 => {
                if farm.fields.is_empty() {
                    println!("No fields to farm");
                } else {
                    println!("Pick a field to farm");
                    print_fields(&farm);
                    let id = input(farm.fields.len() as u32) - 1;
                    match farm.farm_field(id) {
                        Ok(payout) => println!("Field farmed, you received ${payout:.2}"),
                        Err(e) => println!("{}", e),
                    }
                }
            },
            4 => {
                print_shop();
                let id = input(Crop::iter().count() as u32) - 1;
                let crop = Crop::iter().nth(id as usize).unwrap();
                match farm.buy_field(crop) {
                    Ok(_) => println!("Field bought"),
                    Err(e) => println!("{}", e),
                }
            },
            5 => {
                println!("Pick a field to level up");
                print_fields(&farm);
                let id = input(farm.fields.len() as u32) - 1;
                match farm.level_up_field(id) {
                    Ok(_) => println!("Field leveled up"),
                    Err(e) => println!("{}", e),
                }
            },
            6 => {
                println!("Saving game...");
                thread::sleep(Duration::from_secs(2));
                farm.save_to_path("save.json".to_string());
                println!("Game saved");
            },
            7 => {
                println!("Loading game...");
                thread::sleep(Duration::from_secs(2));
                farm.load_from_path("save.json".to_string());
                println!("Game loaded");
            },
            _ => unreachable!(),
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn print_menu() {
    println!(
"
Pick an option:
1: View farm
2: Plant field
3: Farm field
4: Buy new field
5: Level up field
6: Save game
7: Load game
"
    )
}

fn print_farm(farm: &Farm) {
    let field_string = farm.fields.iter().map(|f| 
        if f.planted() {
            format!("{} field, level {}, ready to harvest in {} seconds", f.crop, f.level, f.time_to_farm(util::timestamp())/1000)
        } else {
            format!("{} field, level {}, price to plant ${:.2}, earnings ${:.2} per harvest", f.crop, f.level, f.crop.get_planting_price(), f.earnings())
        }
    ).collect::<Vec<String>>().join("\n  ");
    println!("Fields: [\n  {}\n]", field_string)
}

fn print_shop() {
    let fields_string = Farm::available_crops().iter().enumerate().map(|(i, c)| 
        format!("{}: {} field for {}, harvest price ${:.2}", i+1, c, Field::calculate_price(*c), c.payout())
    ).collect::<Vec<String>>().join("\n");
    println!("Pick a field to buy:\n{}", fields_string)
}

fn print_fields(farm: &Farm) {
    let fields_string = farm.fields.iter().enumerate().map(|(i, f)| 
        if f.planted() {
            format!("{}: {} field, level {}, ready to harvest in {} seconds, price to level up ${:.2}", i+1, f.crop, f.level, f.time_to_farm(util::timestamp())/1000, f.level_up_price().unwrap_or(f64::INFINITY))
        } else {
            format!("{}: {} field, level {}, price to plant ${:.2}, price to level up ${:.2}", i+1, f.crop, f.level, f.crop.get_planting_price(), f.level_up_price().unwrap_or(f64::INFINITY))
        }
    ).collect::<Vec<String>>().join("\n");
    println!("{}", fields_string)
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
        if !(1..=max).contains(&input) {
            println!("Input must be a number in 1 to {max}"); continue
        }
        return input
    }
}
