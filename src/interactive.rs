use std::collections::HashSet;
use std::io::{self, IsTerminal};
use std::path::Path;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::error::{InstallerError, Result};
use crate::install::{find_existing_destinations, install};
use crate::providers::{
    detect_providers, is_agents_provider, parse_providers_csv, supported_providers,
};
use crate::types::{
    InstallMethod, InstallRequest, InstallResult, InstallSkillArgs, ProviderId, Scope, SkillSource,
};

#[derive(Debug, Clone)]
pub struct InteractiveProviderSelectionOptions<'a> {
    pub project_root: Option<&'a Path>,
    pub candidates: Option<Vec<ProviderId>>,
    pub defaults: Option<Vec<ProviderId>>,
    pub message: &'a str,
}

impl<'a> Default for InteractiveProviderSelectionOptions<'a> {
    fn default() -> Self {
        Self {
            project_root: None,
            candidates: None,
            defaults: None,
            message: "Select providers to install to",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InteractiveProviderSelection {
    pub universal_locked: Vec<ProviderId>,
    pub selectable: Vec<ProviderId>,
    pub selected: Vec<ProviderId>,
}

#[derive(Debug)]
struct UiState {
    query: String,
    cursor: usize,
    selected: HashSet<ProviderId>,
    scroll_offset: usize,
}

pub fn prompt_provider_selection(
    options: InteractiveProviderSelectionOptions<'_>,
) -> Result<InteractiveProviderSelection> {
    let candidates = resolve_candidates(&options);
    let universal_locked = candidates
        .iter()
        .copied()
        .filter(|p| is_agents_provider(*p))
        .collect::<Vec<_>>();
    let selectable = candidates
        .iter()
        .copied()
        .filter(|p| !is_agents_provider(*p))
        .collect::<Vec<_>>();

    if selectable.is_empty() {
        let selected = if universal_locked.is_empty() {
            Vec::new()
        } else {
            vec![ProviderId::Universal]
        };
        return Ok(InteractiveProviderSelection {
            universal_locked,
            selectable,
            selected,
        });
    }

    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(InstallerError::PromptError {
            message: "interactive provider selection requires a TTY".to_string(),
        });
    }

    let default_selected = resolve_defaults(&options, &selectable);

    println!("{}", options.message);

    let mut state = UiState {
        query: String::new(),
        cursor: 0,
        selected: default_selected,
        scroll_offset: 0,
    };

    let mut terminal =
        setup_terminal(VIEWPORT_HEIGHT).map_err(|err| InstallerError::PromptError {
            message: err.to_string(),
        })?;

    let mut viewport_bottom = VIEWPORT_HEIGHT;
    let result = run_ui_loop(
        &mut terminal,
        &universal_locked,
        &selectable,
        &mut state,
        &mut viewport_bottom,
    );

    restore_terminal(&mut terminal).map_err(|err| InstallerError::PromptError {
        message: err.to_string(),
    })?;
    move_cursor_below_viewport(viewport_bottom);

    match result {
        Ok(mut selected) => {
            if !universal_locked.is_empty() {
                selected.push(ProviderId::Universal);
            }
            Ok(InteractiveProviderSelection {
                universal_locked,
                selectable,
                selected,
            })
        }
        Err(e) => Err(e),
    }
}

fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    universal_locked: &[ProviderId],
    selectable: &[ProviderId],
    state: &mut UiState,
    viewport_bottom: &mut u16,
) -> Result<Vec<ProviderId>> {
    loop {
        let filtered = filtered_items(selectable, &state.query);
        if state.cursor >= filtered.len() && !filtered.is_empty() {
            state.cursor = filtered.len() - 1;
        }

        let term_width = terminal.size().map(|s| s.width).unwrap_or(80);
        let viewport_height = terminal
            .size()
            .map(|s| s.height.min(VIEWPORT_HEIGHT))
            .unwrap_or(VIEWPORT_HEIGHT);
        let viewport_area = Rect::new(0, 0, term_width, viewport_height);
        let list_height = compute_layout(viewport_area, universal_locked.len())[6].height as usize;
        adjust_scroll(state, filtered.len(), list_height);

        let completed = terminal
            .draw(|frame| draw_ui(frame, universal_locked, &filtered, state))
            .map_err(|err| InstallerError::PromptError {
                message: err.to_string(),
            })?;
        *viewport_bottom = completed.area.bottom();

        let event = event::read().map_err(|err| InstallerError::PromptError {
            message: err.to_string(),
        })?;

        let Event::Key(key) = event else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up => state.cursor = state.cursor.saturating_sub(1),
            KeyCode::Down => {
                if !filtered.is_empty() {
                    state.cursor = (state.cursor + 1).min(filtered.len() - 1);
                }
            }
            KeyCode::Char(' ') => {
                if let Some(provider) = filtered.get(state.cursor).copied() {
                    if state.selected.contains(&provider) {
                        state.selected.remove(&provider);
                    } else {
                        state.selected.insert(provider);
                    }
                }
            }
            KeyCode::Backspace => {
                state.query.pop();
                state.cursor = 0;
                state.scroll_offset = 0;
            }
            KeyCode::Enter => {
                if state.selected.is_empty() && universal_locked.is_empty() {
                    continue;
                }
                let mut selected = state.selected.iter().copied().collect::<Vec<_>>();
                selected.sort_by_key(|p| p.as_str());
                return Ok(selected);
            }
            KeyCode::Esc => return Err(InstallerError::PromptCancelled),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Err(InstallerError::PromptCancelled)
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.query.push(c);
                state.cursor = 0;
                state.scroll_offset = 0;
            }
            _ => {}
        }
    }
}

fn make_divider(label: &str, suffix: &str, width: u16) -> Line<'static> {
    let prefix = "── ";
    let tail = if suffix.is_empty() {
        " ".to_string()
    } else {
        format!(" {} ", suffix)
    };
    let used = prefix.len() + label.len() + tail.len();
    let remaining = (width as usize).saturating_sub(used);
    let fill = "─".repeat(remaining);

    Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(
            label.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}{}", tail, fill),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn adjust_scroll(state: &mut UiState, total_items: usize, visible_height: usize) {
    if total_items == 0 || visible_height == 0 {
        state.scroll_offset = 0;
        return;
    }

    let max_scroll = total_items.saturating_sub(1);
    state.scroll_offset = state.scroll_offset.min(max_scroll);

    if state.cursor < state.scroll_offset {
        state.scroll_offset = state.cursor;
    }

    loop {
        let top_lines = if state.scroll_offset > 0 { 1usize } else { 0 };
        let items_space = visible_height.saturating_sub(top_lines);
        let remaining = total_items - state.scroll_offset;
        let bottom_lines = if remaining > items_space { 1usize } else { 0 };
        let visible_count = items_space.saturating_sub(bottom_lines).max(1);

        if state.cursor < state.scroll_offset + visible_count {
            break;
        }

        state.scroll_offset += 1;
        if state.scroll_offset >= total_items {
            state.scroll_offset = total_items.saturating_sub(1);
            break;
        }
    }
}

fn draw_ui(
    frame: &mut ratatui::Frame,
    universal_locked: &[ProviderId],
    filtered: &[ProviderId],
    state: &UiState,
) {
    let size = frame.area();
    let width = size.width;
    let chunks = compute_layout(size, universal_locked.len());

    render_locked(frame, chunks[0], universal_locked, width);
    render_additional_header(frame, chunks[2], width);
    render_search(frame, chunks[3], state);
    render_instructions(frame, chunks[4]);
    render_selectable(frame, chunks[6], filtered, state);

    let summary = selected_summary(universal_locked, &state.selected);
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "Selected: ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(summary),
    ]));
    frame.render_widget(footer, chunks[8]);
}

fn render_locked(
    frame: &mut ratatui::Frame,
    area: Rect,
    universal_locked: &[ProviderId],
    width: u16,
) {
    let lines = if universal_locked.is_empty() {
        let label = "Universal (.agents/skills) — none";
        let prefix = "── ";
        let used = prefix.len() + label.len() + 1;
        let remaining = (width as usize).saturating_sub(used);
        let fill = "─".repeat(remaining);
        vec![Line::from(Span::styled(
            format!("{}{} {}", prefix, label, fill),
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        let mut out = Vec::with_capacity(universal_locked.len() + 1);
        out.push(make_divider(
            "Universal (.agents/skills)",
            "— always included",
            width,
        ));
        for provider in universal_locked {
            out.push(Line::from(vec![
                Span::styled("  ● ", Style::default().fg(Color::Green)),
                Span::raw(provider_display_name(*provider)),
            ]));
        }
        out
    };

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_additional_header(frame: &mut ratatui::Frame, area: Rect, width: u16) {
    let divider = make_divider("Additional agents", "", width);
    frame.render_widget(Paragraph::new(vec![divider]), area);
}

fn render_search(frame: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let search = Paragraph::new(Line::from(vec![
        Span::styled("Search: ", Style::default().fg(Color::DarkGray)),
        Span::raw(&state.query),
        Span::styled("█", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(search, area);
}

fn render_instructions(frame: &mut ratatui::Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        "↑↓ move, space select, enter confirm",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hint, area);
}

fn render_selectable(
    frame: &mut ratatui::Frame,
    area: Rect,
    filtered: &[ProviderId],
    state: &UiState,
) {
    let height = area.height as usize;
    let mut lines = Vec::new();

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matches found",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let total = filtered.len();
        let offset = state.scroll_offset;

        let has_top = offset > 0;
        let top_lines = if has_top { 1 } else { 0 };
        let items_space = height.saturating_sub(top_lines);
        let remaining = total - offset;
        let has_bottom = remaining > items_space;
        let bottom_lines = if has_bottom { 1 } else { 0 };
        let visible_count = items_space.saturating_sub(bottom_lines).max(1);

        if has_top {
            lines.push(Line::from(Span::styled(
                format!("↑ {} more", offset),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let end = (offset + visible_count).min(total);
        for i in offset..end {
            let provider = filtered[i];
            let is_cursor = i == state.cursor;
            let is_selected = state.selected.contains(&provider);

            let marker = if is_selected { "●" } else { "○" };
            let prefix = if is_cursor { ">" } else { " " };
            let path = provider_project_path(provider);

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} {} ", prefix, marker),
                    Style::default().fg(if is_selected {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(provider_display_name(provider), Style::default()),
                Span::styled(format!(" ({})", path), Style::default().fg(Color::DarkGray)),
            ]));
        }

        if has_bottom {
            let below = total - end;
            lines.push(Line::from(Span::styled(
                format!("↓ {} more", below),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn selected_summary(universal_locked: &[ProviderId], selected: &HashSet<ProviderId>) -> String {
    let mut names = universal_locked
        .iter()
        .map(|p| provider_display_name(*p))
        .collect::<Vec<_>>();

    let mut selected_names = selected
        .iter()
        .map(|p| provider_display_name(*p))
        .collect::<Vec<_>>();
    selected_names.sort();
    names.extend(selected_names);

    if names.is_empty() {
        return "(none)".to_string();
    }
    if names.len() <= 4 {
        return names.join(", ");
    }
    format!("{} +{} more", names[..4].join(", "), names.len() - 4)
}

const VIEWPORT_HEIGHT: u16 = 24;

fn compute_layout(area: Rect, locked_count: usize) -> std::rc::Rc<[Rect]> {
    let locked_len = if locked_count == 0 {
        1
    } else {
        1 + locked_count as u16
    };
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(locked_len), // 0: locked section
            Constraint::Length(1),          // 1: spacer
            Constraint::Length(1),          // 2: additional agents header
            Constraint::Length(1),          // 3: search
            Constraint::Length(1),          // 4: instructions
            Constraint::Length(1),          // 5: spacer
            Constraint::Min(1),             // 6: selectable list
            Constraint::Length(1),          // 7: spacer
            Constraint::Length(1),          // 8: footer
        ])
        .split(area)
}

fn setup_terminal(height: u16) -> io::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.show_cursor()
}

fn move_cursor_below_viewport(viewport_bottom: u16) {
    crossterm::execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(0, viewport_bottom),
        crossterm::cursor::MoveToNextLine(1)
    )
    .ok();
}

// ── Generic single-select prompt ─────────────────────────────────────────────

pub fn prompt_select(message: &str, options: &[&str], default: usize) -> Result<usize> {
    if options.is_empty() {
        return Err(InstallerError::PromptError {
            message: "no options provided".to_string(),
        });
    }

    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(InstallerError::PromptError {
            message: "interactive selection requires a TTY".to_string(),
        });
    }

    println!("{}", message);

    let mut cursor = default.min(options.len() - 1);
    // hint + options
    let viewport_height = 1 + options.len() as u16;

    let mut terminal =
        setup_terminal(viewport_height).map_err(|err| InstallerError::PromptError {
            message: err.to_string(),
        })?;

    let mut viewport_bottom = viewport_height;
    let result = run_select_loop(&mut terminal, options, &mut cursor, &mut viewport_bottom);

    restore_terminal(&mut terminal).map_err(|err| InstallerError::PromptError {
        message: err.to_string(),
    })?;
    move_cursor_below_viewport(viewport_bottom);

    result
}

fn run_select_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    options: &[&str],
    cursor: &mut usize,
    viewport_bottom: &mut u16,
) -> Result<usize> {
    loop {
        let cur = *cursor;
        let completed = terminal
            .draw(|frame| draw_select(frame, options, cur))
            .map_err(|err| InstallerError::PromptError {
                message: err.to_string(),
            })?;
        *viewport_bottom = completed.area.bottom();

        let event = event::read().map_err(|err| InstallerError::PromptError {
            message: err.to_string(),
        })?;

        let Event::Key(key) = event else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up => *cursor = cursor.saturating_sub(1),
            KeyCode::Down => *cursor = (*cursor + 1).min(options.len().saturating_sub(1)),
            KeyCode::Enter => return Ok(*cursor),
            KeyCode::Esc => return Err(InstallerError::PromptCancelled),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Err(InstallerError::PromptCancelled)
            }
            _ => {}
        }
    }
}

fn draw_select(frame: &mut ratatui::Frame, options: &[&str], cursor: usize) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // instructions
            Constraint::Min(1),    // options
        ])
        .split(size);

    let hint = Paragraph::new(Line::from(Span::styled(
        "↑↓ move, enter confirm",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hint, chunks[0]);

    let mut lines = Vec::with_capacity(options.len());
    for (idx, label) in options.iter().enumerate() {
        let is_cursor = idx == cursor;
        let marker = if is_cursor { "●" } else { "○" };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", marker),
                Style::default().fg(if is_cursor {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(*label, Style::default()),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), chunks[1]);
}

// ── Provider helpers ─────────────────────────────────────────────────────────

fn filtered_items(items: &[ProviderId], query: &str) -> Vec<ProviderId> {
    if query.trim().is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .copied()
        .filter(|p| {
            provider_display_name(*p).to_lowercase().contains(&q) || p.as_str().contains(&q)
        })
        .collect()
}

fn resolve_candidates(options: &InteractiveProviderSelectionOptions<'_>) -> Vec<ProviderId> {
    if let Some(candidates) = &options.candidates {
        return dedupe_non_universal(candidates);
    }

    let all = supported_providers()
        .iter()
        .map(|p| p.id)
        .collect::<Vec<_>>();
    dedupe_non_universal(&all)
}

fn resolve_defaults(
    options: &InteractiveProviderSelectionOptions<'_>,
    selectable: &[ProviderId],
) -> HashSet<ProviderId> {
    let base = if let Some(defaults) = &options.defaults {
        defaults.clone()
    } else {
        let detected = detect_providers(options.project_root)
            .into_iter()
            .map(|d| d.provider)
            .collect::<Vec<_>>();
        if detected.is_empty() {
            selectable.to_vec()
        } else {
            detected
        }
    };

    base.into_iter()
        .filter(|p| selectable.contains(p))
        .collect::<HashSet<_>>()
}

fn dedupe_non_universal(input: &[ProviderId]) -> Vec<ProviderId> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for provider in input {
        if *provider == ProviderId::Universal {
            continue;
        }
        if seen.insert(*provider) {
            out.push(*provider);
        }
    }
    out
}

fn provider_display_name(provider: ProviderId) -> &'static str {
    supported_providers()
        .iter()
        .find(|p| p.id == provider)
        .map(|p| p.display_name)
        .unwrap_or(provider.as_str())
}

fn provider_project_path(provider: ProviderId) -> &'static str {
    supported_providers()
        .iter()
        .find(|p| p.id == provider)
        .map(|p| p.project_path)
        .unwrap_or(".agents/skills")
}

// ── Interactive install orchestration ────────────────────────────────────────

pub fn install_interactive(source: SkillSource, args: &InstallSkillArgs) -> Result<InstallResult> {
    let cwd = std::env::current_dir().map_err(|err| InstallerError::IoError {
        path: std::path::PathBuf::from("."),
        message: format!("failed to read cwd: {err}"),
    })?;

    let providers = match &args.providers {
        Some(csv) => parse_providers_csv(csv)?,
        None => {
            let selection = prompt_provider_selection(InteractiveProviderSelectionOptions {
                project_root: args.project_root.as_deref().or(Some(&cwd)),
                candidates: None,
                defaults: None,
                message: "◆  Select providers to install to",
            })?;
            if selection.selected.is_empty() {
                return Err(InstallerError::PromptError {
                    message: "no providers selected".to_string(),
                });
            }
            selection.selected
        }
    };

    let scope = match args.scope {
        Some(s) => s,
        None => {
            print_prompt_spacing();
            let labels = [
                "Project (Install in current directory (committed with your project))",
                "Global",
            ];
            let idx = prompt_select("◆  Installation scope", &labels, 0)?;
            if idx == 0 {
                Scope::Project
            } else {
                Scope::User
            }
        }
    };

    let method = match args.method {
        Some(m) => m,
        None => {
            print_prompt_spacing();
            let labels = [
                "Symlink (Recommended) (Single source of truth, easy updates)",
                "Copy to all agents",
            ];
            let idx = prompt_select("◆  Installation method", &labels, 0)?;
            if idx == 0 {
                InstallMethod::Symlink
            } else {
                InstallMethod::Copy
            }
        }
    };

    let project_root = match scope {
        Scope::User => None,
        Scope::Project => Some(args.project_root.clone().unwrap_or(cwd)),
    };

    let force = if args.force {
        true
    } else {
        let existing =
            find_existing_destinations(&source, &providers, scope, project_root.as_deref())?;
        if existing.is_empty() {
            false
        } else {
            print_prompt_spacing();
            let msg = if existing.len() == 1 {
                format!(
                    "◆  Skill already exists at {}. Overwrite?",
                    existing[0].display()
                )
            } else {
                format!(
                    "◆  Skill already exists in {} locations. Overwrite?",
                    existing.len()
                )
            };
            let idx = prompt_select(&msg, &["Yes", "No"], 1)?;
            if idx == 0 {
                true
            } else {
                return Err(InstallerError::PromptCancelled);
            }
        }
    };

    install(InstallRequest {
        source,
        providers,
        scope,
        project_root,
        method,
        force,
    })
}

fn print_prompt_spacing() {
    // Two-line separation between interactive steps.
    println!();
    println!();
}
