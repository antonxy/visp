// VISP: VI-style SPreadsheet

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

    let mut content = TableContent{
        cells: vec![
            vec![TableCell::String("Value".to_string()), TableCell::Value(10), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Value(20), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Value(20), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Value(20), TableCell::Value(10)],
        ],
        col_widths: vec![10, 5],
        row_heights: vec![1, 2],
        selection: Selection {
            row: 0,
            col: 0,
            rows: 1,
            cols: 1,
        },
    };

    loop {
        terminal.draw(|f| ui(f, &content))?;

        // Wait up to 1s for another event
        if crossterm::event::poll(Duration::from_millis(1_000))? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let event = crossterm::event::read()?;

            if event == Event::Key(KeyCode::Char('j').into()) {
                content.selection.row += 1;
            }
            if event == Event::Key(KeyCode::Char('k').into()) {
                content.selection.row -= 1;
            }
            if event == Event::Key(KeyCode::Char('l').into()) {
                content.selection.col += 1;
            }
            if event == Event::Key(KeyCode::Char('h').into()) {
                content.selection.col -= 1;
            }

            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        }
    }

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

#[derive(Default)]
struct Selection {
    row: u16,
    col: u16,
    rows: u16,
    cols: u16,
}

impl Selection {
    fn selected(&self, row: u16, col: u16) -> bool {
        row >= self.row && row < self.row + self.rows
            && col >= self.col && col < self.col + self.cols
    }
}

struct TableContent {
    cells: Vec<Vec<TableCell>>, // row major
    col_widths: Vec<u16>,
    row_heights: Vec<u16>,
    selection: Selection
}

struct Table<'a> {
    content: &'a TableContent,
}

impl<'a> Widget for Table<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header_style = Style::default().add_modifier(Modifier::BOLD);
        let column_style = Style::default();
        let selected_column_style = Style::default().add_modifier(Modifier::BOLD);

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
                        let table_row : u16 = (row - 1).try_into().unwrap();
                        let table_col : u16 = (col - 1).try_into().unwrap();
                        // Table content
                        let content : Option<&TableCell> = self.content.cells.get(table_row as usize).and_then(|r| r.get(table_col as usize));
                        if let Some(c) = content {
                            //TODO overflow
                            let style = if self.content.selection.selected(table_row, table_col) {
                                selected_column_style
                            } else {
                                column_style
                            };
                            buf.set_string(x, y, c.format_string(), style);
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

    let table = Table {content};
    f.render_widget(table, chunks[0]);

    let command_line = Paragraph::new("Command");
    f.render_widget(command_line, chunks[1]);
}
