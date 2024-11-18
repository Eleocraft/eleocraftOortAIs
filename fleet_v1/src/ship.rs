use oort_api::prelude::*;

use crate::fighter::*;
use crate::missile::*;

pub enum Ship {
    Fighter(Fighter),
    Missile(Missile)
}

impl Ship {
    pub fn new() -> Ship {
        match class() {
            Class::Fighter => { Ship::Fighter(Fighter::new()) }
            Class::Missile => { Ship::Missile(Missile::new()) }
            _ => { Ship::Fighter(Fighter::new()) } // this is only an overflow
        }
    }

    pub fn tick(&mut self) {
        match self {
            Ship::Fighter(implementation) => { implementation.tick(); }
            Ship::Missile(implementation) => { implementation.tick(); }
        }
    }
}