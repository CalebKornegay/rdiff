use std::{error::Error, thread::sleep, time::Duration};

use ratatui::{prelude::{Constraint, Direction, Layout}, Terminal, style::{Style, Color}, widgets::{Borders, Block}};
use ratatui::crossterm::event::{self, Event, KeyCode};

pub struct App {
    current_scroll: i32
}

impl App {
    pub fn new() -> Self {
        Self {current_scroll: 0}
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|frame| {
            let chunks = Layout::default()
            .direction(Direction::Horizontal) // Arrange items horizontally
            .constraints([
                Constraint::Percentage(50), // First box takes 50%
                Constraint::Percentage(50), // Second box takes the remaining 50%
            ])
            .split(frame.size()); // Divide the entire terminal area

            let block1 = Block::default()
                .title("Box 1")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightCyan));
            frame.render_widget(block1, chunks[0]);

            let block2 = Block::default()
                .title("Box 2")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightGreen));
            frame.render_widget(block2, chunks[1]);
            })?;

            // Block until an event occurs
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => println!("Got key {}", key.code)
                }
            }
            // You can also handle other event types here, like Mouse events or Resize events
        }
        }
        Ok(())
    }
}