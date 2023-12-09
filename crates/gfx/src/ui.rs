use crate::app::state::State;
use crate::app::App;
use ratatui::{prelude::*, widgets::*};

/// Renders the user interface.
pub fn render(app: &mut App, f: &mut Frame) {
    let top_constraint = Constraint::Percentage(8);
    let middle_constraint = Constraint::Length(30);
    let bottom_constraint = Constraint::Min(0);

    let active_tab = match app.state {
        State::Map { .. } => 0,
        State::Admin => 1,
        State::Options => 2,
    };

    let sizes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([top_constraint, middle_constraint, bottom_constraint])
        .split(f.size());

    let block = Block::default();
    f.render_widget(block, sizes[0]);

    let titles = vec![
        Line::from(Span::styled("Map", Style::default().fg(Color::Red))),
        Line::from(Span::styled("Admin", Style::default().fg(Color::Red))),
        Line::from(Span::styled("Options", Style::default().fg(Color::Red))),
    ];

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(active_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::White));
    f.render_widget(tabs, sizes[0]);

    match active_tab {
        0 => render_map_tab(&mut app.state, f, sizes[1]),
        1 => render_admin_tab(app, f, sizes[1]),
        2 => render_options_tab(app, f, sizes[1]),
        _ => {}
    }
    if app.state.is_popup() {
        render_popup(1, f);
    }
    render_logs(app, f, sizes[2]);
}

/// Renders a grid of cells
fn render_map_tab(state: &mut State, f: &mut Frame, area: Rect) {
    let selected = state.selected_cell().unwrap_or((0, 0));
    let map = state.map_mut().unwrap();
    let column_width = (area.width / map.x_max as u16).max(1); // Ensure at least 1
    let constraints: Vec<Constraint> = (0..map.x_max)
        .map(|_| Constraint::Length(column_width))
        .collect();

    for (i, row) in map.cells.iter().enumerate() {
        let row_height = (area.height / map.y_max as u16).max(1); // Ensure at least 1
        let row_area = Rect {
            x: area.x,
            y: area.y + i as u16 * row_height,
            width: area.width,
            height: row_height,
        };

        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.clone())
            .split(row_area);

        for (j, _cell) in row.content.iter().enumerate() {
            let selected = selected == (i, j);
            let cell_style = Style::default()
                .fg(if selected { Color::Red } else { Color::White })
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            let content = Paragraph::new(String::new())
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true })
                .style(cell_style);

            f.render_widget(content, row_layout[j]);
        }
    }
}

fn render_logs(app: &mut App, f: &mut Frame, area: Rect) {
    let vertical_scroll = match app.state {
        State::Map {
            vertical_scroll, ..
        } => vertical_scroll,
        _ => 0,
    };
    let paragraph = Paragraph::new(app.logs.join("\n"))
        .scroll((vertical_scroll as u16, 0))
        .block(Block::default().title("Logs").borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

/// Renders a placeholder for the admin tab
fn render_admin_tab(_app: &mut App, f: &mut Frame, area: Rect) {
    // Example code: render a placeholder for the admin panel
    let paragraph = Paragraph::new("Admin content here")
        .block(Block::default().title("Admin").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

/// Renders a placeholder for the options tab
fn render_options_tab(_app: &mut App, f: &mut Frame, area: Rect) {
    // Example code: render a placeholder for the options
    let paragraph = Paragraph::new("Options content here")
        .block(Block::default().title("Options").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

/// Renders a popup for command selection
fn render_popup(popup_select: usize, f: &mut Frame) {
    let block = Block::default().title("Popup").borders(Borders::ALL);
    let area = centered_rect(20, 20, f.size());
    f.render_widget(Clear, area); //this clears out the background
    let list = List::new(vec![
        ListItem::new(Span::styled(
            "Command 1",
            Style::default().fg(if popup_select == 1 {
                Color::Red
            } else {
                Color::White
            }),
        )),
        ListItem::new(Span::styled(
            "Command 2",
            Style::default().fg(if popup_select == 2 {
                Color::Red
            } else {
                Color::White
            }),
        )),
        ListItem::new(Span::styled(
            "Command 3",
            Style::default().fg(if popup_select == 3 {
                Color::Red
            } else {
                Color::White
            }),
        )),
    ]);

    let list_area = area.inner(&Margin {
        vertical: 2,
        horizontal: 2,
    });
    f.render_widget(list, list_area);
    f.render_widget(block, area);
}

/// Returns a [`Rect`] centered in the given [`Rect`] with the given percentage size.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
