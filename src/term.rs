use std::{io, thread, time::Duration};
use std::io::{Write, stdout};

pub struct Terminal {
    title: String
}

impl Terminal {
    pub fn new(title: String) -> Self {
        Self {
            title
        }
    }
//, x: usize, y: usize, color: Color

}