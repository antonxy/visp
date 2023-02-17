use std::{io, thread, time::Duration};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction, Rect},
    buffer::{Buffer},
    style::{Style, Modifier},
    Frame,
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let content = TableContent{
        cells: vec![
            vec![TableCell::String("Value".to_string()), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Value(20)],
        ],
        col_widths: vec![10, 5],
        row_heights: vec![1, 2],
    };

    terminal.draw(|f| ui(f, &content))?;

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

enum TableCell {
    String(String),
    Value(i32),
}

impl TableCell {
    fn format_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Value(v) => format!("{}", v),
        }
    }
}

struct TableContent {
    cells: Vec<Vec<TableCell>>, // row major
    col_widths: Vec<u16>,
    row_heights: Vec<u16>,
}

struct Table<'a> {
    content: &'a TableContent,
}

impl<'a> Widget for Table<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header_style = Style::default().add_modifier(Modifier::BOLD);
        let column_style = Style::default();

        let mut row = 0; //Table row col
        let mut y = area.y; //Buffer position

        while y < area.y + area.height {
            let mut row_height : u16 = self.content.row_heights.get(row - 1).map(|h| *h).unwrap_or(1);
            let mut col = 0;
            let mut x = area.x;
            while x < area.x + area.width {
                let mut col_width : u16 = self.content.col_widths.get(col - 1).map(|w| *w).unwrap_or(4);
                if row == 0 {
                    // Header row
                    if col == 0 {
                        buf.set_string(x, y, "**", header_style);
                    } else {
                        buf.set_string(x, y, format!("{}", col), header_style);
                    }
                } else {
                    if col == 0 {
                        // Header column
                        buf.set_string(x, y, format!("{}", row), header_style);
                    } else {
                        // Table content
                        let content : Option<&TableCell> = self.content.cells.get(row - 1).and_then(|r| r.get(col - 1));
                        if let Some(c) = content {
                            //TODO overflow
                            buf.set_string(x, y, c.format_string(), column_style);
                        }
                    }
                }
                x += col_width;
                col += 1;
            }

            row += 1;
            y += row_height;
        }
    }
}


fn ui<B: Backend>(f: &mut Frame<B>, content: &TableContent) {
   let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Max(10000),
                Constraint::Length(1),
            ].as_ref()
        )
        .split(f.size());

    let block = Block::default()
         .title("Block")
         .borders(Borders::ALL);
    let table = Table {content};
    f.render_widget(table, chunks[0]);

    let command_line = Paragraph::new("Command");
    f.render_widget(command_line, chunks[1]);
}
