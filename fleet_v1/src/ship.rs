use oort_api::prelude::*;

use crate::fighter::*;
use crate::missile::*;
use crate::frigate::*;
use crate::cruiser::*;

pub enum Ship {
    Fighter(Fighter),
    Missile(Missile),
    Frigate(Frigate),
    Cruiser(Cruiser)
}

impl Ship {
    pub fn new() -> Ship {
        match class() {
            Class::Fighter => { Ship::Fighter(Fighter::new()) }
            Class::Missile => { Ship::Missile(Missile::new()) }
            Class::Frigate => { Ship::Fighter(Fighter::new()) }
            Class::Cruiser => { Ship::Cruiser(Cruiser::new()) }
            Class::Torpedo => { Ship::Missile(Missile::new()) } // Torpedo is a missile with a different name (for now)
            _ => { panic!("ship class is unknown") }
        }
    }

    pub fn tick(&mut self) {
        match self {
            Ship::Fighter(implementation) => { implementation.tick(); }
            Ship::Missile(implementation) => { implementation.tick(); }
            Ship::Frigate(implementation) => { implementation.tick(); }
            Ship::Cruiser(implementation) => { implementation.tick(); }
        }
    }
}