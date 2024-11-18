pub mod ship;

pub mod fighter;
pub mod frigate;
pub mod missile;

pub mod navigation;
pub mod ballistics;
pub mod settings;
pub mod utility;
pub mod radar;


#[allow(unused_imports)]
use ship::Ship; // this is so that oort can find the base ship struct