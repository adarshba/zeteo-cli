use anyhow::Result;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;

use crate::backends::{
    elasticsearch::ElasticsearchClient, kibana::KibanaClient, openobserve::OpenObserveClient,
    LogBackendClient,
};
use crate::config::{Config, LogBackend};
use crate::providers::{create_log_tools, AiProvider, ChatRequest, Message, ToolCall};
use crate::session::{try_create_session_store, ConversationInfo, SessionStore, StoredMessage};
use crate::tools::ToolExecutor;

#[derive(Clone)]
struct SlashCommand {
    name: &'static str,
    description: &'static str,
    shortcut: Option<&'static str>,
}

const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        name: "backend",
        description: "Switch log backend (kibana/openobserve)",
        shortcut: Some("b"),
    },
    SlashCommand {
        name: "clear",
        description: "Clear current session history",
        shortcut: Some("c"),
    },
    SlashCommand {
        name: "copy",
        description: "Copy last AI response to clipboard",
        shortcut: Some("y"),
    },
    SlashCommand {
        name: "help",
        description: "Show available commands",
        shortcut: Some("h"),
    },
    SlashCommand {
        name: "index",
        description: "Change index pattern for this session",
        shortcut: Some("i"),
    },
    SlashCommand {
        name: "quit",
        description: "Exit the application",
        shortcut: Some("q"),
    },
    SlashCommand {
        name: "resume",
        description: "Resume a previous conversation",
        shortcut: Some("r"),
    },
];

/// Commands that can be auto-executed without arguments
const AUTO_EXECUTE_COMMANDS: &[&str] = &["quit", "clear", "help", "resume", "copy"];

/// Check if a command should be auto-executed (doesn't require arguments)
fn is_auto_execute_command(cmd: &str) -> bool {
    AUTO_EXECUTE_COMMANDS.contains(&cmd)
}

mod markdown {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::{Line, Span},
    };

    pub fn parse_markdown_to_lines(text: &str, width: usize) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut in_code_block = false;
        let mut code_block_content = String::new();

        for line in text.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    for code_line in code_block_content.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", code_line),
                            Style::default()
                                .fg(Color::Rgb(152, 195, 121))
                                .bg(Color::Rgb(40, 44, 52)),
                        )));
                    }
                    code_block_content.clear();
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    let lang = line.trim_start_matches("```").trim();

                    if !lang.is_empty() {
                        lines.push(Line::from(Span::styled(
                            format!("  ┌─ {} ", lang),
                            Style::default().fg(Color::Rgb(97, 175, 239)),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            "  ┌─ code ",
                            Style::default().fg(Color::Rgb(97, 175, 239)),
                        )));
                    }
                }
                continue;
            }

            if in_code_block {
                code_block_content.push_str(line);
                code_block_content.push('\n');
                continue;
            }

            if let Some(stripped) = line.strip_prefix("### ") {
                lines.push(Line::from(Span::styled(
                    stripped.to_string(),
                    Style::default()
                        .fg(Color::Rgb(198, 120, 221))
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }
            if let Some(stripped) = line.strip_prefix("## ") {
                lines.push(Line::from(Span::styled(
                    stripped.to_string(),
                    Style::default()
                        .fg(Color::Rgb(224, 108, 117))
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }
            if let Some(stripped) = line.strip_prefix("# ") {
                lines.push(Line::from(Span::styled(
                    stripped.to_string(),
                    Style::default()
                        .fg(Color::Rgb(229, 192, 123))
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }

            if line.starts_with("- ") || line.starts_with("* ") {
                let content = &line[2..];
                let mut spans = vec![Span::styled(
                    "  • ",
                    Style::default().fg(Color::Rgb(97, 175, 239)),
                )];
                spans.extend(parse_inline_markdown(content));
                lines.push(Line::from(spans));
                continue;
            }

            if let Some(rest) = line.strip_prefix(|c: char| c.is_ascii_digit()) {
                if let Some(content) = rest.strip_prefix(". ") {
                    let num_char = line.chars().next().unwrap();
                    let mut spans = vec![Span::styled(
                        format!("  {}. ", num_char),
                        Style::default().fg(Color::Rgb(97, 175, 239)),
                    )];
                    spans.extend(parse_inline_markdown(content));
                    lines.push(Line::from(spans));
                    continue;
                }
            }

            if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
                lines.push(Line::from(Span::styled(
                    "─".repeat(width.min(60)),
                    Style::default().fg(Color::Rgb(92, 99, 112)),
                )));
                continue;
            }

            if let Some(content) = line.strip_prefix("> ") {
                let mut spans = vec![Span::styled(
                    "│ ",
                    Style::default().fg(Color::Rgb(92, 99, 112)),
                )];
                spans.extend(parse_inline_markdown(content));
                for span in &mut spans[1..] {
                    span.style = span.style.fg(Color::Rgb(171, 178, 191));
                }
                lines.push(Line::from(spans));
                continue;
            }

            if line.trim().is_empty() {
                lines.push(Line::from(""));
                continue;
            }

            let spans = parse_inline_markdown(line);

            let wrapped = wrap_spans(spans, width);
            lines.extend(wrapped);
        }

        if in_code_block && !code_block_content.is_empty() {
            for code_line in code_block_content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", code_line),
                    Style::default()
                        .fg(Color::Rgb(152, 195, 121))
                        .bg(Color::Rgb(40, 44, 52)),
                )));
            }
        }

        lines
    }

    fn parse_inline_markdown(text: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut current = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '`' => {
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(Color::Rgb(229, 229, 234)),
                        ));
                        current.clear();
                    }
                    let mut code = String::new();
                    while let Some(&next) = chars.peek() {
                        if next == '`' {
                            chars.next();
                            break;
                        }
                        code.push(chars.next().unwrap());
                    }
                    spans.push(Span::styled(
                        code,
                        Style::default()
                            .fg(Color::Rgb(152, 195, 121))
                            .bg(Color::Rgb(40, 44, 52)),
                    ));
                }
                '*' | '_' => {
                    if chars.peek() == Some(&c) {
                        chars.next();
                        if !current.is_empty() {
                            spans.push(Span::styled(
                                current.clone(),
                                Style::default().fg(Color::Rgb(229, 229, 234)),
                            ));
                            current.clear();
                        }
                        let mut bold = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == c {
                                chars.next();
                                if chars.peek() == Some(&c) {
                                    chars.next();
                                    break;
                                }
                                bold.push(c);
                            } else {
                                bold.push(chars.next().unwrap());
                            }
                        }
                        spans.push(Span::styled(
                            bold,
                            Style::default()
                                .fg(Color::Rgb(229, 229, 234))
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        if !current.is_empty() {
                            spans.push(Span::styled(
                                current.clone(),
                                Style::default().fg(Color::Rgb(229, 229, 234)),
                            ));
                            current.clear();
                        }
                        let mut italic = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == c {
                                chars.next();
                                break;
                            }
                            italic.push(chars.next().unwrap());
                        }
                        spans.push(Span::styled(
                            italic,
                            Style::default()
                                .fg(Color::Rgb(229, 229, 234))
                                .add_modifier(Modifier::ITALIC),
                        ));
                    }
                }
                '[' => {
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            Style::default().fg(Color::Rgb(229, 229, 234)),
                        ));
                        current.clear();
                    }
                    let mut link_text = String::new();
                    let mut found_close = false;
                    while let Some(&next) = chars.peek() {
                        if next == ']' {
                            chars.next();
                            found_close = true;
                            break;
                        }
                        link_text.push(chars.next().unwrap());
                    }
                    if found_close && chars.peek() == Some(&'(') {
                        chars.next();
                        let mut url = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == ')' {
                                chars.next();
                                break;
                            }
                            url.push(chars.next().unwrap());
                        }
                        spans.push(Span::styled(
                            link_text,
                            Style::default()
                                .fg(Color::Rgb(97, 175, 239))
                                .add_modifier(Modifier::UNDERLINED),
                        ));
                    } else {
                        current.push('[');
                        current.push_str(&link_text);
                        if found_close {
                            current.push(']');
                        }
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            spans.push(Span::styled(
                current,
                Style::default().fg(Color::Rgb(229, 229, 234)),
            ));
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        spans
    }

    fn wrap_spans(spans: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
        if width == 0 {
            return vec![Line::from(spans)];
        }

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut current_width = 0;

        for span in spans {
            let text = span.content.to_string();
            let style = span.style;

            for word in text.split_inclusive(' ') {
                let word_len = word.chars().count();

                if current_width + word_len > width && current_width > 0 {
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                    current_width = 0;
                }

                current_line.push(Span::styled(word.to_string(), style));
                current_width += word_len;
            }
        }

        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        if lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines
    }
}

struct ChatMessage {
    role: String,
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
}

pub struct TuiApp {
    provider: Arc<dyn AiProvider>,
    tool_executor: Option<ToolExecutor>,
    config: Option<Config>,
    input: String,
    cursor_position: usize,
    messages: Vec<ChatMessage>,
    scroll_offset: usize,
    is_loading: bool,
    show_welcome: bool,
    status_message: Option<String>,
    backend_name: Option<String>,
    cursor_visible: bool,
    frame_count: u64,
    show_slash_modal: bool,
    slash_filter: String,
    slash_selected: usize,
    available_backends: Vec<String>,
    session_store: Option<SessionStore>,
    show_resume_modal: bool,
    resume_sessions: Vec<ConversationInfo>,
    resume_selected: usize,
    session_index_pattern: Option<String>,
    selected_message: Option<usize>, // Index of selected message for copying
}

impl TuiApp {
    pub fn new(
        provider: Arc<dyn AiProvider>,
        tool_executor: Option<ToolExecutor>,
        backend_name: Option<String>,
        config: Option<Config>,
        session_store: Option<SessionStore>,
    ) -> Self {
        let available_backends = config
            .as_ref()
            .map(|c| c.backends.keys().cloned().collect())
            .unwrap_or_default();

        Self {
            provider,
            tool_executor,
            config,
            input: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            is_loading: false,
            show_welcome: true,
            status_message: None,
            backend_name,
            cursor_visible: true,
            frame_count: 0,
            show_slash_modal: false,
            slash_filter: String::new(),
            slash_selected: 0,
            available_backends,
            session_store,
            show_resume_modal: false,
            resume_sessions: Vec::new(),
            resume_selected: 0,
            session_index_pattern: None,
            selected_message: None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableBracketedPaste
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            self.frame_count += 1;
            if self.frame_count.is_multiple_of(10) {
                self.cursor_visible = !self.cursor_visible;
            }

            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.cursor_visible = true;
                        self.frame_count = 0;

                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c')
                        {
                            return Ok(());
                        }

                        // Ctrl+Y to copy selected or last AI response
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('y')
                        {
                            self.copy_response();
                            continue;
                        }

                        // Ctrl+Up to select previous message
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Up
                        {
                            self.select_previous_message();
                            continue;
                        }

                        // Ctrl+Down to select next message
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Down
                        {
                            self.select_next_message();
                            continue;
                        }

                        // Escape clears message selection
                        if key.code == KeyCode::Esc {
                            if self.selected_message.is_some() {
                                self.selected_message = None;
                                continue;
                            }
                            return Ok(());
                        }

                        if self.show_slash_modal {
                            match key.code {
                                KeyCode::Esc => {
                                    self.show_slash_modal = false;
                                    self.slash_filter.clear();
                                    self.slash_selected = 0;
                                    if self.input == "/" {
                                        self.input.clear();
                                        self.cursor_position = 0;
                                    }
                                }
                                KeyCode::Enter => {
                                    let filtered = self.get_filtered_commands();
                                    if let Some(cmd) = filtered.get(self.slash_selected) {
                                        let cmd_name = cmd.name.to_string();
                                        self.show_slash_modal = false;
                                        self.slash_filter.clear();
                                        self.slash_selected = 0;
                                        self.input = format!("/{}", cmd_name);
                                        self.cursor_position = self.input.len();

                                        if is_auto_execute_command(&cmd_name) {
                                            if let Some(result) = self
                                                .execute_slash_command(&self.input.clone())
                                                .await
                                            {
                                                if result == "quit" {
                                                    return Ok(());
                                                }
                                            }
                                            self.input.clear();
                                            self.cursor_position = 0;
                                        } else if cmd_name == "backend" {
                                            self.input = "/backend ".to_string();
                                            self.cursor_position = self.input.len();
                                        } else if cmd_name == "index" {
                                            self.input = "/index ".to_string();
                                            self.cursor_position = self.input.len();
                                        }
                                    }
                                }
                                KeyCode::Up => {
                                    if self.slash_selected > 0 {
                                        self.slash_selected -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    let filtered = self.get_filtered_commands();
                                    if self.slash_selected < filtered.len().saturating_sub(1) {
                                        self.slash_selected += 1;
                                    }
                                }
                                KeyCode::Char(c) => {
                                    self.slash_filter.push(c);
                                    self.slash_selected = 0;
                                    self.input.push(c);
                                    self.cursor_position += 1;
                                }
                                KeyCode::Backspace => {
                                    if !self.slash_filter.is_empty() {
                                        self.slash_filter.pop();
                                        self.slash_selected = 0;
                                    }
                                    if self.cursor_position > 0 {
                                        self.cursor_position -= 1;
                                        self.input.remove(self.cursor_position);
                                    }
                                    if self.input.is_empty() || !self.input.starts_with('/') {
                                        self.show_slash_modal = false;
                                        self.slash_filter.clear();
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        if self.show_resume_modal {
                            match key.code {
                                KeyCode::Esc => {
                                    self.show_resume_modal = false;
                                    self.resume_selected = 0;
                                }
                                KeyCode::Enter => {
                                    if let Some(session) =
                                        self.resume_sessions.get(self.resume_selected)
                                    {
                                        let session_id = session.id.clone();
                                        self.show_resume_modal = false;
                                        self.resume_selected = 0;

                                        if let Err(e) = self.resume_session(&session_id).await {
                                            self.messages.push(ChatMessage {
                                                role: "error".to_string(),
                                                content: format!("Failed to resume session: {}", e),
                                                tool_calls: None,
                                                tool_call_id: None,
                                            });
                                        } else {
                                            self.show_welcome = false;
                                            self.status_message =
                                                Some("Session resumed".to_string());
                                        }
                                    }
                                }
                                KeyCode::Up => {
                                    if self.resume_selected > 0 {
                                        self.resume_selected -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if self.resume_selected
                                        < self.resume_sessions.len().saturating_sub(1)
                                    {
                                        self.resume_selected += 1;
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        match key.code {
                            KeyCode::Enter => {
                                if !self.input.trim().is_empty() && !self.is_loading {
                                    self.show_welcome = false;
                                    let input = self.input.clone();
                                    self.input.clear();
                                    self.cursor_position = 0;

                                    if input.starts_with('/') {
                                        if let Some(result) =
                                            self.execute_slash_command(&input).await
                                        {
                                            if result == "quit" {
                                                return Ok(());
                                            }
                                        }
                                        continue;
                                    }

                                    self.messages.push(ChatMessage {
                                        role: "user".to_string(),
                                        content: input.clone(),
                                        tool_calls: None,
                                        tool_call_id: None,
                                    });
                                    self.is_loading = true;
                                    self.status_message = Some("Thinking...".to_string());
                                    self.scroll_to_bottom();

                                    terminal.draw(|f| self.ui(f))?;

                                    if let Err(e) = self.process_message(input).await {
                                        self.messages.push(ChatMessage {
                                            role: "error".to_string(),
                                            content: e.to_string(),
                                            tool_calls: None,
                                            tool_call_id: None,
                                        });
                                    }
                                    self.is_loading = false;
                                    self.status_message = None;
                                    self.scroll_to_bottom();

                                    self.save_session().await;
                                }
                            }
                            KeyCode::Char('/') if self.input.is_empty() => {
                                self.input.push('/');
                                self.cursor_position = 1;
                                self.show_slash_modal = true;
                                self.slash_filter.clear();
                                self.slash_selected = 0;
                            }
                            KeyCode::Char(c) => {
                                self.input.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                    self.input.remove(self.cursor_position);
                                }
                            }
                            KeyCode::Delete => {
                                if self.cursor_position < self.input.len() {
                                    self.input.remove(self.cursor_position);
                                }
                            }
                            KeyCode::Left => {
                                self.cursor_position = self.cursor_position.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.input.len() {
                                    self.cursor_position += 1;
                                }
                            }
                            KeyCode::Home => {
                                self.cursor_position = 0;
                            }
                            KeyCode::End => {
                                self.cursor_position = self.input.len();
                            }
                            KeyCode::Up => {
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                self.scroll_offset = self.scroll_offset.saturating_add(1);
                            }
                            KeyCode::PageUp => {
                                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                            }
                            KeyCode::PageDown => {
                                self.scroll_offset = self.scroll_offset.saturating_add(10);
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse_event) => match mouse_event.kind {
                        event::MouseEventKind::ScrollUp => {
                            self.scroll_offset = self.scroll_offset.saturating_sub(3);
                        }
                        event::MouseEventKind::ScrollDown => {
                            self.scroll_offset = self.scroll_offset.saturating_add(3);
                        }
                        _ => {}
                    },
                    Event::Paste(text) => {
                        // Handle pasted text - replace newlines with spaces for single-line input
                        // or keep them if we want multi-line support
                        let cleaned_text = text.replace('\r', "").replace('\n', " ");
                        for c in cleaned_text.chars() {
                            self.input.insert(self.cursor_position, c);
                            self.cursor_position += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_filtered_commands(&self) -> Vec<&SlashCommand> {
        let filter = self.slash_filter.to_lowercase();
        SLASH_COMMANDS
            .iter()
            .filter(|cmd| {
                if filter.is_empty() {
                    true
                } else {
                    cmd.name.to_lowercase().contains(&filter)
                        || cmd.shortcut.map(|s| s == filter).unwrap_or(false)
                }
            })
            .collect()
    }

    async fn execute_slash_command(&mut self, input: &str) -> Option<String> {
        let parts: Vec<&str> = input.trim_start_matches('/').split_whitespace().collect();
        let cmd = parts.first()?;
        let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

        match *cmd {
            "quit" | "q" => Some("quit".to_string()),
            "clear" | "c" => {
                self.messages.clear();
                self.scroll_offset = 0;
                self.show_welcome = true;

                if let Some(ref session_store) = self.session_store {
                    let _ = session_store.clear_current_session().await;
                }

                self.status_message = Some("Session cleared".to_string());
                Some("cleared".to_string())
            }
            "help" | "h" => {
                let help_text = SLASH_COMMANDS
                    .iter()
                    .map(|c| {
                        let shortcut = c.shortcut.map(|s| format!(" ({})", s)).unwrap_or_default();
                        format!("**/{}{shortcut}** - {}", c.name, c.description)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                self.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: format!(
                        "## Available Commands\n\n{}\n\n*Tip: Type `/` to see command suggestions*",
                        help_text
                    ),
                    tool_calls: None,
                    tool_call_id: None,
                });
                Some("help".to_string())
            }
            "index" | "i" => {
                if args.is_empty() {
                    let current_index = self.get_current_index_pattern();
                    self.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "## Index Pattern\n\nCurrent: **{}**\n\n*Usage: `/index <pattern>` to change for this session*\n\n*Example: `/index logs-prod-*`*",
                            current_index.as_deref().unwrap_or("not set")
                        ),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    Some("index_show".to_string())
                } else {
                    let new_pattern = args.join(" ");
                    self.session_index_pattern = Some(new_pattern.clone());
                    if let Some(ref mut executor) = self.tool_executor {
                        executor.set_index_pattern(Some(new_pattern.clone()));
                    }
                    self.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "✓ Index pattern changed to **{}** for this session",
                            new_pattern
                        ),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    Some("index_set".to_string())
                }
            }
            "resume" | "r" => {
                if let Some(ref session_store) = self.session_store {
                    match session_store.list_sessions().await {
                        Ok(sessions) => {
                            if sessions.is_empty() {
                                self.messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: "No previous conversations found.".to_string(),
                                    tool_calls: None,
                                    tool_call_id: None,
                                });
                                Some("resume_empty".to_string())
                            } else {
                                self.resume_sessions = sessions;
                                self.resume_selected = 0;
                                self.show_resume_modal = true;
                                Some("resume_modal".to_string())
                            }
                        }
                        Err(e) => {
                            self.messages.push(ChatMessage {
                                role: "error".to_string(),
                                content: format!("Failed to load sessions: {}", e),
                                tool_calls: None,
                                tool_call_id: None,
                            });
                            Some("resume_error".to_string())
                        }
                    }
                } else {
                    self.messages.push(ChatMessage {
                        role: "error".to_string(),
                        content:
                            "Redis not configured. Set REDIS_URL to enable session persistence."
                                .to_string(),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    Some("resume_no_redis".to_string())
                }
            }
            "copy" | "y" => {
                self.copy_response();
                Some("copied".to_string())
            }
            "backend" | "b" => {
                if args.is_empty() {
                    let backends_list = if self.available_backends.is_empty() {
                        "No backends configured".to_string()
                    } else {
                        self.available_backends
                            .iter()
                            .map(|b| {
                                let active = if self.backend_name.as_deref() == Some(b) {
                                    " ●"
                                } else {
                                    ""
                                };
                                format!("- **{}**{}", b, active)
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    };

                    self.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "## Log Backends\n\nCurrent: **{}**\n\nAvailable:\n{}\n\n*Usage: `/backend <name>` to switch*",
                            self.backend_name.as_deref().unwrap_or("none"),
                            backends_list
                        ),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    Some("backend_list".to_string())
                } else {
                    let backend_name = args[0].to_lowercase();
                    if self.switch_backend(&backend_name) {
                        self.messages.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: format!("✓ Switched to **{}** backend", backend_name),
                            tool_calls: None,
                            tool_call_id: None,
                        });
                    } else {
                        self.messages.push(ChatMessage {
                            role: "error".to_string(),
                            content: format!(
                                "Backend '{}' not found. Available: {}",
                                backend_name,
                                self.available_backends.join(", ")
                            ),
                            tool_calls: None,
                            tool_call_id: None,
                        });
                    }
                    Some("backend_switch".to_string())
                }
            }
            _ => {
                self.messages.push(ChatMessage {
                    role: "error".to_string(),
                    content: format!(
                        "Unknown command: /{}. Type /help for available commands.",
                        cmd
                    ),
                    tool_calls: None,
                    tool_call_id: None,
                });
                Some("unknown".to_string())
            }
        }
    }

    fn switch_backend(&mut self, name: &str) -> bool {
        if let Some(config) = &self.config {
            if let Some((client, backend_name)) = try_backend(name, config) {
                self.tool_executor = Some(ToolExecutor::new(client));
                self.backend_name = Some(backend_name);
                return true;
            }
        }
        false
    }

    /// Get assistant message indices for selection
    fn get_assistant_message_indices(&self) -> Vec<usize> {
        self.messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.role == "assistant" && !m.content.is_empty())
            .map(|(i, _)| i)
            .collect()
    }

    /// Select previous assistant message
    fn select_previous_message(&mut self) {
        let indices = self.get_assistant_message_indices();
        if indices.is_empty() {
            return;
        }

        self.selected_message = match self.selected_message {
            None => Some(*indices.last().unwrap()),
            Some(current) => {
                // Find the previous assistant message index
                indices
                    .iter()
                    .rev()
                    .find(|&&i| i < current)
                    .copied()
                    .or(Some(*indices.last().unwrap()))
            }
        };
        self.status_message = Some("Use Ctrl+Y to copy, Esc to cancel".to_string());
    }

    /// Select next assistant message
    fn select_next_message(&mut self) {
        let indices = self.get_assistant_message_indices();
        if indices.is_empty() {
            return;
        }

        self.selected_message = match self.selected_message {
            None => Some(*indices.first().unwrap()),
            Some(current) => {
                // Find the next assistant message index
                indices
                    .iter()
                    .find(|&&i| i > current)
                    .copied()
                    .or(Some(*indices.first().unwrap()))
            }
        };
        self.status_message = Some("Use Ctrl+Y to copy, Esc to cancel".to_string());
    }

    /// Copy selected or last assistant response (with preceding prompt) to clipboard
    fn copy_response(&mut self) {
        // Find the assistant message index
        let assistant_idx = if let Some(idx) = self.selected_message {
            Some(idx)
        } else {
            // Find the last assistant message index
            self.messages
                .iter()
                .enumerate()
                .rev()
                .find(|(_, m)| m.role == "assistant")
                .map(|(i, _)| i)
        };

        if let Some(idx) = assistant_idx {
            let assistant_msg = &self.messages[idx];
            if assistant_msg.content.is_empty() {
                self.status_message = Some("Message is empty".to_string());
                return;
            }

            // Find the preceding user message
            let user_prompt = self.messages[..idx]
                .iter()
                .rev()
                .find(|m| m.role == "user")
                .map(|m| m.content.clone());

            // Format as prompt + response
            let content = if let Some(prompt) = user_prompt {
                format!(
                    "## Prompt\n\n{}\n\n## Response\n\n{}",
                    prompt, assistant_msg.content
                )
            } else {
                assistant_msg.content.clone()
            };

            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(&content) {
                    Ok(_) => {
                        self.status_message =
                            Some("Copied prompt & response to clipboard".to_string());
                        self.selected_message = None; // Clear selection after copy
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to copy: {}", e));
                    }
                },
                Err(e) => {
                    self.status_message = Some(format!("Clipboard not available: {}", e));
                }
            }
        } else {
            self.status_message = Some("No AI response to copy".to_string());
        }
    }

    /// Get the current index pattern (session override or from config)
    fn get_current_index_pattern(&self) -> Option<String> {
        if let Some(ref pattern) = self.session_index_pattern {
            return Some(pattern.clone());
        }

        if let Some(ref config) = self.config {
            if let Some(ref backend_name) = self.backend_name {
                if let Some(backend) = config.backends.get(backend_name) {
                    return match backend {
                        LogBackend::Elasticsearch { index_pattern, .. } => {
                            Some(index_pattern.clone())
                        }
                        LogBackend::Kibana { index_pattern, .. } => Some(index_pattern.clone()),
                        LogBackend::OpenObserve { stream, .. } => Some(stream.clone()),
                    };
                }
            }
        }
        None
    }

    /// Save the current session to Redis
    async fn save_session(&self) {
        if let Some(ref session_store) = self.session_store {
            let stored_messages: Vec<StoredMessage> = self
                .messages
                .iter()
                .filter(|m| m.role == "user" || m.role == "assistant")
                .map(|m| StoredMessage {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect();

            if !stored_messages.is_empty() {
                let _ = session_store.save_messages(&stored_messages).await;
            }
        }
    }

    /// Resume a previous session
    async fn resume_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(ref mut session_store) = self.session_store {
            let stored_messages = session_store.load_messages(session_id).await?;

            self.messages.clear();
            for stored in stored_messages {
                self.messages.push(ChatMessage {
                    role: stored.role,
                    content: stored.content,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }

            session_store.set_current_session_id(session_id.to_string());

            self.scroll_to_bottom();
            Ok(())
        } else {
            anyhow::bail!("No session store available")
        }
    }

    async fn process_message(&mut self, _input: String) -> Result<()> {
        let mut api_messages: Vec<Message> = self
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "tool")
            .map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
                tool_calls: m.tool_calls.clone(),
                tool_call_id: m.tool_call_id.clone(),
            })
            .collect();

        let system_message = self.build_system_message();
        api_messages.insert(
            0,
            Message {
                role: "system".to_string(),
                content: system_message,
                tool_calls: None,
                tool_call_id: None,
            },
        );

        let tools = if self.tool_executor.is_some() {
            Some(create_log_tools())
        } else {
            None
        };

        let request = ChatRequest {
            messages: api_messages.clone(),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            tools,
        };

        let response = self.provider.chat(request).await?;

        if let Some(tool_calls) = response.tool_calls {
            if !tool_calls.is_empty() && self.tool_executor.is_some() {
                self.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                });

                for tool_call in &tool_calls {
                    self.status_message =
                        Some(format!("Querying logs ({})...", tool_call.function.name));

                    let tool_result = if let Some(executor) = &self.tool_executor {
                        match executor
                            .execute(&tool_call.function.name, &tool_call.function.arguments)
                            .await
                        {
                            Ok(result) => result,
                            Err(e) => format!("Error executing tool: {}", e),
                        }
                    } else {
                        "No log backend configured".to_string()
                    };

                    self.messages.push(ChatMessage {
                        role: "tool".to_string(),
                        content: tool_result,
                        tool_calls: None,
                        tool_call_id: Some(tool_call.id.clone()),
                    });
                }

                self.status_message = Some("Analyzing results...".to_string());

                let mut followup_messages: Vec<Message> = self
                    .messages
                    .iter()
                    .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "tool")
                    .map(|m| Message {
                        role: m.role.clone(),
                        content: m.content.clone(),
                        tool_calls: m.tool_calls.clone(),
                        tool_call_id: m.tool_call_id.clone(),
                    })
                    .collect();

                followup_messages.insert(
                    0,
                    Message {
                        role: "system".to_string(),
                        content: self.build_system_message(),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                );

                let followup_request = ChatRequest {
                    messages: followup_messages,
                    temperature: Some(0.7),
                    max_tokens: Some(4096),
                    tools: None,
                };

                let followup_response = self.provider.chat(followup_request).await?;

                self.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: followup_response.content,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        } else {
            self.messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.content,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        Ok(())
    }

    fn build_system_message(&self) -> String {
        let mut msg = String::from(
            "You are Zeteo, an AI assistant specialized in log analysis and observability.\n\n",
        );

        if self.tool_executor.is_some() {
            msg.push_str(&format!(
                "## Your Capabilities\n\
                You have access to a log backend ({backend}) and can query logs to help users investigate issues, \
                find errors, analyze patterns, and troubleshoot problems.\n\n\
                ## Available Tools\n\n\
                ### 1. query_logs\n\
                Search and retrieve logs from the backend.\n\
                - **query** (required): Search string. Use '*' for all logs, or terms like 'error', 'timeout', 'failed'.\n\
                - **max_results** (optional): Number of results (default: 50, max: 200). Start with 20-50 for initial queries.\n\
                - **level** (optional): Filter by severity - must be exactly one of: ERROR, WARN, INFO, DEBUG\n\
                - **service** (optional): Filter by service name (use list_services first if unsure).\n\
                - **start_time** (optional): Relative time like '1h', '30m', '2d' or ISO 8601 format.\n\
                - **end_time** (optional): Defaults to now.\n\n\
                ### 2. list_services\n\
                Get available service names. No parameters required. Call this first if you need to filter by service.\n\n\
                ### 3. get_log_stats\n\
                Get aggregated statistics (counts by level, service distribution).\n\
                - **start_time** (optional): Start of time range.\n\
                - **end_time** (optional): End of time range.\n\n\
                ## Tool Usage Guidelines\n\n\
                1. **Start broad, then narrow**: Begin with a general query, then refine based on results.\n\
                2. **Use appropriate time ranges**: Default to '1h' for recent issues, '24h' for patterns, '7d' for trends.\n\
                3. **Check services first**: If filtering by service, call list_services to get valid names.\n\
                4. **Combine filters wisely**: Use level + query together for targeted results.\n\
                5. **Handle empty results**: If no results, try broadening the query or time range.\n\n\
                ## Response Format\n\n\
                - Summarize findings clearly with key insights first.\n\
                - Highlight error patterns, anomalies, or concerning trends.\n\
                - Provide actionable recommendations when issues are found.\n\
                - Format log snippets in code blocks for readability.\n\
                - If results are truncated, suggest how to narrow the search.",
                backend = self.backend_name.as_deref().unwrap_or("logs")
            ));
        } else {
            msg.push_str(
                "You can help with general questions about observability, logging best practices, \
                and troubleshooting strategies.\n\n\
                **Note**: No log backend is currently configured. To enable log analysis, \
                configure a backend (kibana, openobserve, or elasticsearch) in your config file.",
            );
        }

        msg
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = usize::MAX / 2;
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(f.size());

        self.render_header(f, chunks[0]);
        self.render_chat(f, chunks[1]);
        self.render_input(f, chunks[2]);

        if self.show_slash_modal {
            self.render_slash_modal(f, chunks[2]);
        }

        if self.show_resume_modal {
            self.render_resume_modal(f, chunks[1]);
        }
    }

    fn render_slash_modal(&self, f: &mut Frame, input_area: Rect) {
        let filtered_commands = self.get_filtered_commands();

        if filtered_commands.is_empty() {
            return;
        }

        let modal_height = (filtered_commands.len() + 2).min(10) as u16;
        let modal_width = 45u16.min(input_area.width.saturating_sub(8));

        let modal_area = Rect {
            x: input_area.x + 4,
            y: input_area.y.saturating_sub(modal_height + 1),
            width: modal_width,
            height: modal_height,
        };

        f.render_widget(Clear, modal_area);

        let mut lines: Vec<Line> = Vec::new();

        for (i, cmd) in filtered_commands.iter().enumerate() {
            let is_selected = i == self.slash_selected;
            let shortcut = cmd
                .shortcut
                .map(|s| format!(" ({})", s))
                .unwrap_or_default();

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(0, 122, 255))
            } else {
                Style::default().fg(Color::White)
            };

            let desc_style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(200, 200, 200))
                    .bg(Color::Rgb(0, 122, 255))
            } else {
                Style::default().fg(Color::Rgb(142, 142, 147))
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" /{}{}", cmd.name, shortcut), style),
                Span::styled(format!("  {}", cmd.description), desc_style),
            ]));
        }

        let modal = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(58, 58, 60)))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Commands ")
                    .title_style(Style::default().fg(Color::Rgb(142, 142, 147))),
            )
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));

        f.render_widget(modal, modal_area);
    }

    fn render_resume_modal(&self, f: &mut Frame, chat_area: Rect) {
        if self.resume_sessions.is_empty() {
            return;
        }

        let modal_height = (self.resume_sessions.len() + 2).min(12) as u16;
        let modal_width = 60u16.min(chat_area.width.saturating_sub(8));

        let modal_area = Rect {
            x: chat_area.x + (chat_area.width.saturating_sub(modal_width)) / 2,
            y: chat_area.y + (chat_area.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        f.render_widget(Clear, modal_area);

        let mut lines: Vec<Line> = Vec::new();

        for (i, session) in self.resume_sessions.iter().enumerate() {
            let is_selected = i == self.resume_selected;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(0, 122, 255))
            } else {
                Style::default().fg(Color::White)
            };

            let time_style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(200, 200, 200))
                    .bg(Color::Rgb(0, 122, 255))
            } else {
                Style::default().fg(Color::Rgb(142, 142, 147))
            };

            let time_ago = format_time_ago(session.updated_at);
            let msg_count = format!("{} msgs", session.message_count);

            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", session.title), style),
                Span::styled(format!("  {} • {}", time_ago, msg_count), time_style),
            ]));
        }

        let modal = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(58, 58, 60)))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Resume Conversation ")
                    .title_style(Style::default().fg(Color::Rgb(142, 142, 147))),
            )
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));

        f.render_widget(modal, modal_area);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let backend_indicator = if self.tool_executor.is_some() {
            format!(" [{}]", self.backend_name.as_deref().unwrap_or("logs"))
        } else {
            String::new()
        };

        let header = Paragraph::new(Line::from(vec![
            Span::styled("●", Style::default().fg(Color::Rgb(0, 122, 255))),
            Span::styled(
                "  zeteo",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                backend_indicator,
                Style::default().fg(Color::Rgb(142, 142, 147)),
            ),
        ]))
        .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    fn render_chat(&mut self, f: &mut Frame, area: Rect) {
        if self.show_welcome && self.messages.is_empty() {
            self.render_welcome(f, area);
            return;
        }

        let inner = Rect {
            x: area.x + 4,
            y: area.y,
            width: area.width.saturating_sub(8),
            height: area.height,
        };

        let mut lines: Vec<Line> = Vec::new();

        for msg in &self.messages {
            match msg.role.as_str() {
                "user" => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "You",
                        Style::default()
                            .fg(Color::Rgb(142, 142, 147))
                            .add_modifier(Modifier::BOLD),
                    )]));
                    for line in msg.content.lines() {
                        for wrapped in wrap_text(line, inner.width.saturating_sub(2) as usize) {
                            lines.push(Line::from(Span::styled(
                                wrapped,
                                Style::default().fg(Color::White),
                            )));
                        }
                    }
                }
                "assistant" => {
                    if msg.content.is_empty() && msg.tool_calls.is_some() {
                        continue;
                    }
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![Span::styled(
                        "Zeteo",
                        Style::default()
                            .fg(Color::Rgb(0, 122, 255))
                            .add_modifier(Modifier::BOLD),
                    )]));
                    let md_lines = markdown::parse_markdown_to_lines(
                        &msg.content,
                        inner.width.saturating_sub(2) as usize,
                    );
                    lines.extend(md_lines);
                }
                "tool" => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  📊 [Log query executed]",
                        Style::default().fg(Color::Rgb(142, 142, 147)),
                    )));
                }
                "error" => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        &msg.content,
                        Style::default().fg(Color::Rgb(255, 69, 58)),
                    )));
                }
                _ => {}
            }
        }

        if self.is_loading {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Zeteo",
                Style::default()
                    .fg(Color::Rgb(0, 122, 255))
                    .add_modifier(Modifier::BOLD),
            )]));
            let loading_text = self.status_message.as_deref().unwrap_or("...");
            lines.push(Line::from(Span::styled(
                loading_text,
                Style::default().fg(Color::Rgb(142, 142, 147)),
            )));
        }

        let total = lines.len();
        let visible = inner.height as usize;
        let max_scroll = total.saturating_sub(visible);

        self.scroll_offset = self.scroll_offset.min(max_scroll);
        let scroll = self.scroll_offset;

        let chat = Paragraph::new(lines).scroll((scroll as u16, 0));
        f.render_widget(chat, inner);

        if total > visible {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some(" "))
                .thumb_symbol("│");

            let mut state = ScrollbarState::new(max_scroll).position(scroll);
            f.render_stateful_widget(scrollbar, area, &mut state);
        }
    }

    fn render_welcome(&self, f: &mut Frame, area: Rect) {
        let center_y = area.height / 2;

        let backend_status = if self.tool_executor.is_some() {
            format!(
                "Connected to {}",
                self.backend_name.as_deref().unwrap_or("log backend")
            )
        } else {
            "No log backend configured".to_string()
        };

        let welcome = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "●",
                Style::default().fg(Color::Rgb(0, 122, 255)),
            )]),
            Line::from(""),
            Line::from(Span::styled(
                "How can I help you today?",
                Style::default().fg(Color::Rgb(142, 142, 147)),
            )),
            Line::from(""),
            Line::from(Span::styled(
                backend_status,
                Style::default().fg(Color::Rgb(100, 100, 100)),
            )),
        ])
        .alignment(Alignment::Center);

        let welcome_area = Rect {
            x: area.x,
            y: area.y + center_y.saturating_sub(3),
            width: area.width,
            height: 7,
        };

        f.render_widget(welcome, welcome_area);
    }

    fn render_input(&self, f: &mut Frame, area: Rect) {
        let inner = Rect {
            x: area.x + 4,
            y: area.y,
            width: area.width.saturating_sub(8),
            height: area.height,
        };

        let prompt = if self.is_loading {
            Span::styled("◉ ", Style::default().fg(Color::Rgb(255, 159, 10)))
        } else {
            Span::styled("› ", Style::default().fg(Color::Rgb(0, 122, 255)))
        };

        let (text_before_cursor, cursor_char, text_after_cursor) = if self.is_loading {
            let status = self.status_message.as_deref().unwrap_or("Processing...");
            (status.to_string(), String::new(), String::new())
        } else if self.input.is_empty() {
            (
                "Ask about logs, errors, or any question...".to_string(),
                String::new(),
                String::new(),
            )
        } else {
            let before = self.input[..self.cursor_position].to_string();
            let cursor = if self.cursor_position < self.input.len() {
                self.input[self.cursor_position..self.cursor_position + 1].to_string()
            } else {
                " ".to_string()
            };
            let after = if self.cursor_position < self.input.len() {
                self.input[self.cursor_position + 1..].to_string()
            } else {
                String::new()
            };
            (before, cursor, after)
        };

        let text_style = if self.input.is_empty() || self.is_loading {
            Style::default().fg(Color::Rgb(142, 142, 147))
        } else {
            Style::default().fg(Color::White)
        };

        let mut spans = vec![prompt];
        spans.push(Span::styled(text_before_cursor, text_style));

        if !self.is_loading && !self.input.is_empty() && self.cursor_visible {
            spans.push(Span::styled(
                cursor_char.clone(),
                Style::default().fg(Color::Black).bg(Color::White),
            ));
        } else if !self.is_loading && !self.input.is_empty() {
            spans.push(Span::styled(cursor_char.clone(), text_style));
        } else if !self.is_loading && self.input.is_empty() && self.cursor_visible {
        }

        spans.push(Span::styled(text_after_cursor, text_style));

        let input_line = Line::from(spans);

        let border_color = if self.is_loading {
            Color::Rgb(255, 159, 10)
        } else if !self.input.is_empty() {
            Color::Rgb(0, 122, 255)
        } else {
            Color::Rgb(58, 58, 60)
        };

        let input = Paragraph::new(input_line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .border_type(ratatui::widgets::BorderType::Rounded),
        );

        f.render_widget(input, inner);
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Formats a Unix timestamp (seconds since epoch) as a human-readable relative time string.
/// Returns strings like "just now", "5m ago", "2h ago", or "3d ago".
fn format_time_ago(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{}m ago", mins)
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{}h ago", hours)
    } else {
        let days = diff / 86400;
        format!("{}d ago", days)
    }
}

fn try_provider(name: &str) -> Option<Arc<dyn AiProvider>> {
    match name {
        "openai" => {
            let key = std::env::var("OPENAI_API_KEY").ok()?;
            Some(Arc::new(crate::providers::OpenAiProvider::new(key, None)))
        }
        "google" => {
            let key = std::env::var("GOOGLE_API_KEY").ok()?;
            Some(Arc::new(crate::providers::GoogleProvider::new(key, None)))
        }
        "azure" => {
            let key = std::env::var("AZURE_OPENAI_API_KEY").ok()?;
            let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT").ok()?;
            let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT").ok()?;
            Some(Arc::new(crate::providers::AzureProvider::new(
                key, endpoint, deployment,
            )))
        }
        "vertex" => {
            let project = std::env::var("GOOGLE_CLOUD_PROJECT").ok()?;
            let location = std::env::var("GOOGLE_CLOUD_LOCATION")
                .unwrap_or_else(|_| "us-central1".to_string());
            Some(Arc::new(crate::providers::VertexProvider::new(
                project, location, None,
            )))
        }
        _ => None,
    }
}

fn find_provider() -> Option<Arc<dyn AiProvider>> {
    ["openai", "google", "azure", "vertex"]
        .iter()
        .find_map(|p| try_provider(p))
}

fn try_backend(name: &str, config: &Config) -> Option<(Arc<dyn LogBackendClient>, String)> {
    let backend_config = config.backends.get(name)?;

    match backend_config {
        LogBackend::Elasticsearch {
            url,
            username,
            password,
            index_pattern,
            verify_ssl,
        } => ElasticsearchClient::new(
            url.clone(),
            username.clone(),
            password.clone(),
            index_pattern.clone(),
            *verify_ssl,
        )
        .ok()
        .map(|c| {
            (
                Arc::new(c) as Arc<dyn LogBackendClient>,
                "elasticsearch".to_string(),
            )
        }),
        LogBackend::Kibana {
            url,
            auth_token,
            index_pattern,
            verify_ssl,
            version,
        } => KibanaClient::new(
            url.clone(),
            auth_token.clone(),
            index_pattern.clone(),
            version.clone(),
            *verify_ssl,
        )
        .ok()
        .map(|c| {
            (
                Arc::new(c) as Arc<dyn LogBackendClient>,
                "kibana".to_string(),
            )
        }),
        LogBackend::OpenObserve {
            url,
            username,
            password,
            organization,
            stream,
            verify_ssl,
        } => OpenObserveClient::new(
            url.clone(),
            username.clone(),
            password.clone(),
            organization.clone(),
            stream.clone(),
            *verify_ssl,
        )
        .ok()
        .map(|c| {
            (
                Arc::new(c) as Arc<dyn LogBackendClient>,
                "openobserve".to_string(),
            )
        }),
    }
}

fn find_backend(config: &Config) -> Option<(Arc<dyn LogBackendClient>, String)> {
    ["openobserve", "kibana", "elasticsearch"]
        .iter()
        .find_map(|name| try_backend(name, config))
}

pub async fn create_tui_session(
    provider: Option<String>,
    backend: Option<String>,
) -> Result<TuiApp> {
    let provider = match provider {
        Some(name) => try_provider(&name.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not configured", name))?,
        None => find_provider()
            .ok_or_else(|| anyhow::anyhow!("No AI provider configured. Set OPENAI_API_KEY, AZURE_OPENAI_API_KEY, or GOOGLE_API_KEY."))?,
    };

    let config = Config::load().ok();

    let (tool_executor, backend_name) = if let Some(ref cfg) = config {
        let backend_result = match backend {
            Some(name) => try_backend(&name.to_lowercase(), cfg)
                .ok_or_else(|| anyhow::anyhow!("Backend '{}' not found in config", name))
                .ok(),
            None => find_backend(cfg),
        };

        if let Some((client, name)) = backend_result {
            (Some(ToolExecutor::new(client)), Some(name))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let session_store = try_create_session_store().await;

    Ok(TuiApp::new(
        provider,
        tool_executor,
        backend_name,
        config,
        session_store,
    ))
}
