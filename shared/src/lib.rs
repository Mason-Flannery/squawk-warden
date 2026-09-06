#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Reading {
    pub temperature: f32,
    pub humidity: f32,
}


