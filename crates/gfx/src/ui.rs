use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn render(app: &mut App, f: &mut Frame) {
    let sizes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    let block = Block::default();
    f.render_widget(block, sizes[0]);

    // Titles for the tabs
    let titles = vec![
        Line::from(Span::styled("Map", Style::default().fg(Color::Red))),
        Line::from(Span::styled("Admin", Style::default().fg(Color::Red))),
        Line::from(Span::styled("Options", Style::default().fg(Color::Red))),
    ];

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.active_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::White));
    f.render_widget(tabs, sizes[0]);

    // Match the selected tab and render the corresponding content
    match app.active_tab {
        0 => render_map_tab(app, f, sizes[1]),
        1 => render_admin_tab(app, f, sizes[1]),
        2 => render_options_tab(app, f, sizes[1]),
        _ => {}  // It's always a good practice to handle the default case
    }
    if let Some(popup) = &app.popup {
        render_popup(popup.selected, f);
    }
}


fn render_map_tab(app: &mut App, f: &mut Frame, area: Rect) {
    // Calculate column width, assuming all columns are the same width for simplicity
    let column_width = (area.width / app.grid[0].len() as u16).max(1); // Ensure at least 1
    let constraints: Vec<Constraint> = app.grid[0]
        .iter()
        .map(|_| Constraint::Length(column_width))
        .collect();

    for (i, row) in app.grid.iter().enumerate() {
        // Calculate the row height, assuming all rows are the same height for simplicity
        let row_height = (area.height / app.grid.len() as u16).max(1); // Ensure at least 1
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

        for (j, &cell) in row.iter().enumerate() {
            let selected = app.selected_position == (i, j);
            let cell_style = Style::default()
                .fg(if selected { Color::Red } else { Color::White })
                .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() });
            let content = Paragraph::new(format!("{}", cell))
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true })
                .style(cell_style);

            f.render_widget(content, row_layout[j]);
        }
    }
}

fn render_admin_tab(app: &mut App, f: &mut Frame, area: Rect) {
    // Example code: render a placeholder for the admin panel
    let paragraph = Paragraph::new("Admin content here")
        .block(Block::default().title("Admin").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_options_tab(app: &mut App, f: &mut Frame, area: Rect) {
    // Example code: render a placeholder for the options
    let paragraph = Paragraph::new("Options content here")
        .block(Block::default().title("Options").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_popup(popup_select: usize, f: &mut Frame) {
    let block = Block::default().title("Popup").borders(Borders::ALL);
    let area = centered_rect(20, 20, f.size());
    f.render_widget(Clear, area); //this clears out the background
    let list = List::new(vec![
        ListItem::new(Span::styled(
            "Command 1",
            Style::default().fg(if popup_select == 1 { Color::Red } else { Color::White }),
        )),
        ListItem::new(Span::styled(
            "Command 2",
            Style::default().fg(if popup_select == 2 { Color::Red } else { Color::White }),
        )),
        ListItem::new(Span::styled(
            "Command 3",
            Style::default().fg(if popup_select == 3 { Color::Red } else { Color::White }),
        )),
    ]).highlight_style(Style::default().add_modifier(if popup_select == 1 { Modifier::BOLD } else { Modifier::empty() }))
        .highlight_style(Style::default().add_modifier(if popup_select == 2 { Modifier::BOLD } else { Modifier::empty() }))
        .highlight_style(Style::default().add_modifier(if popup_select == 3 { Modifier::BOLD } else { Modifier::empty() }));


    let listarea = area.inner(&Margin {
        vertical: 2,
        horizontal: 2,
    });
    f.render_widget(list, listarea);
    f.render_widget(block, area);
}

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