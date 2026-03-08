use std::collections::HashMap;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, List, ListItem,
        ListState as TuiListState, Paragraph, Wrap,
    },
};

use crate::state::{AppState, Tab};

/// Renders the entire body area, splitting it into the findings list and the side panel.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to draw into.
/// * `state` - Current application state (active tab, match lists, selection).
/// * `area` - The rectangular region allocated to the body by the root layout.
pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
        .split(area);

    render_findings_panel(frame, state, cols[0]);
    render_side_panel(frame, state, cols[1]);
}

/// Renders the scrollable list of secret matches for the active tab.
///
/// Applies alternating row background colors, a bold highlighted row for the
/// selected item, and an accent color that changes with the active tab.
/// Shows a placeholder message while no results have arrived yet.
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
        Tab::Files => (
            &state.files.items,
            state.files.selected(),
            state.files.scroll(),
            "FILES (hardcoded)",
            Color::Cyan,
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
            .enumerate()
            .map(|(i, m)| {
                let base = if i % 2 == 0 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(render_match_line(m)).style(base)
            })
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
                .bg(accent)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, inner, &mut list_state);
}

/// Renders the right-side vertical stack of three info cards.
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

/// Renders a card displaying the full details of the currently selected match.
///
/// Shows scope, provider (with its registered color), file path, line number, and
/// masked key. If the list is empty, shows a "No selected item" placeholder.
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

    let scope = match state.tab {
        Tab::Env => "ENV",
        Tab::Ides => "IDES",
        Tab::Files => "FILES",
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Scope: ", Style::default().fg(Color::Gray)),
            Span::styled(
                scope,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
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
            Span::styled("Key: ", Style::default().fg(Color::Gray)),
            Span::raw(item.key.clone()),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

/// Renders a compact summary card showing total findings and scanner status.
///
/// The status label reads `SCANNING` (yellow) while the background thread is active
/// and switches to `DONE` (green) once the channel closes.
fn render_stats_card(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_list();
    let total = active.len();

    let block = Block::default()
        .title(Span::styled(
            " Scan Summary ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(format!("Total findings {}", total)))
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
}

/// Renders a horizontal bar chart grouped by secret provider.
///
/// Match counts are aggregated dynamically by iterating over [`vault_core::patterns::get_patterns`].
/// Each bar uses the `(r, g, b)` color registered on the corresponding [`SecretPattern`].
/// Unrecognized providers are counted under an "Other" bar.
///
/// The bar width adapts responsively to the panel width:
/// - ≥ 50 cols → 5-wide bars
/// - ≥ 40 cols → 4-wide bars
/// - < 40 cols → 3-wide bars
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
    let patterns = vault_core::patterns::get_patterns();

    // Count occurrences dynamically
    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut other_count = 0u64;

    for item in &active.items {
        let provider = item.provider.as_str();
        let matched = patterns.iter().find(|p| provider.contains(p.name));

        if let Some(p) = matched {
            *counts.entry(p.name.to_string()).or_insert(0) += 1;
        } else {
            other_count += 1;
        }
    }

    // Build bars dynamically
    let mut bars: Vec<Bar> = Vec::new();
    let mut max_count = 1u64;

    for p in &patterns {
        let count = *counts.get(p.name).unwrap_or(&0);
        max_count = max_count.max(count);

        bars.push(
            Bar::default()
                .value(count)
                .label(p.short_name.into())
                .style(Style::default().fg(Color::Rgb(p.color.0, p.color.1, p.color.2)))
                .value_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
        );
    }

    max_count = max_count.max(other_count);
    bars.push(
        Bar::default()
            .value(other_count)
            .label("Other".into())
            .style(Style::default().fg(Color::Gray))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    let width = if inner.width >= 50 {
        5
    } else if inner.width >= 40 {
        4
    } else {
        3
    };

    let chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(width)
        .bar_gap(1)
        .max(max_count)
        .label_style(Style::default().fg(Color::Gray))
        .value_style(Style::default().fg(Color::White));
    frame.render_widget(chart, inner);
}

/// Formats a single [`KeyMatch`] into a colored [`Line`] for the findings list.
///
/// The provider name is rendered in its registered pattern color, followed by the
/// file path, line number, and masked key value.
fn render_match_line(m: &vault_core::KeyMatch) -> Line<'static> {
    let provider = Span::styled(format!("[{}]", m.provider), provider_style(&m.provider));
    let rest = format!(" {}:{}  {}", m.file_path.display(), m.line_number, m.key);
    Line::from(vec![provider, Span::raw(rest)])
}

/// Returns the ratatui [`Style`] registered for the given provider name.
///
/// Looks up the provider in [`vault_core::patterns::get_patterns`] by name substring match
/// and returns its RGB color with bold modifier. Falls back to `Color::Gray` for unknown providers.
///
/// # Arguments
///
/// * `provider_str` - The `provider` field from a [`KeyMatch`].
pub(crate) fn provider_style(provider_str: &str) -> Style {
    let patterns = vault_core::patterns::get_patterns();

    if let Some(p) = patterns.iter().find(|p| provider_str.contains(p.name)) {
        return Style::default()
            .fg(Color::Rgb(p.color.0, p.color.1, p.color.2))
            .add_modifier(Modifier::BOLD);
    }

    Style::default().fg(Color::Gray)
}
