use std::{thread, time::Duration, io::Write, sync::Arc};

use cli_farm::{farm::{Farm, Field}, util};
use colored::Colorize;
use crossterm::{terminal, cursor, style};

fn main() {
    //cli_farm::cli::run();
    let mut cli = CLI::new();
    cli.run();
}

#[derive(Clone, Copy, Debug)]
enum State {
    MainMenu,
    ShopMenu,
    FarmView,
    PlantMenu,
    HarvestMenu,
    LevelUpMenu,
    SellMenu,
}

struct CLI {
    state: Box<State>,
    farm: Box<Farm>,
}

impl CLI {

    fn get_state_pointer(&self) -> Box<State> {
        unsafe {
            let pointer = &*self.state as *const State;
            Box::from_raw(pointer as *mut State)
        }
    }

    fn set_state(&mut self, state: State) {
        unsafe {
            let pointer = &mut *self.state as *mut State;
            (*pointer) = state;
        }
        println!("State set to {:?}", state);
    }

    fn new() -> Self {
        Self {
            state: Box::new(State::MainMenu),
            farm: Box::new(Farm::new("".to_string())),
        }
    }

    fn run(&mut self) {
        let farm_pointer = unsafe {
            let pointer = &mut *self.farm as *mut Farm;
            Box::from_raw(pointer)
        };
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        let state_pointer = self.get_state_pointer();
        thread::spawn(move || {
            let state_pointer = state_pointer;
            let farm = farm_pointer;
            loop {
                Self::print_header(state_pointer.clone(), &farm);
                //thread::sleep(Duration::from_secs(1));
            }
        });

        loop {
            self.main_menu();
        }
    }

    fn main_menu(&mut self) {
        match input() {
            // Exit game
            0 => std::process::exit(0),
            // Plant field
            1 => self.plant_menu(),
            // Harvest field
            2 => self.harvest_menu(),
            3 => self.buy_menu(),
            4 => self.level_up_menu(),
            5 => self.sell_menu(),
            6 => {
                self.farm.save_to_path("save.json".to_string());
            },
            7 => {
                let raw_pointer: *mut Farm = &mut *self.farm;
                let new_farm = Farm::load_from_path("save.json".to_string());
                unsafe {
                    (*raw_pointer) = new_farm;
                }
            },
            _ => println!("Invalid input"),
        }
        self.set_state(State::MainMenu);
    }

    fn plant_menu(&mut self) {
        self.set_state(State::PlantMenu);
        loop {
            let input = input();
            if input == 0 { break }
            match self.farm.plant_field(input) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }
    }

    fn harvest_menu(&mut self) {
        self.set_state(State::HarvestMenu);
        loop {
            println!("Pick a field to harvest (0 to go back):");
            let input = input();
            if input == 0 { break }
            match self.farm.farm_field(input) {
                Ok(payout) => {
                    println!("You earned {}", format_money(payout));
                },
                Err(e) => println!("{}", e),
            }
        }
    }

    fn buy_menu(&mut self) {
        self.set_state(State::ShopMenu);
        loop {
            let crops = Farm::available_crops();
            let input = input();
            if input == 0 { break }
            let crop = match crops.get(input as usize - 1) {
                Some(crop) => crop,
                None => {println!("Invalid input"); continue},
            };
            match self.farm.buy_field(*crop) {
                Ok(_) => println!("You bought a {} field", crop),
                Err(e) => println!("{}", e),
            }
        }
    }

    fn level_up_menu(&mut self) {
        self.set_state(State::LevelUpMenu);
        loop {
            let input = input();
            if input == 0 { break }
            match self.farm.level_up_field(input) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }
    }

    fn sell_menu(&mut self) {
        self.set_state(State::SellMenu);
        loop {
            let input = input();
            if input == 0 { break }
            match self.farm.sell_field(input) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }
    }

    fn print_header(state: Box<State>, farm: &Farm) {
        let welcome = format!("Welcome to {}'s farm!", farm.name).bold().underline().bright_green();
        let balance = format!("Balance: {}", format_money(farm.money)).bold().bright_green();
        let pick = format!("Pick an option:");
        let options = format!(
    "
Pick an option:
{}: Exit
{}: Plant field
{}: Harvest field
{}: Buy new field
{}: Level up field
{}: Sell field
{}: Save game
{}: Load game
    ","0".bold(), "1".bold(), "2".bold(), "3".bold(), "4".bold(), "5".bold(), "6".bold(), "7".bold()
    );
    
        let field_string = farm.fields.iter().enumerate().map(|(i, f)| 
            if f.planted() {
                format!("{}: {} field, level {}, ready to harvest {}, earnings {} per harvest", i+1, f.crop, f.level.to_string().red().bold(), 
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
                format!("{}: {} field, level {}, price to plant {}, earnings {} per harvest", i+1, f.crop, f.level.to_string().red().bold(), format_money(f.crop.get_planting_price()), format_money(f.earnings()))
            }
        ).collect::<Vec<String>>().join("\n  ");
        
        let menu_specific_string = match *state {
            State::MainMenu => "".to_string(),
            State::ShopMenu => {
                let crops = Farm::available_crops();
                let fields_string = crops.iter().enumerate().map(|(i, c)| 
                    format!("{}: {} field for {}, earnings per harvest {}", format!("{}", i+1).bold(), c, format_money(Field::calculate_price(*c)), format_money(c.payout()))
                ).collect::<Vec<String>>().join("\n");
                format!("{}{}\n{}", "0".bold(), ": Back", fields_string)
                },
            State::PlantMenu => "Pick a field to plant (0 to go back):".to_string(),
            State::HarvestMenu => "Pick a field to harvest (0 to go back):".to_string(),
            State::LevelUpMenu => "Pick a field to level up (0 to go back):".to_string(),
            State::SellMenu => "Pick a field to sell (0 to go back):".to_string(),
            _ => todo!(),
        };


        let out = format!("{}\n{}\n{}\n{}\n{}\n{}", welcome, balance, field_string, pick, options, menu_specific_string);
        
        // clear screen
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        println!("{}", out);
    }
}

fn input() -> u32 {
    loop {
        // clear line
        print!("{}", terminal::Clear(terminal::ClearType::CurrentLine));
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
        return input
    }
}



fn format_money(money: f64) -> String {
    format!("{}",format!("${:.2}", money).bold().bright_green())
}

fn delete_line(line_number: u16) {
    // Move the cursor to the start of the line
    print!("{}", cursor::MoveTo(0, line_number));

    // Clear the line
    print!("{}", terminal::Clear(terminal::ClearType::CurrentLine));

    // Flush the output to the terminal
    std::io::stdout().flush().unwrap();
}


fn delete_lines(from: u16, to: u16) {
    // current cursor pos
    
    for line_number in from..=to {
        delete_line(line_number);
    }
    
    // Move the cursor back to the original position
}

fn update_line(line_number: u16, new_content: &str) {
    // Move the cursor to the start of the line
    print!("{}", cursor::MoveTo(0, line_number));

    // Clear the line
    print!("{}", terminal::Clear(terminal::ClearType::CurrentLine));

    // Print the updated content
    print!("{}", style::Print(new_content));

    // Flush the output to the terminal
    std::io::stdout().flush().unwrap();
}