use std::{error::Error, ffi::{OsStr, OsString}, fs::{self, File}, path::Path};
use clap::Parser;
use diffy::{self, DiffOptions};
use ratatui::{crossterm::event::{KeyEventKind, MouseEventKind}, layout::{Constraint, Rect}, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Clear, Paragraph}, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode};
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};
use syntect_tui::translate_colour;
use imara_diff::{Algorithm, diff, UnifiedDiffBuilder, intern::InternedInput};

use crate::ui::{generate_block, Ui};
use crate::helpers::compare_hashes;
use crate::args::Args;

pub struct App {
    current_line: usize,
    current_col: usize,
    args: Args
}

impl App {
    pub fn new() -> Self {
        let args = Args::parse();

        Self {
            current_line: 0,
            current_col: 0,
            args: args
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        let ts = ThemeSet::load_defaults();
        let ps = SyntaxSet::load_defaults_newlines();

        let theme = &ts.themes["base16-ocean.dark"];

        
        let mut backgrounds: Vec<Color> = Vec::new();
        
        let mut syntaxes = vec![
            ps.find_syntax_for_file(&self.args.file_1),
            ps.find_syntax_for_file(&self.args.file_2),
            ].iter().map(|s| {
                if s.is_ok() && s.as_ref().unwrap().is_some() {
                    let syn = s.as_ref().unwrap().unwrap();
                    if syn.name == ps.find_syntax_plain_text().name {
                        backgrounds.push(Color::Rgb(0x12, 0x12, 0x12));
                        None
                    } else {
                        backgrounds.push({
                            if theme.settings.background.is_some() {
                                translate_colour(theme.settings.background.unwrap()).unwrap_or(Color::Rgb(0x12, 0x12, 0x12))
                            } else {
                                Color::Rgb(0x12, 0x12, 0x12)
                            }
                        });
                        Some(HighlightLines::new(syn, theme))
                    }
                } else {
                    None
                }
            }).collect::<Vec<Option<HighlightLines>>>();
            
            // Put the file handles in a vec so we can iterate over them in the boxes later
            let mut v_fps: Vec<File> = Vec::new();
            v_fps.push(File::open(&self.args.file_1)?);
            v_fps.push(File::open(&self.args.file_2)?);
            
            // Compute the hashes to see if the files are the same
            compare_hashes(&mut v_fps)?;
            drop(v_fps);
            
            // Hopefully this doesn't blow up your computer
            let f1 = fs::read_to_string(&self.args.file_1)?;
            let f2 = fs::read_to_string(&self.args.file_2)?;
            
            // let input = InternedInput::new(f1.as_str(), f2.as_str()); 
            // let out = diff(Algorithm::Histogram, &input, UnifiedDiffBuilder::new(&input));
            // eprintln!("{:?}", out);
        // Draw a loading screen in case the files are large so the user doesn't think our program sucks (as much)
        terminal.draw(|frame| {
            let l1 = Line::from("Computing the diffs between the files...");
            let l2 = Line::from("This may take a while...");

            let block = generate_block(String::new());
            
            let b = Ui::center_rect(frame.area(), Constraint::Length(l1.width() as u16), Constraint::Length(2));

            let p = Paragraph::new(vec![l1, l2])
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::Yellow));

            frame.render_widget(block, frame.area());
            frame.render_widget(p, b);
        })?;

        // Set up the diff options based on cli args
        let mut ops = DiffOptions::new();
        ops.set_context_len(if self.args.suppress_common_lines {self.args.context_lines.unwrap_or(0)} else {usize::MAX});

        let keybinds_text = vec![
            "[n] next page",
            "[l] last page",
            "[h] help",
            "[r] reset",
            "[q] quit",
        ];

        // Compute the diffs
        let diff1 = ops.create_patch(&f1, &f2);

        // Get the formatted Lines for display for each frame, slicing them based 
        // let (left_lines, right_lines) = App::prepare_diff_lines(&diff1);
        let (left_colors, left_lines, right_colors, right_lines) = App::get_diff_lines(&diff1);

        // Put a limit on the self.current_line so it won't go off the page. Harder for horizontal scroll :(
        let max_file_len = std::cmp::max(left_colors.len(), right_colors.len());
        let mut max_height: usize = 0;
        let mut show_help: bool = false;
        
        loop {
            terminal.draw(|frame| {
                frame.render_widget(Clear, frame.area());
                // Show the help screen if 'h' was pressed
                if show_help {
                    Ui::show_help(frame, &keybinds_text);
                    return;
                }
                
                // Get the layout and min_width so that we show the same amount of characters per row
                let mut layout_rect = frame.area();
                layout_rect.height -= 1;

                let layout = Ui::new(layout_rect);
                let min_width = layout.get_min_width();
                max_height = layout.get_height();

                let keybinds_rect = Rect::new(
                    0, layout_rect.height, layout_rect.width, 1
                );

                frame.render_widget( 
                    Block::default()
                    .title(keybinds_text.join(" "))
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .title_style(Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)))
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
                    .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12))), keybinds_rect
                );

                layout.boxes.iter().enumerate().for_each(|(i, &b)| {
                    let box_name = match i {
                        0 => Path::new(&self.args.file_1.clone()).file_name().unwrap().to_os_string(),
                        1 => Path::new(&self.args.file_2.clone()).file_name().unwrap().to_os_string(),
                        _ => OsString::new()
                    };
                    let block = generate_block(box_name.into_string().unwrap());

                    // let text = match i {
                    //     0 => self.generate_block_lines(&left_lines, &b),
                    //     1 => self.generate_block_lines(&right_lines, &b),
                    //     _ => Vec::new(),
                    // };

                    let text = match i {
                        0 => self.generate_rect_lines(&left_lines, &b, &mut  syntaxes[i], &ps),
                        1 => self.generate_rect_lines(&right_lines, &b, &mut syntaxes[i], &ps),
                        _ => Vec::new(),
                    };

                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .bg(backgrounds[i])
                        .left_aligned();
                    
                    let shift = ((self.current_line + b.height as usize - 3) as f64).log10() as u16 + 3;

                    // Reduce width a little and shift over so we can render line numbers
                    let mut text_rect = b.clone();
                    text_rect.width = min_width - shift;
                    text_rect.x += shift;

                    // Generate the box that the line numbers go into
                    let mut line_numbers_rect = b.clone();
                    line_numbers_rect.width = shift;

                    // let line_numbers = generate_line_numbers(self.current_line, b.height as usize);
                    let line_numbers = match i {
                        0 => self.get_line_numbers(&left_colors, b.height as usize),
                        1 => self.get_line_numbers(&right_colors, b.height as usize),
                        _ => Paragraph::new("")
                    };
                    
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
                                KeyCode::Char('h') => {
                                    show_help = !show_help;
                                    break;
                                },
                                KeyCode::Char('q') => should_break = true,
                                KeyCode::Char('r') => {
                                    self.current_col = 0;
                                    self.current_line = 0;
                                    break;
                                },
                                KeyCode::Char('e') => {
                                    if max_height > max_file_len {
                                        break;
                                    }
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
                                    self.current_line = self.current_line.saturating_sub(max_height - 5);
                                    break;
                                },
                                // KeyCode::Right => {
                                //     self.current_col += 1;
                                //     break;
                                // },
                                // KeyCode::Left => {
                                //     // Don't scroll past beginning of line
                                //     if self.current_col > 0 {
                                //         self.current_col -= 1;
                                //         break;
                                //     }
                                // },
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

    fn generate_rect_lines<'a>(&self, lines: &'a Vec<String>, b: &Rect, highlighter: &mut Option<HighlightLines<'a>>, syntax: &SyntaxSet) -> Vec<Line<'a>> {
        lines.iter().skip(self.current_line).take(b.height as usize).map(|line| {
            let len = line.len();
            let mut size = 0;

            if len <= self.current_col {
                return Line::from("");
            }

            if highlighter.is_none() {
                Line::from(
                    line[
                        self.current_col..
                        std::cmp::min(
                            len,
                            self.current_col + b.width as usize
                        )
                    ].to_string()
                )
            } else {
                Line::from(
                    highlighter.as_mut().unwrap().highlight_line(
                        // &line[
                        //     self.current_col..
                        //     std::cmp::min(
                        //         len,
                        //         self.current_col + b.width as usize
                        //     )
                        // ], syntax
                        line, syntax
                    )
                    .unwrap_or(Vec::new())
                    .into_iter()
                    .map(|segment| {
                        let fg = Color::Rgb(segment.0.foreground.r, segment.0.foreground.g, segment.0.foreground.b);

                        // 👇 Don't use background color — override or skip
                        let style = Style::default().fg(fg);

                        let ret;
                        
                        if size < self.current_col && size + segment.1.len() > self.current_col {
                            ret = Span::styled(
                                segment.1[self.current_col.saturating_sub(size)..]
                                .to_string(),
                                style
                            );
                        } else if size + segment.1.len() < self.current_col {
                            ret = Span::from("");
                        } else if size + segment.1.len() > self.current_col + b.width as usize {
                            ret = Span::styled(
                                    {
                                        if segment.1.len() > b.width as usize {
                                            segment.1[self.current_col..std::cmp::min(self.current_col+b.width as usize, segment.1.len())].to_string()
                                        } else {
                                            segment.1[..segment.1.len().saturating_sub(size)].to_string()
                                        }
                                    },
                                    style
                                )
                        } else {
                            ret = Span::styled(segment.1, style);
                        }

                        size += segment.1.len();

                        ret

                        // Span::styled(segment.1, style)
                    })
                    .collect::<Vec<Span>>()
                )
            }
        })
        .collect::<Vec<Line>>()
    }

    fn get_line_numbers<'a>(&self, colors: &Vec<char>, height: usize) -> Paragraph<'a> {
        Paragraph::new(
                colors.iter().enumerate().skip(self.current_line).take(height).map(|(i, &c)| {
                Line::styled((i.saturating_add(1)).to_string(),
                    Style::default().fg(
                        match c {
                            'g' => Color::Green,
                            'r' => Color::Red,
                            _ => Color::DarkGray,
                        }
                    ).bg(Color::Rgb(0x12, 0x12, 0x12))
                )
            })
            .collect::<Vec<Line>>()
        ).block(generate_block(String::new()))
        .left_aligned()
    }

    fn get_diff_lines<'a>(patch: &'a diffy::Patch<'a, str>) -> (Vec<char>, Vec<String>, Vec<char>, Vec<String>) {
        let len = patch.hunks().iter().map(|hunk| hunk.lines().len()).sum();
        let mut left_colors: Vec<char> = Vec::with_capacity(len);
        let mut right_colors: Vec<char> = Vec::with_capacity(len);
        let mut left_lines: Vec<String> = Vec::with_capacity(len);
        let mut right_lines: Vec<String> = Vec::with_capacity(len);

        for hunk in patch.hunks() {
            for line in hunk.lines() {
                match line.to_owned() {
                    diffy::Line::Context(l) => {
                        left_colors.push('c');
                        right_colors.push('c');
                        left_lines.push(String::from(l.trim_end().replace("\t", &" ".repeat(4))));
                        right_lines.push(String::from(l.trim_end().replace("\t", &" ".repeat(4))));
                    },
                    diffy::Line::Delete(l) => {
                        left_colors.push('r');
                        right_colors.push('c');
                        left_lines.push(String::from(l.trim_end().replace("\t", &" ".repeat(4))));
                        right_lines.push(String::new());
                    },
                    diffy::Line::Insert(l) => {
                        left_colors.push('c');
                        right_colors.push('g');
                        left_lines.push(String::new());
                        right_lines.push(String::from(l.trim_end().replace("\t", &" ".repeat(4))));
                    }
                }
            }
        }

        (left_colors, left_lines, right_colors, right_lines)
    }

    fn generate_block_lines<'a>(&self, input_lines: &Vec<Line<'a>>, b: &Rect) -> Vec<Line<'a>> {
        input_lines.iter().skip(self.current_line).map(|line| {
            let span = &line.spans[0];
            let style = span.style;
            let mut content = span.content.clone().to_string();
            let len = content.len();

            if len <= self.current_col {
                return Line::from("");
            }
            
            if self.args.hex  {
                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)]
                .as_bytes()
                .chunks(self.args.width.unwrap_or(4))
                .map(|chunk| {
                    chunk.iter().map(|x| format!("{:0x}", x)).collect()
                })
                .collect::<Vec<String>>()
                .iter().map(|l| {
                    let mut s = String::new();
                    for _ in l.len()..self.args.width.unwrap_or(4) *  2 {
                        s.push('0');
                    }
                    format!("{}{}", s,  l)
                })
                .collect::<Vec<String>>().join(" ");
            }  else {
                content = content[self.current_col..std::cmp::min(len, self.current_col + b.width as usize)].to_string();
            }

            Line::styled(content, style)
        }).collect::<Vec<Line>>()
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
