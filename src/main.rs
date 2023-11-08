use core::fmt;
use std::{thread, time::Duration};

use serde::{Serialize, Deserialize};
use strum::IntoEnumIterator;

fn main() {
    cli();
}

/// Milliseconds since the UNIX epoch
fn get_sys_timestamp() -> u128 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let millis = since_the_epoch.as_millis();
    millis
}

#[derive(Debug)]
enum GameErrors {
    InsufficientFunds,
    MaxLevelReached,
    OutOfBounds,
    AlreadyPlanted,
    AlreadyFarmed,
    NotYetReady,
}

impl fmt::Display for GameErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameErrors::InsufficientFunds => "Insufficient funds",
            GameErrors::MaxLevelReached => "Max level reached",
            GameErrors::OutOfBounds => "Out of bounds",
            GameErrors::AlreadyPlanted => "Already planted",
            GameErrors::AlreadyFarmed => "Already farmed",
            GameErrors::NotYetReady => "Not yet ready",
        };
        write!(f, "{s}")
    }
}

type Result<T> = core::result::Result<T, GameErrors>;

fn seconds_to_millis(seconds: u128) -> u128 {
    seconds * 1000
}

type Money = f64;
type Level = u8;

#[derive(Clone, Copy, Debug, strum::EnumIter, Serialize, Deserialize)]
enum Crop {
    Wheat,
    Potato,
    Carrot,
}

impl Crop {
    fn get_new_field_price(&self) -> Money {
        match self {
            Crop::Wheat => 10.,
            Crop::Potato => 100.,
            Crop::Carrot => 1000.,
        }
    }

    fn get_planting_price(&self) -> Money {
        match self {
            Crop::Wheat => 1.,
            Crop::Potato => 20.,
            Crop::Carrot => 50.,
        }
    }

    fn get_max_level(&self) -> Level {
        match self {
            Crop::Wheat => 5,
            Crop::Potato => 10,
            Crop::Carrot => 20,
        }
    }

    fn level_multiplier(&self) -> f64 {
        match self {
            Crop::Wheat => 0.1,
            Crop::Potato => 0.2,
            Crop::Carrot => 0.4,
        }
    }

    fn grow_time(&self) -> u128 {
        match self {
            Crop::Wheat => seconds_to_millis(10),
            Crop::Potato => seconds_to_millis(20),
            Crop::Carrot => seconds_to_millis(40),
        }
    }

    fn payout(&self) -> Money {
        match self {
            Crop::Wheat => 3.,
            Crop::Potato => 10.,
            Crop::Carrot => 100.,
        }
    }

    fn get_next_level_price(&self, level: Level) -> Money {
        let base_price = self.get_planting_price() * 10.;
        let level_multiplier = self.level_multiplier();
        let price = base_price * (level_multiplier * level as f64);
        price
    }
}

impl fmt::Display for Crop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Crop::Wheat => "Wheat",
            Crop::Potato => "Potato",
            Crop::Carrot => "Carrot",
        };
        write!(f, "{s}")
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Field {
    crop: Crop,
    level: Level,
    plant_timestamp: Option<u128>,
}

impl Field {
    fn new(crop: Crop) -> Field {
        Self {
            crop,
            level: 1,
            plant_timestamp: None,
        }
    }

    fn calculate_price(crop: Crop) -> Money {
        crop.get_new_field_price()
    }

    fn level_up_price(&self) -> Result<Money> {
        if self.level >= self.crop.get_max_level() { return Err(GameErrors::MaxLevelReached) }
        Ok(self.crop.get_next_level_price(self.level))
    }

    fn level_up(&mut self) -> Result<()> {
        if self.level >= self.crop.get_max_level() { return Err(GameErrors::MaxLevelReached) }
        self.level += 1;
        Ok(())
    }

    fn planted(&self) -> bool {
        self.plant_timestamp.is_some()
    }

    fn plant(&mut self, timestamp: u128) -> Result<()> {
        if self.planted() { return Err(GameErrors::AlreadyPlanted) }
        self.plant_timestamp = Some(timestamp);
        Ok(())
    }

    fn time_to_farm(&self, timestamp: u128) -> u128 {
        self.crop.grow_time().checked_sub(timestamp - self.plant_timestamp.unwrap()).unwrap_or(0)
    }

    fn farm(&mut self) -> Result<()> {
        if !self.planted() { return Err(GameErrors::AlreadyFarmed) }
        if self.time_to_farm(get_sys_timestamp()) > 0 { return Err(GameErrors::NotYetReady) }
        self.plant_timestamp = None;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Farm {
    money: f64,
    fields: Vec<Field>,
}

impl Farm {
    fn new() -> Self {
        Self {
            money: 20.,
            fields: Vec::new(),
        }
    }

    fn available_crops() -> Vec<Crop> {
        Crop::iter().collect::<Vec<Crop>>()
    }

    fn buy_field(&mut self, crop: Crop) -> Result<()> {
        let price = crop.get_new_field_price();
        if self.money < price { return Err(GameErrors::InsufficientFunds) }
        self.fields.push(Field::new(crop));
        self.money -= price;
        Ok(())
    }

    fn level_up_field(&mut self, id: u32) -> Result<()> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameErrors::OutOfBounds),
        };

        if field.level_up_price()? > self.money { return Err(GameErrors::InsufficientFunds) }
        let level_up_price = field.level_up_price()?;

        field.level_up()?;
        self.money -= level_up_price;

        Ok(())
    }

    fn plant_field(&mut self, id: u32) -> Result<()> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameErrors::OutOfBounds),
        };

        self.money -= field.crop.get_planting_price();
        field.plant(get_sys_timestamp())?;

        Ok(())
    }

    fn farm_field(&mut self, id: u32) -> Result<Money> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameErrors::OutOfBounds),
        };

        field.farm()?;
        let payout = field.crop.payout() * (1. + field.crop.level_multiplier()*(field.level as f64));
        self.money += payout;
        Ok(payout)
    }

    fn save_to_path(&self, path: String) {
        let json: String = serde_json::to_string(self).unwrap();
        let file = std::fs::File::create(path).unwrap();
        // write all to file
        std::io::Write::write_all(&mut std::io::BufWriter::new(file), json.as_bytes()).unwrap();
    }

    fn load_from_path(&mut self, path: String) {
        let contents = std::fs::read_to_string(path).unwrap();
        let farm: Farm = serde_json::from_str(&contents).unwrap();
        *self = farm;
    }
}

fn cli() {
    let mut farm = Farm::new();
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
                println!("Savin game...");
                farm.save_to_path("save.json".to_string());
                println!("Game saved");
            },
            7 => {
                println!("Loading game...");
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
            format!("{} field, level {}, ready to harvest in {} seconds", f.crop, f.level, f.time_to_farm(get_sys_timestamp())/1000)
        } else {
            format!("{} field, level {}, price to plant ${:.2}", f.crop, f.level, f.crop.get_planting_price())
        }
    ).collect::<Vec<String>>().join("\n  ");
    println!(
"Fields: [
  {}
]
", field_string)
}

fn print_shop() {
let fields_string = Farm::available_crops().iter().enumerate().map(|(i, c)| 
    format!("{}: {} field for {}, harvest price ${:.2}", i+1, c, Field::calculate_price(*c), c.payout())
).collect::<Vec<String>>().join("\n");
println!(
"Pick a field to buy:
{}
", fields_string)
}

fn print_fields(farm: &Farm) {
    let fields_string = farm.fields.iter().enumerate().map(|(i, f)| 
        if f.planted() {
            format!("{}: {} field, level {}, ready to harvest in {} seconds, price to level up ${:.2}", i+1, f.crop, f.level, f.time_to_farm(get_sys_timestamp())/1000, f.level_up_price().unwrap_or(f64::INFINITY))
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
