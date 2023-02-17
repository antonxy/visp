// VISP: VI-style SPreadsheet

use std::{io, time::Duration};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    widgets::{Widget, Paragraph},
    layout::{Layout, Constraint, Direction, Rect},
    buffer::{Buffer},
    style::{Style, Modifier, Color},
    Frame,
    Terminal
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn col_nr_to_label(col: u16) -> String {
    if col < 26 {
        char::from_u32('A' as u32 + col as u32).unwrap().to_string()
    } else {
        let front = col / 26;
        col_nr_to_label(front - 1) + &col_nr_to_label(col - (26 * front))
    }
}

fn add_clamp(val: &mut u16) {
    if *val < u16::MAX {
        *val += 1;
    }
}

fn sub_clamp(val: &mut u16, min: u16) {
    if *val > min {
        *val -= 1;
    }
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let table_content = TableContent{
        cells: vec![
            vec![TableCell::String("Value".to_string()), TableCell::Value(10), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Value(20), TableCell::Value(10)],
            vec![TableCell::String("Value".to_string()), TableCell::Empty, TableCell::Value(10)],
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

    let mut state = AppState {
        table_content,
        mode: AppMode::Normal,
    };

    loop {
        terminal.draw(|f| ui(f, &state))?;

        // Wait up to 1s for another event
        if crossterm::event::poll(Duration::from_millis(1_000))? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let event = crossterm::event::read()?;

            if state.mode == AppMode::Normal {
                if event == Event::Key(KeyCode::Char('j').into()) {
                    add_clamp(&mut state.table_content.selection.row);
                }
                if event == Event::Key(KeyCode::Char('k').into()) {
                    sub_clamp(&mut state.table_content.selection.row, 0);
                }
                if event == Event::Key(KeyCode::Char('l').into()) {
                    add_clamp(&mut state.table_content.selection.col);
                }
                if event == Event::Key(KeyCode::Char('h').into()) {
                    sub_clamp(&mut state.table_content.selection.col, 0);
                }
            } else if state.mode == AppMode::Visual {
                if event == Event::Key(KeyCode::Char('j').into()) {
                    add_clamp(&mut state.table_content.selection.rows);
                }
                if event == Event::Key(KeyCode::Char('k').into()) {
                    sub_clamp(&mut state.table_content.selection.rows, 1);
                }
                if event == Event::Key(KeyCode::Char('l').into()) {
                    add_clamp(&mut state.table_content.selection.cols);
                }
                if event == Event::Key(KeyCode::Char('h').into()) {
                    sub_clamp(&mut state.table_content.selection.cols, 1);
                }
            }

            if event == Event::Key(KeyCode::Esc.into()) {
                state.mode = AppMode::Normal;
                state.table_content.selection.set_single();
            }
            if event == Event::Key(KeyCode::Char('v').into()) {
                state.mode = AppMode::Visual;
            }

            if event == Event::Key(KeyCode::Char('q').into()) {
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

struct AppState {
    table_content: TableContent,
    mode: AppMode,
}

#[derive(PartialEq)]
enum AppMode {
    Normal,
    Visual
}

enum TableCell {
    Empty,
    String(String),
    Value(i32),
}

impl TableCell {
    fn format_string(&self) -> String {
        match self {
            Self::Empty => "".to_string(),
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
    fn set_single(&mut self) {
        self.rows = 1;
        self.cols = 1;
    }

    fn row_selected(&self, row: u16) -> bool {
        row >= self.row && row < self.row + self.rows
    }

    fn col_selected(&self, col: u16) -> bool {
        col >= self.col && col < self.col + self.cols
    }

    fn selected(&self, row: u16, col: u16) -> bool {
        self.row_selected(row) && self.col_selected(col)
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
        let column_style = Style::default();
        let selected_column_style = Style::default().fg(Color::White).bg(Color::Black);

        let header_style = column_style.add_modifier(Modifier::BOLD);
        let selected_header_style = selected_column_style.add_modifier(Modifier::BOLD);

        let draw_cell = |buf: &mut Buffer, cell: Option<&TableCell>, rect: Rect, selected: bool| {
            let style = if selected {
                selected_column_style
            } else {
                column_style
            };
            for x in rect.x..rect.x + rect.width {
                for y in rect.y..rect.y + rect.height {
                    buf.get_mut(x, y).set_char(' ').set_style(style);
                }
            }
            if let Some(c) = cell {
                buf.set_stringn(rect.x, rect.y, c.format_string(), rect.width as usize, style);
            }
        };

        let mut row = 0; 
        let mut y = area.y; //Buffer position

        while y < area.y + area.height {
            let table_row = if row == 0 { None } else { Some(row - 1) };
            let row_height : u16 = table_row.and_then(|r| self.content.row_heights.get(r)).map(|h| *h).unwrap_or(1);

            let mut col = 0;
            let mut x = area.x;
            while x < area.x + area.width {
                let table_col = if col == 0 { None } else { Some(col - 1) };
                let col_width : u16 = table_col.and_then(|c| self.content.col_widths.get(c)).map(|c| *c).unwrap_or(4);

                if let Some(table_row) = table_row {
                    if let Some(table_col) = table_col {
                        // Table content
                        let cell : Option<&TableCell> = self.content.cells.get(table_row).and_then(|r| r.get(table_col));
                        let selected = self.content.selection.selected(table_row as u16, table_col as u16);
                        draw_cell(buf, cell, Rect::new(x, y, col_width, row_height).intersection(area), selected);
                    } else {
                        // Header column
                        let style = if self.content.selection.row_selected(table_row as u16) {
                            selected_header_style
                        } else {
                            header_style
                        };
                        buf.set_string(x, y, format!("{}", row), style);
                    }

                } else {
                    // Header row
                    if let Some(table_col) = table_col {
                        let style = if self.content.selection.col_selected(table_col as u16) {
                            selected_header_style
                        } else {
                            header_style
                        };
                        buf.set_string(x, y, col_nr_to_label(table_col as u16), style);
                    } else {
                        buf.set_string(x, y, "**", header_style);
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


fn ui<B: Backend>(f: &mut Frame<B>, state: &AppState) {
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

    let table = Table {content: &state.table_content};
    f.render_widget(table, chunks[0]);

    let command_line = Paragraph::new("Command");
    f.render_widget(command_line, chunks[1]);
}
