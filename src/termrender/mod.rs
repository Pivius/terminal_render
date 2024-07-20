use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub struct Term {
    term: Terminal<CrosstermBackend<io::Stdout>>,
    width: usize,
    height: usize,
}

impl Term {
    pub fn new() -> Self {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
        execute!(stdout, cursor::Hide).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("Failed to initialize terminal");

        Self {
            term: terminal,
            width: 80,
            height: 24,
        }
    }

    pub fn reset(&mut self) {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
        execute!(stdout, cursor::Hide).unwrap();
    }

    pub fn clear(&mut self) {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
    }

    pub fn draw_text(&mut self, text: &str, x: usize, y: usize, color: Color) {
        let text_color = color;
        let style = Style::default().fg(text_color);
        
        self.term.draw(|f| {
            let size = f.size();
            let paragraph = Paragraph::new(Span::styled(text, style))
                .block(Block::default().borders(Borders::NONE))
                .alignment(tui::layout::Alignment::Left);
            let area = tui::layout::Rect::new(x as u16, y as u16, size.width, size.height);
            f.render_widget(paragraph, area);
        }).unwrap();
    }
}