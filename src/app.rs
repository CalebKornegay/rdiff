use std::{error::Error, thread::sleep, time::Duration};

use ratatui::{prelude::{Constraint, Direction, Layout}, Terminal, style::{Style, Color}, widgets::{Borders, Block}};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::ui::Ui;

pub struct App {
    current_scroll: i32,
    args: Vec<String>
}

impl App {
    pub fn new() -> Self {
        Self {
            current_scroll: 0,
            args: std::env::args().collect::<Vec<String>>()
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|frame| {
                let boxes = Ui::new(&frame);
                boxes.iter().for_each(|box| {
                    let block = Block::default()
                    .title("Box 1")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::LightCyan));
                    frame.render_widget(block, box);
                });
            // let block1 = Block::default()
            //     .title("Box 1")
            //     .borders(Borders::ALL)
            //     .style(Style::default().bg(Color::LightCyan));
            // frame.render_widget(block1, boxes[0]);

            // let block2 = Block::default()
            //     .title("Box 2")
            //     .borders(Borders::ALL)
            //     .style(Style::default().bg(Color::LightGreen));
            // frame.render_widget(block2, boxes[1]);
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