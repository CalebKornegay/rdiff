use std::rc::Rc;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

pub struct Ui {
    pub boxes: Rc<[Rect]>
}

impl Ui {
    pub fn new(frame: &Frame, num_boxes: u8) -> Self {
        Self {
            boxes: Layout::default()
                    .direction(Direction::Horizontal) // Arrange items horizontally
                    .constraints(vec![
                        Constraint::Percentage((100u8 / num_boxes) as u16);
                        num_boxes as usize
                    ])
                    .split(frame.area())
        }
    }
}