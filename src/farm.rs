use std::fmt;
use crossterm::style::Stylize;
use strum::IntoEnumIterator;
use util::Result;

use serde::{Serialize, Deserialize};

use crate::util::{self, GameError};

type Money = f64;
type Level = u8;

#[derive(Clone, Copy, Debug, strum::EnumIter, Serialize, Deserialize)]
pub enum Crop {
    Wheat,
    Potato,
    Carrot,
}

impl Crop {
    pub fn get_new_field_price(&self) -> Money {
        match self {
            Crop::Wheat => 10.,
            Crop::Potato => 100.,
            Crop::Carrot => 1000.,
        }
    }

    pub fn get_planting_price(&self) -> Money {
        match self {
            Crop::Wheat => 1.,
            Crop::Potato => 20.,
            Crop::Carrot => 50.,
        }
    }

    pub fn get_max_level(&self) -> Level {
        match self {
            Crop::Wheat => 5,
            Crop::Potato => 10,
            Crop::Carrot => 20,
        }
    }

    pub fn level_multiplier(&self) -> f64 {
        0.5
    }

    pub fn grow_time(&self) -> u128 {
        let time = match self {
            Crop::Wheat => 100,
            Crop::Potato => 300,
            Crop::Carrot => 1000,
        };
        util::seconds_to_millis(time)
    }

    pub fn payout(&self) -> Money {
        match self {
            Crop::Wheat => 1.,
            Crop::Potato => 10.,
            Crop::Carrot => 100.,
        }
    }

    pub fn get_next_level_price(&self, level: Level) -> Money {
        let base_price = self.get_planting_price() * 10.;
        let level_multiplier = self.level_multiplier()/2.;
        let price = base_price * (level_multiplier * level as f64);
        price
    }
}

impl fmt::Display for Crop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Crop::Wheat => "Wheat".bold().dark_green(),
            Crop::Potato => "Potato".bold().dark_yellow(),
            Crop::Carrot => "Carrot".bold().yellow(),
        };
        write!(f, "{s}")
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Field {
    pub crop: Crop,
    pub level: Level,
    pub plant_timestamp: Option<u128>,
}

impl Field {
    pub fn new(crop: Crop) -> Field {
        Self {
            crop,
            level: 1,
            plant_timestamp: None,
        }
    }

    pub fn calculate_price(crop: Crop) -> Money {
        crop.get_new_field_price()
    }

    pub fn level_up_price(&self) -> Result<Money> {
        if self.level >= self.crop.get_max_level() { return Err(GameError::MaxLevelReached) }
        Ok(self.crop.get_next_level_price(self.level))
    }

    pub fn level_up(&mut self) -> Result<()> {
        if self.level >= self.crop.get_max_level() { return Err(GameError::MaxLevelReached) }
        self.level += 1;
        Ok(())
    }

    pub fn planted(&self) -> bool {
        self.plant_timestamp.is_some()
    }

    pub fn plant(&mut self, timestamp: u128) -> Result<()> {
        if self.planted() { return Err(GameError::AlreadyPlanted) }
        self.plant_timestamp = Some(timestamp);
        Ok(())
    }

    pub fn time_to_farm(&self, timestamp: u128) -> u128 {
        self.crop.grow_time().checked_sub(timestamp - self.plant_timestamp.unwrap()).unwrap_or(0)
    }

    pub fn farm(&mut self) -> Result<()> {
        if !self.planted() { return Err(GameError::AlreadyFarmed) }
        if self.time_to_farm(util::timestamp()) > 0 { return Err(GameError::NotYetReady) }
        self.plant_timestamp = None;
        Ok(())
    }

    pub fn earnings(&self) -> Money {
        let payout = self.crop.payout();
        payout * (1. + self.crop.level_multiplier()).powi(self.level as i32)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Farm {
    pub name: String,
    pub money: f64,
    pub fields: Vec<Field>,
}

impl Farm {
    pub fn new(name: String) -> Self {
        Self {
            name,
            money: 20.,
            fields: Vec::new(),
        }
    }

    pub fn available_crops() -> Vec<Crop> {
        Crop::iter().collect::<Vec<Crop>>()
    }

    pub fn buy_field(&mut self, crop: Crop) -> Result<()> {
        let price = crop.get_new_field_price();
        if self.money < price { return Err(GameError::InsufficientFunds) }
        self.fields.push(Field::new(crop));
        self.money -= price;
        Ok(())
    }

    pub fn level_up_field(&mut self, id: u32) -> Result<()> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameError::OutOfBounds),
        };

        if field.level_up_price()? > self.money { return Err(GameError::InsufficientFunds) }
        let level_up_price = field.level_up_price()?;

        field.level_up()?;
        self.money -= level_up_price;

        Ok(())
    }

    pub fn plant_field(&mut self, id: u32) -> Result<()> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameError::OutOfBounds),
        };

        if field.crop.get_planting_price() > self.money { return Err(GameError::InsufficientFunds) }

        self.money -= field.crop.get_planting_price();
        field.plant(util::timestamp())?;

        Ok(())
    }

    pub fn farm_field(&mut self, id: u32) -> Result<Money> {
        let field = match self.fields.get_mut(id as usize) {
            Some(field) => field,
            None => return Err(GameError::OutOfBounds),
        };

        field.farm()?;
        let payout = field.earnings();
        self.money += payout;
        Ok(payout)
    }

    pub fn save_to_path(&self, path: String) {
        let json: String = serde_json::to_string(self).unwrap();
        let file = std::fs::File::create(path).unwrap();
        // write all to file
        std::io::Write::write_all(&mut std::io::BufWriter::new(file), json.as_bytes()).unwrap();
    }

    pub fn load_from_path(path: String) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        let farm: Farm = serde_json::from_str(&contents).unwrap();
        farm
    }
}
