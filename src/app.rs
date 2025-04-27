use std::{error::Error, fs::{self, File}};
use clap::Parser;
use diffy::{self, DiffOptions};
use ratatui::{crossterm::event::{KeyEventKind, MouseEventKind}, style::{Color, Style}, text::{Line, Span}, widgets::Paragraph, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::ui::{generate_block, generate_line_numbers, Ui};
use crate::helpers::{compare_hashes, get_max_line_count};
use crate::args::Args;

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

        let f1 = fs::read_to_string(&self.args.file_1).unwrap();
        let f2 = fs::read_to_string(&self.args.file_2).unwrap();
        let f3 = fs::read_to_string(&self.args.file_3.as_ref().unwrap_or(&String::new())).unwrap_or(String::new());

        terminal.draw(|frame| {
            let layout = Ui::new(&frame, 1);
            let b = layout.boxes[0];
            let l1 = Line::from(format!("Computing the diffs between the {} files...", self.num_files));
            let l2 = Line::from("This may take a while...");
            let p = Paragraph::new(vec![l1, l2])
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(p, b);
        })?;

        let mut ops = DiffOptions::new();
        ops.set_context_len(if self.args.suppress_common_lines {10} else {usize::MAX});
        let diff1 = ops.create_patch(&f1, &f2);
        let diff2 = ops.create_patch(&f1, &f3);

        let (left_lines, middle_lines) = App::prepare_diff_lines(&diff1);
        let (_, right_lines) = App::prepare_diff_lines(&diff2);

        // Put a limit on the self.current_line so it won't go off the page. Harder for horizontal scroll :(
        // let max_file_len: usize = get_max_line_count(&v_fps);
        let max_file_len = std::cmp::max(middle_lines.len(), std::cmp::max(left_lines.len(), right_lines.len()));
        let mut max_height: usize = 0;
        
        loop {
            terminal.draw(|frame| {
                // Get the layout and min_width so that we show the same amount of characters per row
                let layout = Ui::new(&frame, self.num_files);
                let min_width = layout.boxes.iter().map(|&b| b.width).min().unwrap();
                max_height = layout.boxes.iter().map(|&b| b.height).min().unwrap() as usize;
                layout.boxes.iter().enumerate().for_each(|(i, &b)| {
                    // let mut fp = &v_fps[i as usize];
                    // fp.rewind().unwrap();

                    // let br = BufReader::new(fp);
                    // let lines = br.lines()
                    //     .skip(self.current_line)
                    //     .filter_map(Result::ok)
                    //     .take(b.height as usize - 2)
                    //     .collect::<Vec<String>>();

                    let box_name = match i {
                        0 => self.args.file_1.clone(),
                        1 => self.args.file_2.clone(),
                        2 => self.args.file_3.as_ref().unwrap().clone(),
                        _ => String::new()
                    };
                    let block = generate_block(box_name);

                    // let text = lines.iter().map(|line| {
                    //     if line.len() <= self.current_col {
                    //         return Line::from("");
                    //     }

                    //     Line::from(Span::styled(&line[self.current_col..std::cmp::min(line.len(), self.current_col + b.width as usize)], Style::default().fg(Color::Rgb(0xb0,  0xb0, 0xb0))))
                    // }).collect::<Vec<Line>>();

                    let mut text: Vec<Line> = Vec::new();
                    if i == 0 {
                        text = left_lines.iter().skip(self.current_line).map(|line| {
                            let span = &line.spans[0];
                            let style = span.style;
                            let mut content = span.content.clone().to_string();
                            let len = content.len();

                            if len <= self.current_col {
                                return Line::from("");
                            }

                            if self.args.hex  {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].as_bytes().chunks(self.args.width.unwrap_or(4)).map(|chunk| {
                                    chunk.iter().map(|x| format!("{:0x}", x)).collect()
                                }).collect::<Vec<String>>().iter().map(|l| {
                                    let mut s = String::new();
                                    for _ in l.len()..self.args.width.unwrap_or(4) *  2 {
                                        s.push('0');
                                    }
                                    format!("{}{}", s,  l)
                                }).collect::<Vec<String>>().join(" ");
                            }  else {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].to_string();
                            }

                            Line::styled(content, style)
                        }).collect::<Vec<Line>>();
                    } else if i == 1 {
                        text = middle_lines.iter().skip(self.current_line).map(|line| {
                            let span = &line.spans[0];
                            let style = span.style;
                            let mut content = span.content.clone().to_string();
                            let len = content.len();

                            if len <= self.current_col {
                                return Line::from("");
                            }
                            
                            if self.args.hex  {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].as_bytes().chunks(self.args.width.unwrap_or(4)).map(|chunk| {
                                    chunk.iter().map(|x| format!("{:0x}", x)).collect()
                                }).collect::<Vec<String>>().iter().map(|l| {
                                    let mut s = String::new();
                                    for _ in l.len()..self.args.width.unwrap_or(4) *  2 {
                                        s.push('0');
                                    }
                                    format!("{}{}", s,  l)
                                }).collect::<Vec<String>>().join(" ");
                            }  else {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].to_string();
                            }

                            Line::styled(content, style)
                        }).collect::<Vec<Line>>();
                    } else if i == 2 {
                        text = right_lines.iter().skip(self.current_line).map(|line| {
                            let span = &line.spans[0];
                            let style = span.style;
                            let mut content = span.content.clone().to_string();
                            let len = content.len();

                            if len <= self.current_col {
                                return Line::from("");
                            }
                            
                            if self.args.hex  {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].as_bytes().chunks(self.args.width.unwrap_or(4)).map(|chunk| {
                                    chunk.iter().map(|x| format!("{:0x}", x)).collect()
                                }).collect::<Vec<String>>().iter().map(|l| {
                                    let mut s = String::new();
                                    for _ in l.len()..self.args.width.unwrap_or(4) *  2 {
                                        s.push('0');
                                    }
                                    format!("{}{}", s,  l)
                                }).collect::<Vec<String>>().join(" ");
                            }  else {
                                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].to_string();
                            }

                            Line::styled(content, style)
                        }).collect::<Vec<Line>>();
                    }

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
                                KeyCode::Char('e') => {
                                    self.current_line = max_file_len - max_height +  3;
                                    break;
                                },
                                KeyCode::Char('b') => {
                                    self.current_line = 0;
                                    break;
                                },
                                KeyCode::PageDown |
                                KeyCode::Char('n') => {
                                    if self.current_line + max_height  - 3 < max_file_len {
                                        self.current_line += max_height - 5
                                    }
                                    break;
                                },
                                KeyCode::PageUp |
                                KeyCode::Char('l') => {
                                    self.current_line = std::cmp::max(self.current_line.saturating_sub(max_height - 5), 0);
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
                        Event::Mouse(e) => {
                            match e.kind {
                                MouseEventKind::ScrollDown => {
                                    // Down allow scroll too far down
                                    if self.current_line + max_height  - 3 < max_file_len {
                                        self.current_line += 1;
                                        break;
                                    }
                                },
                                MouseEventKind::ScrollUp => {
                                    // Don't scroll past beginning
                                    if self.current_line > 0 {
                                        self.current_line -= 1;
                                        break;
                                    }
                                }
                                _ => continue
                            }
                        }
                        _ => continue
                    }
                }

                if should_break {
                    break;
                }
        }
        
        Ok(())
    }

    fn prepare_diff_lines<'a>(patch: &'a diffy::Patch<'a, str>) -> (Vec<Line<'a>>, Vec<Line<'a>>) {
        let mut left: Vec<Line> = Vec::new();
        let mut right: Vec<Line> = Vec::new();

        let context_style = Style::default().fg(Color::DarkGray);
        let deleted_style = Style::default().fg(Color::Red);
        let added_style = Style::default().fg(Color::Green);
        let empty_line = Line::from(vec![Span::from("")]);

        for hunk in patch.hunks() {
            for line in hunk.lines() {
                match line.to_owned() {
                    diffy::Line::Context(l) => {
                        let styled = Span::styled(l.trim(), context_style);
                        left.push(Line::from(vec![styled.clone()]));
                        right.push(Line::from(vec![styled]));
                    },
                    diffy::Line::Delete(l) => {
                        let styled = Span::styled(l.trim(), deleted_style);
                        left.push(Line::from(vec![styled]));
                        right.push(empty_line.clone()); // Add placeholder to keep alignment
                    },
                    diffy::Line::Insert(l) => {
                        let styled = Span::styled(l.trim(), added_style);
                        left.push(empty_line.clone()); // Add placeholder
                        right.push(Line::from(vec![styled]));
                    }
                }
            }
        }
        (left, right)
    }
}
