use std::{
    hash::Hash,
    fmt::Debug
};

use crate::prelude::*;

pub trait GetState<T>
where T: Debug
{
    fn get_state(&self) -> T;
}

pub trait Invariant<T>{
    fn check(&self, state: Vec<T>) -> Result<(), SimulationError>;
}

pub trait ProgressCounter {
    fn get_progress(&self) -> u64;
}
pub struct SimulationError{
    pub message: String
}