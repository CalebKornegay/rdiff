use std::{error::Error, fs::File, io::{BufRead, BufReader, Seek}};
use ratatui::{crossterm::event::KeyEventKind, style::{Color, Style}, text::{Line, Span}, widgets::Paragraph, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};
use clap::Parser;

use crate::ui::{generate_block, generate_line_numbers, Ui};
use crate::helpers::{compare_hashes};

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
    current_col: usize,
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
            current_col: 0,
            args: args,
            num_files: num_files
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        // Put the file handles in a vec so we can iterate over them in the boxes later
        let mut v_fps: Vec<File> = Vec::new();
        v_fps.push(File::open(&self.args.file_1)?);
        v_fps.push(File::open(&self.args.file_2)?);
        if self.args.file_3.is_some() {
            v_fps.push(File::open(self.args.file_3.as_ref().unwrap())?);
        }

        compare_hashes(&mut v_fps)?;

        // Get the file size of each so we can see the max
        let lc_1: usize = BufReader::new(&v_fps[0])
            .lines().filter_map(Result::ok).count();
        let lc_2: usize = BufReader::new(&v_fps[1])
            .lines().filter_map(Result::ok).count();
        let lc_3: usize = {
            if v_fps.len() == 3 {
                BufReader::new(&v_fps[2])
                .lines().filter_map(Result::ok).count()
            } else {
                0
            }
        };

        // Put a limit on the self.current_line so it won't go off the page. Harder for horizontal scroll :(
        let max_file_len: usize = std::cmp::max(std::cmp::max(lc_1, lc_2), lc_3);
        let mut max_height: usize = 0;
        
        loop {
            terminal.draw(|frame| {
                // Get the layout and min_width so that we show the same amount of characters per row
                let layout = Ui::new(&frame, self.num_files);
                let min_width = layout.boxes.iter().map(|&b| b.width).min().unwrap();
                max_height = layout.boxes.iter().map(|&b| b.height).min().unwrap() as usize;
                layout.boxes.iter().enumerate().for_each(|(i, &b)| {
                    let mut fp = &v_fps[i as usize];
                    fp.rewind().unwrap();

                    let br = BufReader::new(fp);
                    let lines = br.lines()
                        .skip(self.current_line)
                        .take(b.height as usize - 2)
                        .filter_map(Result::ok)
                        .collect::<Vec<String>>();

                    let box_name = match i {
                        0 => self.args.file_1.clone(),
                        1 => self.args.file_2.clone(),
                        2 => self.args.file_3.as_ref().unwrap().clone(),
                        _ => String::new()
                    };
                    let block = generate_block(box_name);

                    let text = lines.iter().map(|line| {
                        if line.len() <= self.current_col {
                            return Line::from("");
                        }

                        Line::from(Span::styled(&line[self.current_col..std::cmp::min(line.len(), self.current_col + b.width as usize)], Style::default().fg(Color::Rgb(0xb0,  0xb0, 0xb0))))
                    }).collect::<Vec<Line>>();

                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .left_aligned();
                    
                    let shift = ((self.current_line + b.height as usize - 3) as f64).log10() as u16 + 3;

                    // Reduce width a little and shift over so we can render line numbers
                    let mut text_rect = b.clone();
                    text_rect.width = min_width - shift;
                    text_rect.x += shift;

                    // Generate the box that the line numbers go into
                    let mut line_numbers_rect = b.clone();
                    line_numbers_rect.width = shift;

                    let line_numbers = generate_line_numbers(self.current_line, b.height as usize);
                    
                    frame.render_widget(line_numbers, line_numbers_rect);
                    frame.render_widget(paragraph, text_rect);
                });
            })?;

                // While loop so that we don't re-render the screen when nothing would've changed
                let mut should_break = false;
                while !should_break {
                    let event: Event = event::read()?;
                    match event {
                        Event::Key(key) =>  {
                            // Don't render the key event twice
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }

                            // Enable quit, refresh, and vertical and horizontal scroll
                            match key.code {
                                KeyCode::Char('q') => should_break = true,
                                KeyCode::Char('r') => {
                                    self.current_col = 0;
                                    self.current_line = 0;
                                    break;
                                },
                                KeyCode::Right => {
                                    self.current_col += 1;
                                    break;
                                },
                                KeyCode::Left => {
                                    // Don't scroll past beginning of line
                                    if self.current_col > 0 {
                                        self.current_col -= 1;
                                        break;
                                    }
                                },
                                KeyCode::Up => {
                                    // Don't scroll past beginning
                                    if self.current_line > 0 {
                                        self.current_line -= 1;
                                        break;
                                    }
                                },
                                KeyCode::Down => {
                                    // Down allow scroll too far down
                                    if self.current_line + max_height  - 3 < max_file_len {
                                        self.current_line += 1;
                                        break;
                                    }
                                }
                                _ => continue
                            }
                        },
                        _ => continue
                    }
                }

                if should_break {
                    break;
                }
        }
        
        Ok(())
    }
}
