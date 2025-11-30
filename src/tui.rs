use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;

use crate::providers::{AiProvider, ChatRequest, Message};

struct ChatMessage {
    role: String,
    content: String,
}

pub struct TuiApp {
    provider: Arc<dyn AiProvider>,
    input: String,
    cursor_position: usize,
    messages: Vec<ChatMessage>,
    scroll_offset: usize,
    is_loading: bool,
    show_welcome: bool,
}

impl TuiApp {
    pub fn new(provider: Arc<dyn AiProvider>) -> Self {
        Self {
            provider,
            input: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            is_loading: false,
            show_welcome: true,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

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

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Enter => {
                            if !self.input.trim().is_empty() && !self.is_loading {
                                self.show_welcome = false;
                                let input = self.input.clone();
                                self.input.clear();
                                self.cursor_position = 0;
                                
                                if input.trim() == "/quit" || input.trim() == "/q" {
                                    return Ok(());
                                }
                                
                                if input.trim() == "/clear" {
                                    self.messages.clear();
                                    self.scroll_offset = 0;
                                    continue;
                                }
                                
                                if let Err(e) = self.send_message(input).await {
                                    self.messages.push(ChatMessage {
                                        role: "error".to_string(),
                                        content: e.to_string(),
                                    });
                                }
                                self.scroll_to_bottom();
                            }
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
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn send_message(&mut self, input: String) -> Result<()> {
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.clone(),
        });

        self.is_loading = true;

        let messages: Vec<Message> = self.messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = ChatRequest {
            messages,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let response = self.provider.chat(request).await?;

        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.content,
        });

        self.is_loading = false;
        Ok(())
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }

    fn ui(&self, f: &mut Frame) {
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
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(Line::from(vec![
            Span::styled("●", Style::default().fg(Color::Rgb(0, 122, 255))),
            Span::styled("  zeteo", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]))
        .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    fn render_chat(&self, f: &mut Frame, area: Rect) {
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
                    lines.push(Line::from(vec![
                        Span::styled("You", Style::default().fg(Color::Rgb(142, 142, 147)).add_modifier(Modifier::BOLD)),
                    ]));
                    for line in msg.content.lines() {
                        for wrapped in wrap_text(line, inner.width.saturating_sub(2) as usize) {
                            lines.push(Line::from(Span::styled(wrapped, Style::default().fg(Color::White))));
                        }
                    }
                }
                "assistant" => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Zeteo", Style::default().fg(Color::Rgb(0, 122, 255)).add_modifier(Modifier::BOLD)),
                    ]));
                    for line in msg.content.lines() {
                        for wrapped in wrap_text(line, inner.width.saturating_sub(2) as usize) {
                            lines.push(Line::from(Span::styled(wrapped, Style::default().fg(Color::Rgb(229, 229, 234)))));
                        }
                    }
                }
                "error" => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(&msg.content, Style::default().fg(Color::Rgb(255, 69, 58)))));
                }
                _ => {}
            }
        }

        if self.is_loading {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Zeteo", Style::default().fg(Color::Rgb(0, 122, 255)).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(Span::styled("...", Style::default().fg(Color::Rgb(142, 142, 147)))));
        }

        let total = lines.len();
        let visible = inner.height as usize;
        let max_scroll = total.saturating_sub(visible);
        let scroll = self.scroll_offset.min(max_scroll);

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
        
        let welcome = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("●", Style::default().fg(Color::Rgb(0, 122, 255))),
            ]),
            Line::from(""),
            Line::from(Span::styled("How can I help you today?", Style::default().fg(Color::Rgb(142, 142, 147)))),
        ])
        .alignment(Alignment::Center);

        let welcome_area = Rect {
            x: area.x,
            y: area.y + center_y.saturating_sub(2),
            width: area.width,
            height: 5,
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

        let display = if self.is_loading {
            "...".to_string()
        } else if self.input.is_empty() {
            "Message".to_string()
        } else {
            self.input.clone()
        };

        let style = if self.input.is_empty() && !self.is_loading {
            Style::default().fg(Color::Rgb(142, 142, 147))
        } else {
            Style::default().fg(Color::White)
        };

        let input = Paragraph::new(display)
            .style(style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(58, 58, 60)))
            );

        f.render_widget(input, inner);

        if !self.is_loading && !self.input.is_empty() {
            let cursor_x = inner.x + 1 + self.cursor_position as u16;
            let cursor_y = inner.y + 1;
            if cursor_x < inner.x + inner.width - 1 {
                f.set_cursor(cursor_x, cursor_y);
            }
        }
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
            Some(Arc::new(crate::providers::AzureProvider::new(key, endpoint, deployment)))
        }
        "vertex" => {
            let project = std::env::var("GOOGLE_CLOUD_PROJECT").ok()?;
            let location = std::env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "us-central1".to_string());
            Some(Arc::new(crate::providers::VertexProvider::new(project, location, None)))
        }
        _ => None,
    }
}

fn find_provider() -> Option<Arc<dyn AiProvider>> {
    ["openai", "google", "azure", "vertex"]
        .iter()
        .find_map(|p| try_provider(p))
}

pub async fn create_tui_session(provider: Option<String>) -> Result<TuiApp> {
    let provider = match provider {
        Some(name) => try_provider(&name.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not configured", name))?,
        None => find_provider()
            .ok_or_else(|| anyhow::anyhow!("No provider configured"))?,
    };

    Ok(TuiApp::new(provider))
}
