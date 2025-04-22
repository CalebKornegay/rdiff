use std::{error::Error, thread::sleep, time::Duration};

use ratatui::{crossterm::terminal, Terminal};

pub struct App {
    current_scroll: i32
}

impl App {
    pub fn new() -> Self {
        Self {current_scroll: 0}
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}