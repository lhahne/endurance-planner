mod app;
mod models;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{App, Screen};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, not releases
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.screen {
                    Screen::Welcome => handle_welcome_input(app, key.code),
                    Screen::AgeInput => handle_age_input(app, key.code),
                    Screen::DistanceSelect => handle_distance_input(app, key.code),
                    Screen::RaceTypeSelect => handle_race_type_input(app, key.code),
                    Screen::WorkoutsPerWeekSelect => handle_workouts_input(app, key.code),
                    Screen::PlanView => handle_plan_view_input(app, key.code),
                }

                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_welcome_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.next_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

fn handle_age_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if app.age_input.len() < 3 {
                app.age_input.push(c);
            }
        }
        KeyCode::Backspace => {
            app.age_input.pop();
        }
        KeyCode::Enter => {
            if !app.age_input.is_empty() {
                app.next_screen();
            }
        }
        KeyCode::Esc => app.previous_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_distance_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.select_prev_distance(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next_distance(),
        KeyCode::Enter => app.next_screen(),
        KeyCode::Esc => app.previous_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_race_type_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.select_prev_race_type(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next_race_type(),
        KeyCode::Enter => app.next_screen(),
        KeyCode::Esc => app.previous_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_workouts_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.increase_workouts(),
        KeyCode::Down | KeyCode::Char('j') => app.decrease_workouts(),
        KeyCode::Enter => app.next_screen(),
        KeyCode::Esc => app.previous_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_plan_view_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.scroll_plan_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_plan_down(),
        KeyCode::Esc => app.previous_screen(),
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}
