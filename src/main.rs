mod app;
mod file_io;
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

use app::{App, PlanFocus, Screen};

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
                    Screen::PlanView => handle_plan_view_input(app, key.code),
                    Screen::SavePlan => handle_save_input(app, key.code),
                    Screen::LoadPlan => handle_load_input(app, key.code),
                    Screen::EditWorkout => handle_edit_input(app, key.code),
                }

                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_plan_view_input(app: &mut App, key: KeyCode) {
    match key {
        // Global keys
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('s') | KeyCode::Char('S') => app.go_to_save(),
        KeyCode::Char('l') | KeyCode::Char('L') => app.go_to_load(),

        // Tab switches focus between panels
        KeyCode::Tab => app.toggle_focus(),

        // Panel-specific handling
        _ => match app.plan_focus {
            PlanFocus::Summary => handle_summary_input(app, key),
            PlanFocus::Weeks => handle_weeks_input(app, key),
        },
    }
}

fn handle_summary_input(app: &mut App, key: KeyCode) {
    match key {
        // Navigate between fields
        KeyCode::Up | KeyCode::Char('k') => app.select_prev_summary_field(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next_summary_field(),

        // Modify selected field value
        KeyCode::Left | KeyCode::Char('h') => app.decrement_summary_field(),
        KeyCode::Right => app.increment_summary_field(),

        _ => {}
    }
}

fn handle_weeks_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.scroll_plan_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_plan_down(),
        KeyCode::Left | KeyCode::Char('h') => app.select_prev_workout(),
        KeyCode::Right | KeyCode::Char('l') => app.select_next_workout(),
        KeyCode::Char('e') | KeyCode::Char('E') => app.go_to_edit(),
        _ => {}
    }
}

fn handle_save_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            app.file_path_input.push(c);
        }
        KeyCode::Backspace => {
            app.file_path_input.pop();
        }
        KeyCode::Enter => {
            app.save_plan();
        }
        KeyCode::Esc => {
            app.screen = Screen::PlanView;
        }
        _ => {}
    }
}

fn handle_load_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            app.file_path_input.push(c);
        }
        KeyCode::Backspace => {
            app.file_path_input.pop();
        }
        KeyCode::Enter => {
            app.load_plan();
        }
        KeyCode::Esc => {
            app.screen = Screen::PlanView;
        }
        _ => {}
    }
}

fn handle_edit_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            app.edit_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.edit_buffer.pop();
        }
        KeyCode::Enter => {
            app.apply_edit();
        }
        KeyCode::Esc => {
            app.screen = Screen::PlanView;
        }
        _ => {}
    }
}
