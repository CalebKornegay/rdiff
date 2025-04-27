use std::rc::Rc;
use ratatui::{layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Style}, text::Line, widgets::{Block, Borders, Paragraph}, Frame};

pub struct Ui {
    pub boxes: Rc<[Rect]>
}

impl Ui {
    pub fn new(frame: &Frame, num_boxes: u8) -> Self {
        Self {
            boxes: Layout::default()
                    .direction(Direction::Horizontal) // Arrange items horizontally
                    .constraints(vec![
                        // Constraint::Percentage((100f64 / num_boxes as f64) as u16);
                        Constraint::Ratio(1, num_boxes as u32);
                        num_boxes as usize
                    ])
                    .flex(Flex::Start)
                    .split(frame.area())
        }
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

pub fn generate_line_numbers<'a>(current_line: usize, height: usize) -> Paragraph<'a> {
    Paragraph::new((current_line..current_line + height).map(|i| {
        Line::styled(i.to_string(), Style::default().fg(Color::Rgb(0x12, 0x12, 0x12)))
    }).collect::<Vec<Line>>()).block(generate_block(String::new())).left_aligned()
}
