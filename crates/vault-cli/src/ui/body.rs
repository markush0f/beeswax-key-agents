use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, Gauge, List, ListItem,
        ListState as TuiListState, Paragraph, Wrap,
    },
};

use crate::state::{AppState, Tab};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
        .split(area);

    render_findings_panel(frame, state, cols[0]);
    render_side_panel(frame, state, cols[1]);
}

fn render_findings_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let (items, selected, scroll, base_title, accent) = match state.tab {
        Tab::Env => (
            &state.env.items,
            state.env.selected(),
            state.env.scroll(),
            "ENV Findings",
            Color::Green,
        ),
        Tab::Ides => (
            &state.ides.items,
            state.ides.selected(),
            state.ides.scroll(),
            "IDES Findings",
            Color::Magenta,
        ),
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" {}  [{} items] ", base_title, items.len()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let view_h = usize::from(inner.height);
    let start = scroll.min(items.len());
    let end = (start + view_h).min(items.len());
    let visible = &items[start..end];

    let list_items: Vec<ListItem> = if visible.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "Waiting for results...",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        visible
            .iter()
            .map(|m| ListItem::new(render_match_line(m)))
            .collect()
    };

    let mut list_state = TuiListState::default();
    if !visible.is_empty() {
        let in_view = selected.saturating_sub(start).min(visible.len() - 1);
        list_state.select(Some(in_view));
    }

    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_side_panel(frame: &mut Frame, state: &AppState, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Min(6),
        ])
        .split(area);

    render_selected_card(frame, state, rows[0]);
    render_stats_card(frame, state, rows[1]);
    render_provider_card(frame, state, rows[2]);
}

fn render_selected_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Selected Item ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active = state.active_list();
    if active.is_empty() {
        frame.render_widget(
            Paragraph::new("No selected item")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Left),
            inner,
        );
        return;
    }

    let idx = active.selected().min(active.items.len().saturating_sub(1));
    let item = &active.items[idx];

    let lines = vec![
        Line::from(vec![
            Span::styled("Provider: ", Style::default().fg(Color::Gray)),
            Span::styled(item.provider.clone(), provider_style(&item.provider)),
        ]),
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::Gray)),
            Span::raw(item.file_path.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Line: ", Style::default().fg(Color::Gray)),
            Span::raw(item.line_number.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if item.hardcoded {
                    "HARDCODED"
                } else {
                    "POSSIBLE REF"
                },
                if item.hardcoded {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("Key: ", Style::default().fg(Color::Gray)),
            Span::raw(item.key.clone()),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_stats_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_list();
    let total = active.len();
    let hardcoded = active.items.iter().filter(|m| m.hardcoded).count();
    let refs = total.saturating_sub(hardcoded);

    let ratio = if total == 0 {
        0.0
    } else {
        hardcoded as f64 / total as f64
    };

    let border_color = if ratio >= 0.5 {
        Color::Red
    } else if ratio > 0.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let block = Block::default()
        .title(Span::styled(
            " Risk & Volume ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(format!(
            "Total {} | Hardcoded {} | Ref {}",
            total, hardcoded, refs
        )))
        .style(Style::default().fg(Color::White)),
        rows[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Scanner: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if active.done { "DONE" } else { "RUNNING" },
                Style::default()
                    .fg(if active.done {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        rows[1],
    );

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(if ratio >= 0.5 {
                Color::Red
            } else {
                Color::Yellow
            }))
            .ratio(ratio)
            .label(format!("{:.0}% hardcoded", ratio * 100.0)),
        rows[2],
    );
}

fn render_provider_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Providers ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active = state.active_list();
    let mut openai = 0u64;
    let mut gemini = 0u64;
    let mut anthropic = 0u64;
    let mut other = 0u64;

    for item in &active.items {
        let provider = item.provider.to_ascii_lowercase();
        if provider.contains("openai") {
            openai += 1;
        } else if provider.contains("gemini") {
            gemini += 1;
        } else if provider.contains("anthropic") {
            anthropic += 1;
        } else {
            other += 1;
        }
    }

    let bars = [
        Bar::default()
            .value(openai)
            .label("OpenAI".into())
            .style(Style::default().fg(Color::Green))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        Bar::default()
            .value(gemini)
            .label("Gemini".into())
            .style(Style::default().fg(Color::Blue))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        Bar::default()
            .value(anthropic)
            .label("Anthro".into())
            .style(Style::default().fg(Color::Yellow))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        Bar::default()
            .value(other)
            .label("Other".into())
            .style(Style::default().fg(Color::Gray))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    ];

    let max = openai.max(gemini).max(anthropic).max(other).max(1);
    let width = if inner.width >= 42 {
        6
    } else if inner.width >= 34 {
        5
    } else {
        3
    };

    let chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(width)
        .bar_gap(1)
        .max(max)
        .label_style(Style::default().fg(Color::Gray))
        .value_style(Style::default().fg(Color::White));
    frame.render_widget(chart, inner);
}

fn render_match_line(m: &vault_core::KeyMatch) -> Line<'static> {
    let provider = Span::styled(format!("[{}]", m.provider), provider_style(&m.provider));

    let hardcoded = if m.hardcoded {
        Span::styled(
            "[HARDCODED]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("[POSSIBLE REF]", Style::default().fg(Color::Yellow))
    };

    let rest = format!(" {}:{}  {}", m.file_path.display(), m.line_number, m.key);
    Line::from(vec![provider, Span::raw(" "), hardcoded, Span::raw(rest)])
}

fn provider_style(provider: &str) -> Style {
    let provider = provider.to_ascii_lowercase();
    if provider.contains("openai") {
        return Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
    }
    if provider.contains("gemini") {
        return Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD);
    }
    if provider.contains("anthropic") {
        return Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
    }
    Style::default().fg(Color::Gray)
}
