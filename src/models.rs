use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Task {
    pub input: String,
    pub status: Status,
}

#[derive(Debug)]
pub enum Status {
    Qued,
    Processing,
    Done,
}

pub type Que = Arc<Mutex<VecDeque<(String, String)>>>;
pub type Database = Arc<Mutex<HashMap<String, Task>>>;
