use crate::datapoint::Datapoint;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub struct App {
    pub datapoints: Vec<Datapoint>,
    pub table_state: TableState,
    pub server_info: String,
    pub scan_interval: u64,
}

impl App {
    pub fn new(server_info: String, scan_interval: u64) -> Self {
        Self {
            datapoints: Vec::new(),
            table_state: TableState::default(),
            server_info,
            scan_interval,
        }
    }

    pub fn update_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        self.datapoints = datapoints;
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.datapoints.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.datapoints.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    // Check if we need to show error details or bitfield details
    let (show_error_details, show_bitfield_details) = if let Some(idx) = app.table_state.selected() {
        if let Some(dp) = app.datapoints.get(idx) {
            (dp.error.is_some(), dp.get_bitfield_status().is_some())
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };
    
    let show_details = show_error_details || show_bitfield_details;
    
    let chunks = if show_details {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(if show_bitfield_details { 10 } else { 4 }),
                Constraint::Length(3),
            ])
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area())
    };

    draw_header(f, chunks[0], app);
    draw_datapoints_table(f, chunks[1], app);
    
    if show_details {
        if show_error_details {
            draw_error_details(f, chunks[2], app);
        } else if show_bitfield_details {
            draw_bitfield_details(f, chunks[2], app);
        }
        draw_footer(f, chunks[3]);
    } else {
        draw_footer(f, chunks[2]);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let error_count = app.datapoints.iter().filter(|dp| dp.error.is_some()).count();
    let ok_count = app.datapoints.iter().filter(|dp| dp.value.is_some()).count();
    let total = app.datapoints.len();
    
    let status = if error_count == total && total > 0 {
        Span::styled(" [ALL ERRORS]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else if error_count > 0 {
        Span::styled(format!(" [{} errors]", error_count), Style::default().fg(Color::Yellow))
    } else if ok_count > 0 {
        Span::styled(" [Connected]", Style::default().fg(Color::Green))
    } else {
        Span::styled(" [Waiting...]", Style::default().fg(Color::Gray))
    };
    
    let title = Line::from(vec![
        Span::raw(format!("Datapoint Monitor - {} | Scan Interval: {}ms", 
            app.server_info, app.scan_interval)),
        status,
    ]);
    
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn draw_datapoints_table(f: &mut Frame, area: Rect, app: &mut App) {
    let header_cells = ["Name", "Address", "Type", "Value", "Status", "Last Updated"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.datapoints.iter().map(|dp| {
        let (status, status_color, value_str, data_type) = if let Some(ref error) = dp.error {
            // Parse error message to show friendly status
            let status_text = if error.contains("Connection timeout") {
                "TIMEOUT"
            } else if error.contains("Connection failed") || error.contains("Connection refused") {
                "CONN FAIL"
            } else if error.contains("Read timeout") {
                "READ TMO"
            } else if error.contains("Modbus exception") {
                "MODBUS ERR"
            } else if error.contains("Read error") {
                "READ ERR"
            } else {
                "ERROR"
            };
            (status_text, Color::Red, "-".to_string(), "-".to_string())
        } else if let Some(ref value) = dp.value {
            ("OK", Color::Green, value.to_string(), value.type_name().to_string())
        } else {
            ("WAITING", Color::Gray, "-".to_string(), "-".to_string())
        };

        let last_updated = dp
            .last_updated
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "-".to_string());

        let cells = vec![
            Cell::from(dp.name.clone()),
            Cell::from(format!("{}", dp.address)),
            Cell::from(data_type).style(Style::default().fg(Color::Cyan)),
            Cell::from(value_str),
            Cell::from(status).style(Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Cell::from(last_updated),
        ];

        Row::new(cells).height(1)
    });

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(10),
        Constraint::Percentage(8),
        Constraint::Percentage(20),
        Constraint::Percentage(12),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Datapoints"))
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_error_details(f: &mut Frame, area: Rect, app: &App) {
    if let Some(idx) = app.table_state.selected() {
        if let Some(dp) = app.datapoints.get(idx) {
            if let Some(ref error) = dp.error {
                let error_text = vec![
                    Line::from(vec![
                        Span::styled("Error Details: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw(error.as_str()),
                    ]),
                ];
                let error_widget = Paragraph::new(error_text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Details")
                        .style(Style::default().fg(Color::Red)));
                f.render_widget(error_widget, area);
                return;
            }
        }
    }
}

fn draw_bitfield_details(f: &mut Frame, area: Rect, app: &App) {
    if let Some(idx) = app.table_state.selected() {
        if let Some(dp) = app.datapoints.get(idx) {
            if let Some(bitfield_status) = dp.get_bitfield_status() {
                let mut lines = vec![];
                for (bit, name, is_set) in bitfield_status {
                    let status_char = if is_set { "✓" } else { "✗" };
                    let color = if is_set { Color::Green } else { Color::Gray };
                    lines.push(Line::from(vec![
                        Span::styled(format!("  Bit {:2}: ", bit), Style::default().fg(Color::Yellow)),
                        Span::styled(status_char, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        Span::raw(" "),
                        Span::raw(name),
                    ]));
                }
                
                let bitfield_widget = Paragraph::new(lines)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Bitfield Details")
                        .style(Style::default().fg(Color::Cyan)));
                f.render_widget(bitfield_widget, area);
                return;
            }
        }
    }
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = vec![
        Span::raw("Controls: "),
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate | "),
        Span::styled("q/Esc/Ctrl+C", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ];
    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}
