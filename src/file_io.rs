use std::fs;
use std::io;
use std::path::Path;

use crate::models::{
    Distance, HeartRateZones, RaceType, TrainingPhase, TrainingPlan, TrainingWeek, UserProfile,
    Workout, WorkoutType,
};

/// Errors that can occur during file operations
#[derive(Debug)]
pub enum FileError {
    IoError(io::Error),
    ParseError(String),
}

impl From<io::Error> for FileError {
    fn from(err: io::Error) -> Self {
        FileError::IoError(err)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::IoError(e) => write!(f, "IO error: {}", e),
            FileError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

/// Save a training plan to a Markdown file
pub fn save_plan_to_markdown(plan: &TrainingPlan, path: &Path) -> Result<(), FileError> {
    let content = plan_to_markdown(plan);
    fs::write(path, content)?;
    Ok(())
}

/// Load a training plan from a Markdown file
pub fn load_plan_from_markdown(path: &Path) -> Result<TrainingPlan, FileError> {
    let content = fs::read_to_string(path)?;
    parse_markdown_plan(&content)
}

/// Convert a training plan to Markdown format
pub fn plan_to_markdown(plan: &TrainingPlan) -> String {
    let mut md = String::new();

    // Header with metadata
    md.push_str("# Endurance Training Plan\n\n");

    // Metadata section (parseable)
    md.push_str("## Plan Details\n\n");
    md.push_str(&format!("- **Age:** {}\n", plan.profile.age));
    md.push_str(&format!(
        "- **Target Distance:** {}\n",
        plan.profile.target_distance.name()
    ));
    md.push_str(&format!(
        "- **Race Type:** {}\n",
        plan.profile.race_type.name()
    ));
    md.push_str(&format!(
        "- **Workouts per Week:** {}\n",
        plan.profile.workouts_per_week
    ));
    md.push_str(&format!(
        "- **MAF Heart Rate:** {} bpm\n",
        plan.hr_zones.maf_hr
    ));
    md.push_str(&format!(
        "- **Zone 1 (Recovery):** {}-{} bpm\n",
        plan.hr_zones.maf_hr - 20,
        plan.hr_zones.maf_hr - 10
    ));
    md.push_str(&format!(
        "- **Zone 2 (Aerobic):** {}-{} bpm\n\n",
        plan.hr_zones.maf_hr - 10,
        plan.hr_zones.maf_hr
    ));

    // Training weeks
    md.push_str("## Training Schedule\n\n");

    for week in &plan.weeks {
        md.push_str(&format!(
            "### Week {} - {} ({} min total)\n\n",
            week.week_number,
            week.phase.name(),
            week.total_volume_minutes
        ));

        for (i, workout) in week.workouts.iter().enumerate() {
            if workout.duration_minutes == 0 {
                md.push_str(&format!("**Day {}:** Rest\n\n", i + 1));
            } else {
                md.push_str(&format!(
                    "**Day {}:** {} ({} min)\n",
                    i + 1,
                    workout.workout_type.name(),
                    workout.duration_minutes
                ));
                md.push_str(&format!("> {}\n\n", workout.description));
            }
        }

        md.push_str("---\n\n");
    }

    md
}

/// Parse a Markdown file back into a TrainingPlan
fn parse_markdown_plan(content: &str) -> Result<TrainingPlan, FileError> {
    let lines: Vec<&str> = content.lines().collect();

    // Parse metadata
    let age = extract_number(&lines, "Age:")?;
    let distance = extract_distance(&lines)?;
    let race_type = extract_race_type(&lines)?;
    let workouts_per_week = extract_number(&lines, "Workouts per Week:")?;

    let profile = UserProfile {
        age,
        target_distance: distance,
        race_type,
        workouts_per_week,
    };

    let hr_zones = HeartRateZones::from_age(age);

    // Parse weeks
    let weeks = parse_weeks(&lines)?;

    Ok(TrainingPlan {
        profile,
        hr_zones,
        weeks,
    })
}

fn extract_number(lines: &[&str], key: &str) -> Result<u8, FileError> {
    for line in lines {
        if line.contains(key) {
            let parts: Vec<&str> = line.split(key).collect();
            if parts.len() >= 2 {
                let value_part = parts[1].trim().trim_end_matches(" bpm");
                // Remove markdown formatting
                let clean = value_part
                    .trim_start_matches("**")
                    .trim_end_matches("**")
                    .trim();
                if let Ok(num) = clean.parse::<u8>() {
                    return Ok(num);
                }
            }
        }
    }
    Err(FileError::ParseError(format!(
        "Could not find or parse '{}'",
        key
    )))
}

fn extract_distance(lines: &[&str]) -> Result<Distance, FileError> {
    for line in lines {
        if line.contains("Target Distance:") {
            let lower = line.to_lowercase();
            if lower.contains("5k") && !lower.contains("50k") {
                return Ok(Distance::FiveK);
            } else if lower.contains("10k") && !lower.contains("100k") {
                return Ok(Distance::TenK);
            } else if lower.contains("half") {
                return Ok(Distance::HalfMarathon);
            } else if lower.contains("marathon") && !lower.contains("half") {
                return Ok(Distance::Marathon);
            } else if lower.contains("50k") {
                return Ok(Distance::FiftyK);
            } else if lower.contains("100k") {
                return Ok(Distance::HundredK);
            } else if lower.contains("100 mile") {
                return Ok(Distance::HundredMiles);
            }
        }
    }
    Err(FileError::ParseError(
        "Could not parse target distance".to_string(),
    ))
}

fn extract_race_type(lines: &[&str]) -> Result<RaceType, FileError> {
    for line in lines {
        if line.contains("Race Type:") {
            let lower = line.to_lowercase();
            if lower.contains("trail") {
                return Ok(RaceType::Trail);
            } else if lower.contains("road") {
                return Ok(RaceType::Road);
            }
        }
    }
    Err(FileError::ParseError(
        "Could not parse race type".to_string(),
    ))
}

fn parse_weeks(lines: &[&str]) -> Result<Vec<TrainingWeek>, FileError> {
    let mut weeks = Vec::new();
    let mut current_week: Option<TrainingWeek> = None;
    let mut current_workouts: Vec<Workout> = Vec::new();
    let mut pending_workout: Option<(String, u16)> = None;

    for line in lines {
        // Check for week header: ### Week N - Phase (X min total)
        if line.starts_with("### Week") {
            // Save previous week if exists
            if let Some(mut week) = current_week.take() {
                week.workouts = std::mem::take(&mut current_workouts);
                weeks.push(week);
            }

            // Parse new week
            if let Some(week) = parse_week_header(line) {
                current_week = Some(week);
            }
        }
        // Check for workout: **Day N:** Type (X min)
        else if line.starts_with("**Day") {
            // Save pending workout description
            if let Some((name, duration)) = pending_workout.take() {
                current_workouts.push(Workout {
                    workout_type: parse_workout_type(&name),
                    duration_minutes: duration,
                    description: String::new(),
                });
            }

            if let Some((name, duration)) = parse_day_line(line) {
                pending_workout = Some((name, duration));
            }
        }
        // Check for description: > description
        else if line.starts_with('>') {
            if let Some((name, duration)) = pending_workout.take() {
                let description = line.trim_start_matches('>').trim().to_string();
                current_workouts.push(Workout {
                    workout_type: parse_workout_type(&name),
                    duration_minutes: duration,
                    description,
                });
            }
        }
    }

    // Handle last pending workout
    if let Some((name, duration)) = pending_workout.take() {
        current_workouts.push(Workout {
            workout_type: parse_workout_type(&name),
            duration_minutes: duration,
            description: String::new(),
        });
    }

    // Save last week
    if let Some(mut week) = current_week.take() {
        week.workouts = current_workouts;
        weeks.push(week);
    }

    if weeks.is_empty() {
        return Err(FileError::ParseError("No weeks found in file".to_string()));
    }

    Ok(weeks)
}

fn parse_week_header(line: &str) -> Option<TrainingWeek> {
    // ### Week 1 - Base Building (180 min total)
    let line = line.trim_start_matches('#').trim();

    let week_num = line
        .split_whitespace()
        .nth(1)?
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .ok()?;

    let phase = if line.contains("Base") {
        TrainingPhase::Base
    } else if line.contains("Build") {
        TrainingPhase::Build
    } else if line.contains("Peak") {
        TrainingPhase::Peak
    } else if line.contains("Taper") {
        TrainingPhase::Taper
    } else {
        TrainingPhase::Base
    };

    // Extract total minutes
    let total_volume = line
        .split('(')
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .unwrap_or(0);

    Some(TrainingWeek {
        week_number: week_num,
        phase,
        workouts: Vec::new(),
        total_volume_minutes: total_volume,
    })
}

fn parse_day_line(line: &str) -> Option<(String, u16)> {
    // **Day 1:** Easy Run (45 min)
    // or **Day 7:** Rest
    let after_day = line.split(":**").nth(1)?.trim();

    if after_day == "Rest" || after_day.starts_with("Rest") {
        return Some(("Rest".to_string(), 0));
    }

    // Find the workout name and duration
    if let Some(paren_start) = after_day.find('(') {
        let name = after_day[..paren_start].trim().to_string();
        let duration_str = after_day[paren_start + 1..]
            .trim_end_matches(')')
            .split_whitespace()
            .next()?;
        let duration = duration_str.parse().unwrap_or(0);
        Some((name, duration))
    } else {
        Some((after_day.to_string(), 0))
    }
}

fn parse_workout_type(name: &str) -> WorkoutType {
    match name.to_lowercase().as_str() {
        "easy run" => WorkoutType::EasyRun,
        "long run" => WorkoutType::LongRun,
        "recovery run" => WorkoutType::RecoveryRun,
        "intervals" => WorkoutType::Intervals {
            reps: 4,
            work_minutes: 3,
            rest_minutes: 2,
            target_rpe: crate::models::RPE::Seven,
        },
        "tempo run" => WorkoutType::TempoRun {
            duration_minutes: 20,
            target_rpe: crate::models::RPE::Six,
        },
        "hill repeats" => WorkoutType::HillRepeats {
            reps: 6,
            target_rpe: crate::models::RPE::Eight,
        },
        "technical trail" => WorkoutType::TechnicalTrail,
        "vertical training" => WorkoutType::VerticalTraining {
            elevation_gain_meters: 500,
        },
        _ => WorkoutType::Rest,
    }
}

/// Update a specific workout in the plan
#[allow(dead_code)]
pub fn update_workout(
    plan: &mut TrainingPlan,
    week_index: usize,
    workout_index: usize,
    new_description: String,
) {
    if let Some(week) = plan.weeks.get_mut(week_index) {
        if let Some(workout) = week.workouts.get_mut(workout_index) {
            workout.description = new_description;
        }
    }
}

/// Update workout duration
#[allow(dead_code)]
pub fn update_workout_duration(
    plan: &mut TrainingPlan,
    week_index: usize,
    workout_index: usize,
    new_duration: u16,
) {
    if let Some(week) = plan.weeks.get_mut(week_index) {
        if let Some(workout) = week.workouts.get_mut(workout_index) {
            let old_duration = workout.duration_minutes;
            workout.duration_minutes = new_duration;
            // Update total volume
            week.total_volume_minutes = week.total_volume_minutes - old_duration + new_duration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_roundtrip() {
        let profile = UserProfile {
            age: 35,
            target_distance: Distance::Marathon,
            race_type: RaceType::Road,
            workouts_per_week: 5,
        };

        let hr_zones = HeartRateZones::from_age(35);

        let plan = TrainingPlan {
            profile,
            hr_zones,
            weeks: vec![TrainingWeek {
                week_number: 1,
                phase: TrainingPhase::Base,
                workouts: vec![
                    Workout {
                        workout_type: WorkoutType::LongRun,
                        duration_minutes: 90,
                        description: "Long run at MAF HR".to_string(),
                    },
                    Workout {
                        workout_type: WorkoutType::EasyRun,
                        duration_minutes: 45,
                        description: "Easy aerobic run".to_string(),
                    },
                ],
                total_volume_minutes: 135,
            }],
        };

        let md = plan_to_markdown(&plan);
        let parsed = parse_markdown_plan(&md).unwrap();

        assert_eq!(parsed.profile.age, 35);
        assert_eq!(parsed.profile.target_distance, Distance::Marathon);
        assert_eq!(parsed.profile.race_type, RaceType::Road);
        assert_eq!(parsed.weeks.len(), 1);
    }
}
