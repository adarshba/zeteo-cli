use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Instant;

use crate::providers::{AiProvider, ChatRequest, Message};

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, PartialEq)]
enum FocusedPanel {
    Chat,
    Logs,
    Input,
}

pub struct TuiApp {
    provider: Arc<dyn AiProvider>,
    provider_name: String,
    input: String,
    input_mode: InputMode,
    focused_panel: FocusedPanel,
    messages: Vec<Message>,
    logs: Vec<String>,
    status_message: String,
    show_help: bool,
    session_start: Instant,
}

impl TuiApp {
    pub fn new(provider: Arc<dyn AiProvider>, provider_name: String) -> Self {
        let status_msg = format!("Connected to AI provider: {}", provider_name);
        Self {
            provider,
            provider_name,
            input: String::new(),
            input_mode: InputMode::Normal,
            focused_panel: FocusedPanel::Input,
            messages: Vec::new(),
            logs: Vec::new(),
            status_message: status_msg,
            show_help: false,
            session_start: Instant::now(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('h') => {
                                self.show_help = !self.show_help;
                            }
                            KeyCode::Char('i') => {
                                self.input_mode = InputMode::Editing;
                                self.focused_panel = FocusedPanel::Input;
                            }
                            KeyCode::Tab => {
                                self.cycle_focus();
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                let input = self.input.drain(..).collect::<String>();
                                if !input.trim().is_empty() {
                                    if let Err(e) = self.handle_input(input).await {
                                        self.status_message = format!("Error: {}", e);
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                self.input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Chat => FocusedPanel::Logs,
            FocusedPanel::Logs => FocusedPanel::Input,
            FocusedPanel::Input => FocusedPanel::Chat,
        };
    }

    async fn handle_input(&mut self, input: String) -> Result<()> {
        // Handle commands
        if input.starts_with('/') {
            return self.handle_command(&input);
        }

        // Add user message
        self.messages.push(Message {
            role: "user".to_string(),
            content: input.clone(),
        });

        self.status_message = "Thinking...".to_string();

        // Get AI response
        let request = ChatRequest {
            messages: self.messages.clone(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };

        let response = self.provider.chat(request).await?;

        // Add AI response
        self.messages.push(Message {
            role: "assistant".to_string(),
            content: response.content,
        });

        self.status_message = format!("Response received ({})", response.model);

        Ok(())
    }

    fn handle_command(&mut self, command: &str) -> Result<()> {
        match command.trim() {
            "/clear" => {
                self.messages.clear();
                self.status_message = "Conversation cleared".to_string();
            }
            "/logs" => {
                self.focused_panel = FocusedPanel::Logs;
                self.status_message = "Switched to logs panel".to_string();
            }
            "/help" => {
                self.show_help = !self.show_help;
            }
            _ => {
                self.status_message = format!("Unknown command: {}", command);
            }
        }
        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        if self.show_help {
            self.render_help(f);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Title
                Constraint::Min(10),        // Main content
                Constraint::Length(3),      // Input
                Constraint::Length(3),      // Status
            ])
            .split(f.size());

        // Title
        self.render_title(f, chunks[0]);

        // Main content (split into chat and logs)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Chat
                Constraint::Percentage(40),  // Logs
            ])
            .split(chunks[1]);

        self.render_chat(f, main_chunks[0]);
        self.render_logs(f, main_chunks[1]);

        // Input
        self.render_input(f, chunks[2]);

        // Status
        self.render_status(f, chunks[3]);
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let provider_icon = match self.provider_name.to_lowercase().as_str() {
            "openai" => "🤖",
            "vertex" => "🔷",
            "google" => "🔵",
            "azure" => "☁️",
            _ => "✨",
        };

        let title = Paragraph::new(format!(
            "ZETEO - TUI Mode {} Provider: {} | Press 'h' for help, 'q' to quit",
            provider_icon, self.provider_name
        ))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

        f.render_widget(title, area);
    }

    fn render_chat(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == FocusedPanel::Chat;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| {
                let icon = if m.role == "user" { "👤" } else { "🤖" };
                let style = if m.role == "user" {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Blue)
                };

                let content = if m.content.len() > 100 {
                    format!("{}: {}...", icon, &m.content[..97])
                } else {
                    format!("{}: {}", icon, m.content)
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(messages)
            .block(
                Block::default()
                    .title(format!(" Chat ({} messages) ", self.messages.len()))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        f.render_widget(list, area);
    }

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == FocusedPanel::Logs;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let log_items: Vec<ListItem> = if self.logs.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No logs available. Use MCP server to query logs.",
                Style::default().fg(Color::DarkGray),
            )))]
        } else {
            self.logs
                .iter()
                .map(|log| {
                    ListItem::new(Line::from(Span::styled(
                        log.clone(),
                        Style::default().fg(Color::White),
                    )))
                })
                .collect()
        };

        let list = List::new(log_items)
            .block(
                Block::default()
                    .title(format!(" Logs ({}) ", self.logs.len()))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        f.render_widget(list, area);
    }

    fn render_input(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focused_panel == FocusedPanel::Input;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let mode_indicator = match self.input_mode {
            InputMode::Normal => " [NORMAL] Press 'i' to edit ",
            InputMode::Editing => " [EDITING] Press ESC to exit, ENTER to send ",
        };

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(mode_indicator)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        f.render_widget(input, area);
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let elapsed = self.session_start.elapsed();
        let duration = format!(
            "{}h {}m {}s",
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60
        );

        let status = Paragraph::new(format!(
            "Status: {} | Messages: {} | Session: {}",
            self.status_message,
            self.messages.len(),
            duration
        ))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

        f.render_widget(status, area);
    }

    fn render_help(&self, f: &mut Frame) {
        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "ZETEO TUI - Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw("        - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("h", Style::default().fg(Color::Yellow)),
                Span::raw("        - Toggle this help screen"),
            ]),
            Line::from(vec![
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw("        - Enter insert mode (edit input)"),
            ]),
            Line::from(vec![
                Span::styled("ESC", Style::default().fg(Color::Yellow)),
                Span::raw("      - Exit insert mode"),
            ]),
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw("      - Cycle focus between panels"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw("    - Send message (in insert mode)"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
                Span::raw("   - Force quit"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Commands (type in input):",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("/clear", Style::default().fg(Color::Yellow)),
                Span::raw("   - Clear conversation history"),
            ]),
            Line::from(vec![
                Span::styled("/logs", Style::default().fg(Color::Yellow)),
                Span::raw("    - Switch focus to logs panel"),
            ]),
            Line::from(vec![
                Span::styled("/help", Style::default().fg(Color::Yellow)),
                Span::raw("    - Toggle this help screen"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'h' again to close help",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true });

        let area = centered_rect(80, 80, f.size());
        f.render_widget(help, area);
    }
}

// Helper function to create a centered rectangle
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

pub async fn create_tui_session(provider: Option<String>) -> Result<TuiApp> {
    let provider_name = provider.unwrap_or_else(|| "openai".to_string());
    
    let provider: Arc<dyn AiProvider> = match provider_name.to_lowercase().as_str() {
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
            Arc::new(crate::providers::OpenAiProvider::new(api_key, None))
        }
        "vertex" => {
            let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
                .map_err(|_| anyhow::anyhow!("GOOGLE_CLOUD_PROJECT not set"))?;
            let location = std::env::var("GOOGLE_CLOUD_LOCATION")
                .unwrap_or_else(|_| "us-central1".to_string());
            Arc::new(crate::providers::VertexProvider::new(project_id, location, None))
        }
        "google" => {
            let api_key = std::env::var("GOOGLE_API_KEY")
                .map_err(|_| anyhow::anyhow!("GOOGLE_API_KEY not set"))?;
            Arc::new(crate::providers::GoogleProvider::new(api_key, None))
        }
        "azure" => {
            let api_key = std::env::var("AZURE_OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_API_KEY not set"))?;
            let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_ENDPOINT not set"))?;
            let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_DEPLOYMENT not set"))?;
            Arc::new(crate::providers::AzureProvider::new(api_key, endpoint, deployment))
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown provider: {}. Supported: openai, vertex, google, azure",
                provider_name
            ));
        }
    };

    Ok(TuiApp::new(provider, provider_name))
}
