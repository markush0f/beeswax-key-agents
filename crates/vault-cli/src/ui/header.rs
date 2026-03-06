use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
};

use crate::state::{AppState, Tab};

use super::common::{elide_middle, spinner_ascii};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, tick: u64) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[0]);

    let brand = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                " VAULT SCANNER ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("real-time secret monitor", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("[OpenAI]", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("[Gemini]", Style::default().fg(Color::Blue)),
        ]),
    ]);
    frame.render_widget(brand, top[0]);

    let stats_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(top[1]);

    let env_status = if state.env.done {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };
    let ide_status = if state.ides.done {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };

    let health = Paragraph::new(Line::from(vec![
        Span::styled(" ENV ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.env.done { " READY " } else { " SCAN " },
            env_status,
        ),
        Span::raw(" "),
        Span::styled(" IDES ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.ides.done { " READY " } else { " SCAN " },
            ide_status,
        ),
        Span::raw(" "),
        Span::styled(spinner_ascii(tick), Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Right);
    frame.render_widget(health, stats_rows[0]);

    let summary = Paragraph::new(Line::from(vec![
        Span::styled("ENV ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.env.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | IDES ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.ides.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | TOTAL ", Style::default().fg(Color::Gray)),
        Span::styled(
            (state.env.len() + state.ides.len()).to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Right);
    frame.render_widget(summary, stats_rows[1]);

    let scan_path = elide_middle(
        &state.scan_path,
        usize::from(rows[1].width).saturating_sub(14).max(8),
    );
    let path_line = Paragraph::new(Line::from(vec![
        Span::styled("SCAN PATH  ", Style::default().fg(Color::DarkGray)),
        Span::styled(scan_path, Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(path_line, rows[1]);

    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(rows[2]);

    let tabs = Tabs::new(vec![
        Line::from(format!(
            "  ENV ({})  {} {}  ",
            state.env.len(),
            if state.env.done { "DONE" } else { "SCAN" },
            if state.env.done {
                " "
            } else {
                spinner_ascii(tick)
            },
        )),
        Line::from(format!(
            "  IDES ({}) {} {}  ",
            state.ides.len(),
            if state.ides.done { "DONE" } else { "SCAN" },
            if state.ides.done {
                " "
            } else {
                spinner_ascii(tick)
            },
        )),
    ])
    .select(match state.tab {
        Tab::Env => 0,
        Tab::Ides => 1,
    })
    .highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .style(Style::default().fg(Color::DarkGray))
    .divider("  ");
    frame.render_widget(tabs, bottom[0]);

    let shortcuts = Paragraph::new(Line::from(vec![
        Span::styled("[E]", Style::default().fg(Color::Cyan)),
        Span::styled(" ENV  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[I]", Style::default().fg(Color::Cyan)),
        Span::styled(" IDES  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[TAB]", Style::default().fg(Color::Cyan)),
        Span::styled(" SWITCH  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Q]", Style::default().fg(Color::Cyan)),
        Span::styled(" QUIT", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(shortcuts, bottom[1]);
}
