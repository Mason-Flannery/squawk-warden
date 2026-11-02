#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Reading {
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DoorAction {
    #[default]
    None,
    Open,
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DoorCommand {
    pub id: u32,
    pub action: DoorAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAck {
    pub id: u32,
}

