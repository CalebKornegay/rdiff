use std::{error::Error, fs::File, io::{BufRead, BufReader, Read, Seek, Write}, time::Duration};
use ratatui::{buffer::Buffer, crossterm::event::KeyEventKind, layout::Margin, style::{Color, Style, Styled}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Wrap}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};
use sha2::{Sha256, Digest};
use clap::Parser;

use crate::ui::{generate_block, Ui};

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

        let mut equal = true;
        let mut hashes: Vec<sha2::digest::Output<Sha256>> = Vec::new();
        v_fps.iter_mut()
            .for_each(|fp| {
                let mut hash = Sha256::new();
                let mut buffer = [0; 1024];
                
                loop {
                    let bytes_read = fp.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    hash.update(&buffer[..bytes_read]);
                }

                hashes.push(hash.finalize());
                fp.rewind().unwrap();
            });

        for i in 1..hashes.len() {
            if hashes[i] != hashes[i - 1] {
                equal = false;
            }
        }
        drop(hashes);

        if equal {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "There is no diff between the files")));
        }

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
                        .take(b.height as usize)
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

                        Line::from(Span::styled(&line[self.current_col..std::cmp::min(line.len(), self.current_col + b.width as usize)], Style::default().fg(Color::Rgb(0x9b,  0x9b, 0x9b))))
                    }).collect::<Vec<Line>>();

                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .left_aligned();
                    
                    let mut rect = b.clone();
                    rect.width = min_width;
                    
                    frame.render_widget(paragraph, rect);
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
