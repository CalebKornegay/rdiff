use std::rc::Rc;

use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

pub struct Ui {
    layout: Rc<[Rect]>
}

impl Ui {
    pub fn new(frame: &Frame) -> Self {
        let args = std::env::args().filter(|s| s.len() > 0 && s.chars().next().unwrap() != '-').collect::<Vec<String>>();
        Self {
            layout: Layout::default()
                    .direction(Direction::Horizontal) // Arrange items horizontally
                    .constraints(vec![
                        Constraint::Percentage((100 / args.len()) as u16);
                        args.len()
                    ])
                    .split(frame.area())
        }
    }
}