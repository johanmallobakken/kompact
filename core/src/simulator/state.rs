use std::{
    hash::Hash,
    fmt::Debug
};

use crate::prelude::*;

pub trait GetState<T>{
    fn get_state(&self) -> T;
}

pub trait Invariant<T: Any>{
    fn check(&self, state: Vec<T>) -> Result<(), SimulationError>;
}

pub trait ProgressCounter {
    fn get_progress(&self) -> u64;
}
pub struct SimulationError{}