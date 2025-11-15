//! Icarus Learning TUI - Real-time Knowledge Distillation Application
//!
//! A terminal user interface for teaching Icarus through knowledge distillation.
//! Watch skills being extracted and stored in real-time.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use icarus_core::learning::{Interaction, SkillLibrary, StrategyExtractor, Skill};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, Gauge},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Initializing,
    TeachingSession { session: usize, step: usize },
    ExtractingKnowledge { session: usize },
    SkillLearned { session: usize, skill: Skill },
    Complete,
}

struct App {
    state: AppState,
    library: SkillLibrary,
    sessions: Vec<SessionData>,
    current_session: usize,
    extraction_progress: f64,
    skills_learned: Vec<Skill>,
    should_quit: bool,
}

struct SessionData {
    title: String,
    problem: String,
    reasoning: Vec<String>,
    solution: String,
    context: HashMap<String, String>,
}

impl App {
    fn new() -> Self {
        let sessions = vec![
            SessionData {
                title: "Advanced Debugging Workflow".to_string(),
                problem: "Production bug: API returning 500 errors intermittently for user authentication endpoint".to_string(),
                reasoning: vec![
                    "Check application logs for stack traces and error patterns".to_string(),
                    "Reproduce the issue locally with the same user data".to_string(),
                    "Add detailed logging around the authentication logic".to_string(),
                    "Identify that the error occurs when JWT token is expired but cache isn't cleared".to_string(),
                    "Implement proper cache invalidation on token expiry".to_string(),
                    "Add monitoring alerts for 500 errors on auth endpoint".to_string(),
                    "Deploy fix and verify error rate drops to zero".to_string(),
                ],
                solution: "Fixed intermittent 500 errors by adding cache invalidation on JWT expiry and implementing proper error monitoring".to_string(),
                context: HashMap::from([
                    ("domain".to_string(), "debugging".to_string()),
                    ("severity".to_string(), "production".to_string()),
                ]),
            },
            SessionData {
                title: "Performance Optimization".to_string(),
                problem: "Dashboard loading time increased from 2s to 15s after adding real-time data updates".to_string(),
                reasoning: vec![
                    "Profile the application to identify bottlenecks".to_string(),
                    "Discover N+1 query problem: fetching related data in a loop".to_string(),
                    "Implement database query optimization with JOIN instead of multiple queries".to_string(),
                    "Add caching layer for frequently accessed data".to_string(),
                    "Implement pagination to limit initial data load".to_string(),
                    "Add lazy loading for non-critical dashboard widgets".to_string(),
                    "Verify load time reduced to 2.5s with profiling tools".to_string(),
                ],
                solution: "Optimized dashboard from 15s to 2.5s by fixing N+1 queries, adding caching, and implementing pagination".to_string(),
                context: HashMap::from([
                    ("domain".to_string(), "performance".to_string()),
                    ("metric".to_string(), "load_time".to_string()),
                    ("improvement".to_string(), "83%".to_string()),
                ]),
            },
            SessionData {
                title: "Code Refactoring".to_string(),
                problem: "Legacy payment processing module has 800-line function with complex nested conditionals".to_string(),
                reasoning: vec![
                    "Analyze the function to understand all code paths and responsibilities".to_string(),
                    "Identify 5 distinct concerns: validation, calculation, external API calls, database updates, notifications".to_string(),
                    "Extract validation logic into PaymentValidator class with clear interface".to_string(),
                    "Extract calculation into PaymentCalculator with testable methods".to_string(),
                    "Extract API integration into PaymentGateway adapter".to_string(),
                    "Extract database operations into PaymentRepository".to_string(),
                    "Create PaymentOrchestrator to coordinate these components".to_string(),
                    "Add comprehensive unit tests for each extracted component".to_string(),
                    "Refactor original function to use orchestrator - now 50 lines".to_string(),
                ],
                solution: "Refactored 800-line function into 5 focused classes with single responsibilities, reduced to 50-line orchestrator".to_string(),
                context: HashMap::from([
                    ("domain".to_string(), "refactoring".to_string()),
                    ("principle".to_string(), "single_responsibility".to_string()),
                    ("before_lines".to_string(), "800".to_string()),
                    ("after_lines".to_string(), "50".to_string()),
                ]),
            },
        ];

        Self {
            state: AppState::Initializing,
            library: SkillLibrary::new(),
            sessions,
            current_session: 0,
            extraction_progress: 0.0,
            skills_learned: Vec::new(),
            should_quit: false,
        }
    }

    async fn run_learning_cycle(&mut self) {
        // Initializing
        self.state = AppState::Initializing;
        sleep(Duration::from_millis(1000)).await;

        // Process each session
        for session_idx in 0..self.sessions.len() {
            self.current_session = session_idx;
            let session = &self.sessions[session_idx];

            // Show each reasoning step
            for step_idx in 0..session.reasoning.len() {
                self.state = AppState::TeachingSession {
                    session: session_idx,
                    step: step_idx,
                };
                sleep(Duration::from_millis(800)).await;
            }

            // Extract knowledge
            self.state = AppState::ExtractingKnowledge {
                session: session_idx,
            };

            // Animate extraction progress
            for i in 0..=100 {
                self.extraction_progress = i as f64 / 100.0;
                sleep(Duration::from_millis(20)).await;
            }

            // Create interaction and extract skill
            let interaction = Interaction {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                problem: session.problem.clone(),
                claude_reasoning: session.reasoning.clone(),
                solution: session.solution.clone(),
                context: session.context.clone(),
            };

            if let Ok(Some(skill)) = StrategyExtractor::extract_skill(&interaction) {
                let skill_clone = skill.clone();
                if self.library.add_skill(skill).await.is_ok() {
                    self.skills_learned.push(skill_clone.clone());
                    self.state = AppState::SkillLearned {
                        session: session_idx,
                        skill: skill_clone,
                    };
                    sleep(Duration::from_millis(2000)).await;
                }
            }
        }

        // Complete
        self.state = AppState::Complete;
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),      // Main content
            Constraint::Length(5),   // Stats
        ])
        .split(size);

    // Title
    let title = Paragraph::new("🧠 ICARUS KNOWLEDGE DISTILLATION")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(title, chunks[0]);

    // Main content
    match &app.state {
        AppState::Initializing => {
            let text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Initializing Skill Library...",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "⏳ Preparing knowledge distillation system",
                    Style::default().fg(Color::Gray),
                )),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(text, chunks[1]);
        }

        AppState::TeachingSession { session, step } => {
            let session_data = &app.sessions[*session];

            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),  // Session title
                    Constraint::Length(6),  // Problem
                    Constraint::Min(0),     // Reasoning steps
                ])
                .split(chunks[1]);

            // Session title
            let session_title = Paragraph::new(format!("SESSION {}: {}", session + 1, session_data.title))
                .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
            f.render_widget(session_title, content_chunks[0]);

            // Problem
            let problem = Paragraph::new(session_data.problem.as_str())
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title("📖 Problem").border_style(Style::default().fg(Color::Blue)));
            f.render_widget(problem, content_chunks[1]);

            // Reasoning steps
            let items: Vec<ListItem> = session_data
                .reasoning
                .iter()
                .enumerate()
                .map(|(i, reasoning)| {
                    let style = if i <= *step {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let icon = if i == *step { "▶" } else if i < *step { "✓" } else { "○" };

                    ListItem::new(format!(" {} Step {}: {}", icon, i + 1, reasoning)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("🤔 Claude's Reasoning").border_style(Style::default().fg(Color::Yellow)));
            f.render_widget(list, content_chunks[2]);
        }

        AppState::ExtractingKnowledge { session } => {
            let session_data = &app.sessions[*session];

            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(chunks[1]);

            let labels = [
                "Analyzing reasoning patterns...",
                "Extracting reusable strategy...",
                "Creating skill representation...",
                "Storing in library...",
            ];

            for (i, label) in labels.iter().enumerate() {
                let progress = if app.extraction_progress >= ((i + 1) as f64 / labels.len() as f64) {
                    1.0
                } else if app.extraction_progress >= (i as f64 / labels.len() as f64) {
                    (app.extraction_progress - (i as f64 / labels.len() as f64)) * labels.len() as f64
                } else {
                    0.0
                };

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title(*label))
                    .gauge_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    .ratio(progress);
                f.render_widget(gauge, content_chunks[i]);
            }
        }

        AppState::SkillLearned { session, skill } => {
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(chunks[1]);

            let title = Paragraph::new("✅ SKILL LEARNED!")
                .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
            f.render_widget(title, content_chunks[0]);

            let details = vec![
                Line::from(vec![
                    Span::styled("Skill ID:      ", Style::default().fg(Color::Cyan)),
                    Span::raw(&skill.id),
                ]),
                Line::from(vec![
                    Span::styled("Name:          ", Style::default().fg(Color::Cyan)),
                    Span::raw(&skill.name),
                ]),
                Line::from(vec![
                    Span::styled("Domain:        ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:?}", skill.domain)),
                ]),
                Line::from(vec![
                    Span::styled("Steps:         ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{}", skill.steps.len())),
                ]),
                Line::from(vec![
                    Span::styled("Success Rate:  ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:.2}", skill.success_rate)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Pattern: ", Style::default().fg(Color::Yellow)),
                    Span::raw(&skill.pattern),
                ]),
            ];

            let details_widget = Paragraph::new(details)
                .block(Block::default().borders(Borders::ALL).title("Skill Details"))
                .wrap(Wrap { trim: true });
            f.render_widget(details_widget, content_chunks[1]);
        }

        AppState::Complete => {
            let text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "🎓 KNOWLEDGE DISTILLATION COMPLETE",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("✨ {} skills learned and stored in library", app.skills_learned.len()),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "📚 Icarus is ready to apply learned knowledge!",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'q' to quit",
                    Style::default().fg(Color::Gray),
                )),
            ];

            let widget = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(widget, chunks[1]);
        }
    }

    // Stats footer
    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Total Skills: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", app.skills_learned.len())),
            Span::raw("  |  "),
            Span::styled("Session: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}/{}", app.current_session + 1, app.sessions.len())),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:?}", app.state).split('{'). next().unwrap_or(""),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Gray)),
            Span::raw("q = quit"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Status").border_style(Style::default().fg(Color::Gray)));
    f.render_widget(stats, chunks[2]);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run learning cycle in background
    let app_clone = tokio::spawn(async move {
        let mut app = App::new();
        app.run_learning_cycle().await;
        app
    });

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Check if background task updated state
            if !app_clone.is_finished() {
                // Task still running - we'd normally get updates via channels
                // For this demo, we'll restart the cycle
            } else if let Ok(updated_app) = app_clone.try_into_inner() {
                app = updated_app;
                if app.state == AppState::Complete {
                    // Keep showing complete screen
                }
            }

            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
