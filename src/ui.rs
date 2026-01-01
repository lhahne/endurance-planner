use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Screen};
use crate::models::{Distance, RaceType, TrainingPhase};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Endurance Training Planner")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.screen {
        Screen::Welcome => render_welcome(frame, area),
        Screen::AgeInput => render_age_input(frame, app, area),
        Screen::DistanceSelect => render_distance_select(frame, app, area),
        Screen::RaceTypeSelect => render_race_type_select(frame, app, area),
        Screen::WorkoutsPerWeekSelect => render_workouts_select(frame, app, area),
        Screen::PlanView => render_plan_view(frame, app, area),
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.screen {
        Screen::Welcome => "Press Enter to start | q to quit",
        Screen::AgeInput => "Enter your age | Enter to continue | Esc to go back | q to quit",
        Screen::DistanceSelect => "Up/Down to select | Enter to continue | Esc to go back",
        Screen::RaceTypeSelect => "Up/Down to select | Enter to continue | Esc to go back",
        Screen::WorkoutsPerWeekSelect => {
            "Up/Down to adjust | Enter to generate plan | Esc to go back"
        }
        Screen::PlanView => "Up/Down to scroll weeks | Esc to go back | q to quit",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, area);
}

fn render_welcome(frame: &mut Frame, area: Rect) {
    let welcome_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to Endurance Training Planner!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("This application will help you create a personalized"),
        Line::from("training plan based on:"),
        Line::from(""),
        Line::from(Span::styled("  * ", Style::default().fg(Color::Yellow))),
        Line::from("    Your age (for Maffetone heart rate zones)"),
        Line::from(Span::styled("  * ", Style::default().fg(Color::Yellow))),
        Line::from("    Target race distance"),
        Line::from(Span::styled("  * ", Style::default().fg(Color::Yellow))),
        Line::from("    Race type (road or trail)"),
        Line::from(Span::styled("  * ", Style::default().fg(Color::Yellow))),
        Line::from("    Number of workouts per week"),
        Line::from(""),
        Line::from("Your plan will use:"),
        Line::from(Span::styled(
            "  - Maffetone MAF heart rate for aerobic training",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "  - RPE (Rate of Perceived Exertion) for intervals",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to begin...",
            Style::default().add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Welcome"));
    frame.render_widget(paragraph, area);
}

fn render_age_input(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Min(0),
        ])
        .split(area);

    let explanation = vec![
        Line::from("Enter your age to calculate your MAF (Maximum Aerobic Function) heart rate."),
        Line::from(""),
        Line::from(Span::styled(
            "Maffetone Formula: MAF HR = 180 - age",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("This determines your optimal aerobic training zone."),
    ];

    let explanation_widget = Paragraph::new(explanation)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(explanation_widget, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Your Age")
        .style(Style::default().fg(Color::Yellow));

    let input_text = if app.age_input.is_empty() {
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::raw(&app.age_input)
    };

    let input = Paragraph::new(input_text)
        .alignment(Alignment::Center)
        .block(input_block);

    let input_area = centered_rect(20, 100, chunks[1]);
    frame.render_widget(input, input_area);

    if !app.age_input.is_empty() {
        if let Ok(age) = app.age_input.parse::<u8>() {
            if (10..=100).contains(&age) {
                let maf = 180 - age as u16;
                let preview = vec![
                    Line::from(Span::styled(
                        format!("Your MAF HR: {} bpm", maf),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(format!("Zone 1 (Recovery): {}-{} bpm", maf - 20, maf - 10)),
                    Line::from(format!("Zone 2 (Aerobic): {}-{} bpm", maf - 10, maf)),
                ];
                let preview_widget = Paragraph::new(preview).alignment(Alignment::Center);
                frame.render_widget(preview_widget, chunks[2]);
            }
        }
    }
}

fn render_distance_select(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new("Select your target race distance:")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = Distance::all()
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let style = if i == app.selected_distance_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.selected_distance_index {
                "> "
            } else {
                "  "
            };
            let weeks = d.plan_weeks();
            ListItem::new(format!("{}{} ({} week plan)", prefix, d.name(), weeks)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Race Distance"),
    );
    frame.render_widget(list, chunks[1]);
}

fn render_race_type_select(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new("Select race type:")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = RaceType::all()
        .iter()
        .enumerate()
        .map(|(i, rt)| {
            let style = if i == app.selected_race_type_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == app.selected_race_type_index {
                "> "
            } else {
                "  "
            };
            let description = match rt {
                RaceType::Road => "Flat to rolling terrain, consistent surface",
                RaceType::Trail => "Technical terrain, elevation changes, varied surfaces",
            };
            ListItem::new(format!("{}{}: {}", prefix, rt.name(), description)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Race Type"));
    frame.render_widget(list, chunks[1]);
}

fn render_workouts_select(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("How many workouts per week?")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let workout_display = Paragraph::new(format!("{}", app.selected_workouts_per_week))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Workouts/Week (2-7)"),
        );
    let workout_area = centered_rect(20, 100, chunks[1]);
    frame.render_widget(workout_display, workout_area);

    let recommendation = match app.selected_workouts_per_week {
        2..=3 => "Good for beginners or those with limited time",
        4..=5 => "Balanced approach for most runners",
        6..=7 => "High volume, ensure adequate recovery",
        _ => "",
    };

    let rec_widget = Paragraph::new(recommendation)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(rec_widget, chunks[2]);
}

fn render_plan_view(frame: &mut Frame, app: &App, area: Rect) {
    let Some(plan) = &app.training_plan else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(area);

    // Summary section
    let summary = vec![
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(
                    "{} {}",
                    plan.profile.target_distance.name(),
                    plan.profile.race_type.name()
                ),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("MAF Heart Rate: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} bpm", plan.hr_zones.maf_hr),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled("Plan Duration: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} weeks", plan.weeks.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!(
                "Zone 1: {}-{} bpm | Zone 2: {}-{} bpm",
                plan.hr_zones.maf_hr - 20,
                plan.hr_zones.maf_hr - 10,
                plan.hr_zones.maf_hr - 10,
                plan.hr_zones.maf_hr
            ),
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let summary_widget = Paragraph::new(summary).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Training Plan Summary"),
    );
    frame.render_widget(summary_widget, chunks[0]);

    // Week details
    if let Some(week) = plan.weeks.get(app.selected_week) {
        let phase_color = match week.phase {
            TrainingPhase::Base => Color::Blue,
            TrainingPhase::Build => Color::Yellow,
            TrainingPhase::Peak => Color::Red,
            TrainingPhase::Taper => Color::Green,
        };

        let mut week_content = vec![
            Line::from(vec![
                Span::styled(
                    format!("Week {} of {} - ", week.week_number, plan.weeks.len()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    week.phase.name(),
                    Style::default()
                        .fg(phase_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" | Total: {} min", week.total_volume_minutes),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(""),
        ];

        for (i, workout) in week.workouts.iter().enumerate() {
            if workout.duration_minutes == 0 {
                continue; // Skip rest days in compact view
            }

            let workout_style = Style::default().fg(Color::White);
            week_content.push(Line::from(vec![
                Span::styled(
                    format!("Day {}: ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", workout.workout_type.name()),
                    workout_style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({} min)", workout.duration_minutes),
                    Style::default().fg(Color::Cyan),
                ),
            ]));

            // Wrap description
            let desc_lines = wrap_text(
                &workout.description,
                (area.width as usize).saturating_sub(8),
            );
            for line in desc_lines {
                week_content.push(Line::from(Span::styled(
                    format!("       {}", line),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            week_content.push(Line::from(""));
        }

        // Add navigation hint
        week_content.push(Line::from(Span::styled(
            format!(
                "Use Up/Down to navigate weeks ({}/{})",
                app.selected_week + 1,
                plan.weeks.len()
            ),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));

        let week_widget = Paragraph::new(week_content)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Week Details"));
        frame.render_widget(week_widget, chunks[1]);
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

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
