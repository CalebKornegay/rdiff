use std::rc::Rc;
use ratatui::{layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Style}, text::Line, widgets::{Block, Borders, Paragraph}, Frame};

pub struct Ui {
    pub boxes: Rc<[Rect]>
}

impl Ui {
    pub fn new(rect: Rect) -> Self {
        Self {
            boxes: Layout::default()
                    .direction(Direction::Horizontal) // Arrange items horizontally
                    .constraints(vec![
                        Constraint::Ratio(1, 2),
                        Constraint::Ratio(1, 2)
                    ])
                    .flex(Flex::Start)
                    .split(rect)
        }
    }

    pub fn show_help<'a>(frame: &mut Frame<'a>, keybinds_text: &Vec<&str>) {
        frame.render_widget(Paragraph::new("").block(generate_block(String::from("Help"))), frame.area());
                    
        let mut help_text = keybinds_text.clone().iter()
            .map(|&l| {
                if l == "[h] help" {
                    Line::from("[h] to exit this screen")
                } else {
                    Line::from(l)
                }
            })
            .collect::<Vec<Line>>();
        help_text.extend_from_slice(
            &[
                Line::from("[e] end of file"),
                Line::from("[b] begining of file"),
                Line::from("[\u{2195}] move up and down using arrow keys or mouse"),
                // Line::from("[\u{2194}] move left and right using arrow keys")
            ]
        );

        let b = Self::center_rect(frame.area(), Constraint::Length(help_text.iter().map(|l| l.width()).max().unwrap() as u16), Constraint::Length(help_text.len() as u16 + 2));
        frame.render_widget(
            Paragraph::new(help_text), b
        );
    }

        
    pub fn center_rect(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
        area
    }

    pub fn get_min_width(&self) -> u16 {
        self.boxes.iter().map(|&b| b.width).min().unwrap()
    }

    pub fn get_height(&self) -> usize {
        self.boxes.iter().map(|&b| b.height).min().unwrap() as usize
    }
}


pub fn generate_block<'a>(name: String) -> Block<'a> {
    Block::default()
        .title(name)
        .title_alignment(ratatui::layout::Alignment::Center)
        .title_style(Style::default().fg(Color::Rgb(0xff, 0xff, 0xff)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0x3a, 0x3a, 0x3a)))
        .style(Style::default().bg(Color::Rgb(0x12, 0x12, 0x12)))
}
