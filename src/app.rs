use std::{error::Error, time::Duration};
use ratatui::{style::{Color, Style}, widgets::{Block, Borders, Paragraph}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};
use clap::Parser;

use crate::ui::Ui;

#[derive(Parser, Debug)]
#[command(
    name = "rdiff",
    author = "Caleb Kornegay <caleb.kornegay@gmail.com>",
    version = "0.0.1",
    about = "A TUI app to visually diff two text files",
    long_about = "This tool shows a side-by-side diff of two or three files\nAuthor: Caleb Kornegay <caleb.kornegay@gmail.com>"
)]
pub struct Args {
    #[arg(help = "First file")]
    pub file_1: String,

    #[arg(help = "Second file")]
    pub file_2: String,

    #[arg(help = "Optional third file")]
    pub file_3: Option<String>,

    #[arg(short = 'x', long)]
    pub hex: bool
}

pub struct App {
    current_scroll: i32,
    args: Args,
    num_files: u8
}

impl App {
    pub fn new(args: Args) -> Self {
        let mut num_files: u8 = 2;
        if args.file_3.is_some() {
            num_files += 1;
        }

        Self {
            current_scroll: 0,
            args: args,
            num_files: num_files
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            // println!("hi");
            terminal.draw(|frame| {
                let layout = Ui::new(&frame, self.num_files);
                layout.boxes.iter().enumerate().for_each(|(i, &b)| {
                    let mut box_num = String::from("File ");
                    box_num.push_str(&(i+1).to_string());
                    let block = Block::default()
                    .title(box_num)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)));
                    frame.render_widget(Paragraph::new("Hello world!").block(block), b);
                });
            })?;

            // Block until an event occurs
            if event::poll(Duration::from_secs(u64::MAX))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        _ => continue
                    }
                }
                // You can also handle other event types here, like Mouse events or Resize events
            }
        }
        Ok(())
    }
}