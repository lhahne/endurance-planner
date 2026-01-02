use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, PlanFocus, Screen, SummaryField};
use crate::models::TrainingPhase;

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
        Screen::PlanView => render_plan_view(frame, app, area),
        Screen::SavePlan => render_save_plan(frame, app, area),
        Screen::LoadPlan => render_load_plan(frame, app, area),
        Screen::EditWorkout => render_edit_workout(frame, app, area),
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.screen {
        Screen::PlanView => match app.plan_focus {
            PlanFocus::Summary => {
                "Tab: weeks | Up/Down: field | Left/Right: value | s: save | l: load | q: quit"
            }
            PlanFocus::Weeks => {
                "Tab: settings | Up/Down: weeks | Left/Right: workouts | e: edit | s: save | l: load | q: quit"
            }
        },
        Screen::SavePlan => "Enter file path | Enter: save | Esc: cancel",
        Screen::LoadPlan => "Enter file path | Enter: load | Esc: cancel",
        Screen::EditWorkout => "Edit description | Enter: save | Esc: cancel",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, area);
}

fn render_plan_view(frame: &mut Frame, app: &App, area: Rect) {
    let Some(plan) = &app.training_plan else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    // Render summary panel (left)
    render_summary_panel(frame, app, plan, chunks[0]);

    // Render weeks panel (right)
    render_weeks_panel(frame, app, plan, chunks[1]);
}

fn render_summary_panel(
    frame: &mut Frame,
    app: &App,
    plan: &crate::models::TrainingPlan,
    area: Rect,
) {
    let is_focused = app.plan_focus == PlanFocus::Summary;

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut lines = vec![];

    // Age field
    let age_selected = is_focused && app.selected_summary_field == SummaryField::Age;
    lines.push(create_field_line(
        "Age:",
        &format!("{}", app.age),
        age_selected,
    ));

    // MAF Adjustment field (moved from below)
    let maf_adj_selected = is_focused && app.selected_summary_field == SummaryField::MafAdjustment;
    lines.push(create_field_line(
        "MAF Adjust:",
        app.selected_maf_adjustment().name(),
        maf_adj_selected,
    ));

    lines.push(Line::from(""));

    // Target Distance field
    let distance_selected = is_focused && app.selected_summary_field == SummaryField::Distance;
    lines.push(create_field_line(
        "Target:",
        plan.profile.target_distance.name(),
        distance_selected,
    ));

    // Race Type field
    let race_type_selected = is_focused && app.selected_summary_field == SummaryField::RaceType;
    lines.push(create_field_line(
        "Type:",
        plan.profile.race_type.name(),
        race_type_selected,
    ));

    // Workouts per Week field
    let workouts_selected =
        is_focused && app.selected_summary_field == SummaryField::WorkoutsPerWeek;
    lines.push(create_field_line(
        "Workouts/week:",
        &format!("{}", app.selected_workouts_per_week),
        workouts_selected,
    ));

    lines.push(Line::from(""));

    // Duration (read-only, derived from distance)
    lines.push(Line::from(vec![
        Span::styled("   Duration: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} weeks", plan.weeks.len()),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    lines.push(Line::from(""));

    // Heart Rate Zones (read-only, derived from age)
    lines.push(Line::from(Span::styled(
        "   Heart Rate Zones",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("   MAF: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} bpm", plan.hr_zones.maf_hr),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   Zone 1: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!(
                "{}-{}",
                plan.hr_zones.maf_hr - 20,
                plan.hr_zones.maf_hr - 10
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   Zone 2: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}-{}", plan.hr_zones.maf_hr - 10, plan.hr_zones.maf_hr),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // Status message if any
    if let Some((msg, is_error)) = &app.status_message {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("   {}", msg),
            Style::default().fg(if *is_error { Color::Red } else { Color::Green }),
        )));
    }

    let title = if is_focused {
        "Settings [editing]"
    } else {
        "Settings"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let summary_widget = Paragraph::new(lines).block(block);
    frame.render_widget(summary_widget, area);
}

/// Helper to create a field line with selection highlighting
fn create_field_line(label: &str, value: &str, is_selected: bool) -> Line<'static> {
    let prefix = if is_selected { " > " } else { "   " };
    let value_style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(Color::Yellow)),
        Span::styled(format!("{} ", label), Style::default().fg(Color::Gray)),
        Span::styled(value.to_string(), value_style),
    ])
}

fn render_weeks_panel(
    frame: &mut Frame,
    app: &App,
    plan: &crate::models::TrainingPlan,
    area: Rect,
) {
    let is_focused = app.plan_focus == PlanFocus::Weeks;

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let Some(week) = plan.weeks.get(app.selected_week) else {
        return;
    };

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

        let is_selected = is_focused && i == app.selected_workout;
        let prefix = if is_selected { "> " } else { "  " };
        let workout_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        week_content.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("Day {}: ", i + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{} ", workout.workout_type.name()), workout_style),
            Span::styled(
                format!("({} min)", workout.duration_minutes),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Wrap description
        let desc_lines = wrap_text(
            &workout.description,
            (area.width as usize).saturating_sub(10),
        );
        let desc_style = if is_selected {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        for line in desc_lines {
            week_content.push(Line::from(Span::styled(
                format!("         {}", line),
                desc_style,
            )));
        }
        week_content.push(Line::from(""));
    }

    let title = if is_focused {
        "Week Details [navigating]"
    } else {
        "Week Details"
    };

    let week_widget = Paragraph::new(week_content)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        );
    frame.render_widget(week_widget, area);
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

fn render_save_plan(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let explanation = vec![
        Line::from(Span::styled(
            "Save Training Plan",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter the file path to save your training plan as a Markdown file."),
        Line::from("The plan can be loaded later and edited."),
    ];

    let explanation_widget = Paragraph::new(explanation)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(explanation_widget, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("File Path")
        .style(Style::default().fg(Color::Yellow));

    let input_text = if app.file_path_input.is_empty() {
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::raw(&app.file_path_input)
    };

    let input = Paragraph::new(input_text)
        .alignment(Alignment::Center)
        .block(input_block);

    let input_area = centered_rect(60, 100, chunks[1]);
    frame.render_widget(input, input_area);

    // Show status message
    if let Some((msg, is_error)) = &app.status_message {
        let status = Paragraph::new(msg.clone())
            .style(Style::default().fg(if *is_error { Color::Red } else { Color::Green }))
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);
    }
}

fn render_load_plan(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let explanation = vec![
        Line::from(Span::styled(
            "Load Training Plan",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter the path to a previously saved training plan Markdown file."),
        Line::from("You can edit the plan after loading."),
    ];

    let explanation_widget = Paragraph::new(explanation)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(explanation_widget, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("File Path")
        .style(Style::default().fg(Color::Yellow));

    let input_text = if app.file_path_input.is_empty() {
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::raw(&app.file_path_input)
    };

    let input = Paragraph::new(input_text)
        .alignment(Alignment::Center)
        .block(input_block);

    let input_area = centered_rect(60, 100, chunks[1]);
    frame.render_widget(input, input_area);

    // Show status message
    if let Some((msg, is_error)) = &app.status_message {
        let status = Paragraph::new(msg.clone())
            .style(Style::default().fg(if *is_error { Color::Red } else { Color::Green }))
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);
    }
}

fn render_edit_workout(frame: &mut Frame, app: &App, area: Rect) {
    let Some(plan) = &app.training_plan else {
        return;
    };

    let Some(week) = plan.weeks.get(app.selected_week) else {
        return;
    };

    let Some(workout) = week.workouts.get(app.selected_workout) else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Header with workout info
    let header = vec![
        Line::from(Span::styled(
            "Edit Workout Description",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Week {} - Day {} - {}",
            week.week_number,
            app.selected_workout + 1,
            workout.workout_type.name()
        )),
    ];

    let header_widget = Paragraph::new(header)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header_widget, chunks[0]);

    // Current description label
    let label = Paragraph::new("Description:").style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(label, chunks[1]);

    // Edit area
    let edit_block = Block::default()
        .borders(Borders::ALL)
        .title("Edit (press Enter to save, Esc to cancel)")
        .style(Style::default().fg(Color::Yellow));

    let edit_text = if app.edit_buffer.is_empty() {
        "_".to_string()
    } else {
        app.edit_buffer.clone()
    };

    let edit_widget = Paragraph::new(edit_text)
        .wrap(Wrap { trim: false })
        .block(edit_block);
    frame.render_widget(edit_widget, chunks[2]);

    // Help text
    let help = Paragraph::new("Type to edit | Enter: save changes | Esc: cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}
