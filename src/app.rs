use std::{error::Error, fs::File, io::{BufRead, BufReader, Seek}, time::Duration};
use ratatui::{layout::Margin, style::{Color, Style, Styled}, widgets::{Block, Borders, Paragraph, Wrap}, Terminal, text::Span};
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
    current_line: usize,
    args: Args,
    num_files: u8
}

impl App {
    pub fn new() -> Self {
        let args = Args::parse();
        let mut num_files: u8 = 2;

        if args.file_3.is_some() {
            num_files += 1;
        }

        Self {
            current_line: 0,
            args: args,
            num_files: num_files
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        let mut v_fps: Vec<File> = Vec::new();
        v_fps.push(File::open(&self.args.file_1)?);
        v_fps.push(File::open(&self.args.file_2)?);
        if self.args.file_3.is_some() {
            v_fps.push(File::open(self.args.file_3.as_ref().unwrap())?);
        }

        let lc_1: usize = BufReader::new(&v_fps[0])
            .lines().filter_map(Result::ok).count();
        v_fps[0].rewind()?;
        let lc_2: usize = BufReader::new(&v_fps[1])
            .lines().filter_map(Result::ok).count();
        v_fps[1].rewind()?;
        let lc_3: usize = {
            if v_fps.len() == 3 {
                let c = BufReader::new(&v_fps[2])
                .lines().filter_map(Result::ok).count();
                v_fps[2].rewind()?;
                c
            } else {
                0
            }
        };
        
        loop {
            // println!("hi");
            terminal.draw(|frame| {
                let layout = Ui::new(&frame, self.num_files);
                layout.boxes.iter().enumerate().for_each(|(i, &b)| {
                    let mut fp = &v_fps[i as usize];
                    fp.rewind().unwrap();

                    let br = BufReader::new(fp);
                    let lines = br.lines()
                        .skip(self.current_line)
                        .take(10)
                        .filter_map(Result::ok)
                        // .filter(|v| v.is_ok())
                        // .map(|v| v.unwrap())
                        .collect::<Vec<String>>();

                    // let mut box_num = String::from("File ");
                    // box_num.push_str(&(i+1).to_string());
                    let box_name = match i {
                        0 => self.args.file_1.clone(),
                        1 => self.args.file_2.clone(),
                        2 => self.args.file_3.as_ref().unwrap().clone(),
                        _ => String::new()
                    };

                    let block = Block::default()
                    .title(box_name)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)));


                    let text = Span::styled(lines.join("\n"), Style::default().fg(Color::Rgb(0xcb,  0xcb, 0xcb)));

                    let paragraph = Paragraph::new(text)
                        .block(block).wrap(Wrap {trim: true})
                        .left_aligned();

                    frame.render_widget(paragraph, b.inner(Margin::new(1, 0)));
                });
            })?;

            // Block until an event occurs
            // if event::poll(Duration::from_secs(u64::MAX))? {
                let mut should_break = false;
                while !should_break {
                    let event: Event = event::read()?;
                    match event {
                        Event::Key(key) =>  {
                            match key.code {
                                KeyCode::Char('q') => should_break = true,
                                _ => continue
                            }
                        },
                        _ => continue
                    }
                }

                if should_break {
                    break;
                }
                // You can also handle other event types here, like Mouse events or Resize events
            // }
        }
        
        Ok(())
    }
}
