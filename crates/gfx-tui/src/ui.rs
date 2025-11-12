use crate::app::App;
use crate::app::state::{MapState, PlayerAction, PopupState, ResourceType, State};
use crate::game_logic::{CellContent, MapCell};
use ratatui::{prelude::*, widgets::*};

pub fn render(app: &mut App, f: &mut Frame) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(7),  // Top bar
            Constraint::Percentage(93), // Bottom
        ])
        .split(f.area());

    let active_tab = match app.state {
        State::Map { .. } => 0,
        State::Admin => 1,
        State::Options => 2,
    };

    // Placeholder might not keep
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
    f.render_widget(tabs, main_chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Main game grid
            Constraint::Percentage(30), // Sidebar
        ])
        .split(main_chunks[1]);
    if let State::Map { .. } = &app.state {
        render_map_ui(app, f, &chunks);
    }
}

fn render_map_ui(app: &mut App, f: &mut Frame, chunks: &[Rect]) {
    // Left side - Game grid
    render_game_grid(app, f, chunks[0]);

    // Right side - Information panels
    render_sidebar(app, f, chunks);

    if app.state.is_popup() {
        render_popup(app, f);
    }
}

fn render_sidebar(app: &mut App, f: &mut Frame, chunks: &[Rect]) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Team info
            Constraint::Percentage(30), // Selected tile info
            Constraint::Percentage(20), // Messages
        ])
        .split(chunks[1]);

    render_team_info(f, sidebar_chunks[0]);
    render_tile_info(app, f, sidebar_chunks[1]);
    render_messages(app, f, sidebar_chunks[2]);
}

fn render_game_grid(app: &mut App, f: &mut Frame, area: Rect) {
    let map = match &app.state {
        State::Map { map, .. } => map,
        _ => return,
    };

    // Create rows for the table
    let rows: Vec<Row> = (0..map.y_max)
        .map(|i| {
            let cells: Vec<Cell> = (0..map.x_max)
                .map(|j| {
                    let cell = &map.cells[i * map.x_max + j];
                    let content = cell
                        .content
                        .iter()
                        .map(|content| match content {
                            CellContent::Rocks(rock) => format!("{:?}", rock),
                            CellContent::Food => content.to_string(),
                            CellContent::Player(player) => format!("P{}", player.id),
                            CellContent::Egg => content.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    let display_text = if content.is_empty() {
                        format!("\n[{},{}]\n", i, j)
                    } else {
                        format!("\n{}\n", content)
                    };
                    let text = Text::from(display_text).alignment(Alignment::Center);

                    Cell::from(text)
                })
                .collect();

            Row::new(cells).height(3)
        })
        .collect();

    // Calculate constraints for equal column widths
    let column_count = map.x_max;
    let width_per_column = (area.width as usize / column_count).max(5);
    let constraints: Vec<Constraint> = (0..column_count)
        .map(|_| Constraint::Length(width_per_column as u16))
        .collect();

    let selected_cell_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::Yellow);

    let table = Table::new(rows, constraints)
        .block(Block::default().title("Zappy World").borders(Borders::ALL))
        .highlight_spacing(HighlightSpacing::Always)
        .column_spacing(1)
        .row_highlight_style(Style::default().fg(Color::Red))
        .column_highlight_style(Style::default().fg(Color::Red))
        .cell_highlight_style(selected_cell_style)
        .bg(Color::Reset);

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_team_info(f: &mut Frame, area: Rect) {
    let teams_info = vec![
        ListItem::new("Team 1: Level 3"),
        ListItem::new("Team 2: Level 2"),
        // Add more teams as needed
    ];

    let teams_list = List::new(teams_info)
        .block(Block::default().title("Teams").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(teams_list, area);
}

fn render_tile_info(app: &App, f: &mut Frame, area: Rect) {
    let selected_info = match app.table_state.selected() {
        Some(row) => {
            let col = app.table_state.selected_column().unwrap_or(0);
            format!(
                "Selected: ({}, {})\nResources:\n- Food: 2\n- Linemate: 1",
                row, col
            )
        }
        None => "No tile selected".to_string(),
    };

    let paragraph = Paragraph::new(selected_info)
        .block(
            Block::default()
                .title("Tile Information")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_messages(app: &App, f: &mut Frame, area: Rect) {
    let messages = app.logs.join("\n");
    let paragraph = Paragraph::new(messages)
        .block(Block::default().title("Messages").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_popup(app: &App, f: &mut Frame) {
    if let State::Map {
        state: MapState::Selected {
            selected_cell,
            popup_state,
        },
        map,
        ..
    } = &app.state
    {
        let cell = &map.cells[selected_cell.0 * map.x_max + selected_cell.1];

        match popup_state {
            PopupState::MainMenu { selected_item } => render_main_menu(f, cell, *selected_item),
            PopupState::ResourceMenu {
                resource_type,
                current_amount,
            } => render_resource_menu(f, *resource_type, *current_amount),
            PopupState::PlayerMenu {
                player_id,
                selected_action,
            } => render_player_menu(f, cell, *player_id, *selected_action),
        }
    }
}

fn render_main_menu(f: &mut Frame, cell: &MapCell, selected_item: usize) {
    let area = centered_rect(30, 40, f.area());
    f.render_widget(Clear, area);

    // Create menu items from cell contents
    let mut items: Vec<(usize, String)> = Vec::with_capacity(20);
    let mut index = 0;

    // Add resources
    for content in &cell.content {
        match content {
            CellContent::Food => {
                items.push((index, content.to_string()));
                index += 1;
            }
            CellContent::Rocks(rock) => {
                items.push((index, rock.to_string()));
                index += 1;
            }
            CellContent::Player(player) => {
                items.push((index, player.to_string()));
                index += 1;
            }
            CellContent::Egg => {
                items.push((index, content.to_string()));
                index += 1;
            }
        }
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(i, text)| {
            let style = if *i == selected_item {
                Style::default().bg(Color::Red).fg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(&**text).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title("Tile Contents")
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        )
        .highlight_style(Style::default().bg(Color::Red));

    f.render_widget(list, area);
}

fn render_resource_menu(f: &mut Frame, resource_type: ResourceType, amount: u32) {
    let area = centered_rect(30, 30, f.area());
    f.render_widget(Clear, area);

    let content = vec![
        ListItem::new(format!("Current: {}", amount)),
        ListItem::new("[+] Increase"),
        ListItem::new("[-] Decrease"),
        ListItem::new("[B] Back"),
    ];

    let list = List::new(content).block(
        Block::default()
            .title(format!("{:?}", resource_type))
            .borders(Borders::ALL)
            .border_type(BorderType::Double),
    );

    f.render_widget(list, area);
}

fn render_player_menu(
    f: &mut Frame,
    cell: &MapCell,
    player_id: u32,
    selected_action: PlayerAction,
) {
    let area = centered_rect(40, 40, f.area());
    f.render_widget(Clear, area);

    let player = cell.content.iter().find_map(|content| {
        if let CellContent::Player(p) = content {
            if p.id == player_id { Some(p) } else { None }
        } else {
            None
        }
    });

    if let Some(player) = player {
        // Create menu items with proper highlighting based on selected_action
        let items = vec![
            ListItem::new(format!("Level: {}", player.level)),
            ListItem::new(format!("Orientation: {:?}", player.orientation)),
            ListItem::new(""),
            ListItem::new("[V] View Field of View").style(
                if matches!(selected_action, PlayerAction::ViewFOV) {
                    Style::default().bg(Color::Red).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
            ListItem::new("[I] View Inventory").style(
                if matches!(selected_action, PlayerAction::ViewInventory) {
                    Style::default().bg(Color::Red).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
            ListItem::new("[B] Back").style(if matches!(selected_action, PlayerAction::Back) {
                Style::default().bg(Color::Red).fg(Color::White)
            } else {
                Style::default()
            }),
        ];

        let list = List::new(items).block(
            Block::default()
                .title(format!("Player {}", player_id))
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        );

        f.render_widget(list, area);
    }
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
