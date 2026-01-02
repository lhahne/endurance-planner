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

    // Markdown conversion tests
    #[test]
    fn test_plan_to_markdown() {
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
            weeks: vec![],
        };

        let md = plan_to_markdown(&plan);
        assert!(md.contains("# Endurance Training Plan"));
        assert!(md.contains("Age:** 35"));
        assert!(md.contains("Target Distance:** Marathon"));
        assert!(md.contains("Race Type:** Road"));
        assert!(md.contains("Workouts per Week:** 5"));
    }

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

    // Distance parsing tests
    #[test]
    fn test_extract_distance_5k() {
        let lines = vec!["- **Target Distance:** 5K"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::FiveK);
    }

    #[test]
    fn test_extract_distance_10k() {
        let lines = vec!["- **Target Distance:** 10K"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::TenK);
    }

    #[test]
    fn test_extract_distance_half_marathon() {
        let lines = vec!["- **Target Distance:** Half Marathon (21.1K)"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::HalfMarathon);
    }

    #[test]
    fn test_extract_distance_marathon() {
        let lines = vec!["- **Target Distance:** Marathon (42.2K)"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::Marathon);
    }

    #[test]
    fn test_extract_distance_50k() {
        let lines = vec!["- **Target Distance:** 50K Ultra"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::FiftyK);
    }

    #[test]
    fn test_extract_distance_100k() {
        let lines = vec!["- **Target Distance:** 100K Ultra"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::HundredK);
    }

    #[test]
    fn test_extract_distance_100_miles() {
        let lines = vec!["- **Target Distance:** 100 Miles Ultra"];
        let distance = extract_distance(&lines).unwrap();
        assert_eq!(distance, Distance::HundredMiles);
    }

    #[test]
    fn test_extract_distance_missing() {
        let lines = vec!["- **Age:** 30"];
        let result = extract_distance(&lines);
        assert!(result.is_err());
    }

    // Race type parsing tests
    #[test]
    fn test_extract_race_type_road() {
        let lines = vec!["- **Race Type:** Road"];
        let race_type = extract_race_type(&lines).unwrap();
        assert_eq!(race_type, RaceType::Road);
    }

    #[test]
    fn test_extract_race_type_trail() {
        let lines = vec!["- **Race Type:** Trail"];
        let race_type = extract_race_type(&lines).unwrap();
        assert_eq!(race_type, RaceType::Trail);
    }

    #[test]
    fn test_extract_race_type_missing() {
        let lines = vec!["- **Age:** 30"];
        let result = extract_race_type(&lines);
        assert!(result.is_err());
    }

    // Number extraction tests
    #[test]
    fn test_extract_number_age() {
        let lines = vec!["- **Age:** 35"];
        let age = extract_number(&lines, "Age:").unwrap();
        assert_eq!(age, 35);
    }

    #[test]
    fn test_extract_number_workouts() {
        let lines = vec!["- **Workouts per Week:** 5"];
        let workouts = extract_number(&lines, "Workouts per Week:").unwrap();
        assert_eq!(workouts, 5);
    }

    #[test]
    fn test_extract_number_with_bpm() {
        let lines = vec!["- **MAF Heart Rate:** 145 bpm"];
        let hr = extract_number(&lines, "MAF Heart Rate:").unwrap();
        assert_eq!(hr, 145);
    }

    #[test]
    fn test_extract_number_missing() {
        let lines = vec!["- **Something:** else"];
        let result = extract_number(&lines, "Age:");
        assert!(result.is_err());
    }

    // Week header parsing tests
    #[test]
    fn test_parse_week_header() {
        let line = "### Week 1 - Base Building (180 min total)";
        let week = parse_week_header(line).unwrap();
        assert_eq!(week.week_number, 1);
        assert_eq!(week.phase, TrainingPhase::Base);
        assert_eq!(week.total_volume_minutes, 180);
    }

    #[test]
    fn test_parse_week_header_build() {
        let line = "### Week 5 - Build Phase (240 min total)";
        let week = parse_week_header(line).unwrap();
        assert_eq!(week.week_number, 5);
        assert_eq!(week.phase, TrainingPhase::Build);
        assert_eq!(week.total_volume_minutes, 240);
    }

    #[test]
    fn test_parse_week_header_peak() {
        let line = "### Week 10 - Peak Training (300 min total)";
        let week = parse_week_header(line).unwrap();
        assert_eq!(week.week_number, 10);
        assert_eq!(week.phase, TrainingPhase::Peak);
    }

    #[test]
    fn test_parse_week_header_taper() {
        let line = "### Week 16 - Taper (120 min total)";
        let week = parse_week_header(line).unwrap();
        assert_eq!(week.week_number, 16);
        assert_eq!(week.phase, TrainingPhase::Taper);
    }

    // Day line parsing tests
    #[test]
    fn test_parse_day_line_with_duration() {
        let line = "**Day 1:** Easy Run (45 min)";
        let (name, duration) = parse_day_line(line).unwrap();
        assert_eq!(name, "Easy Run");
        assert_eq!(duration, 45);
    }

    #[test]
    fn test_parse_day_line_rest() {
        let line = "**Day 7:** Rest";
        let (name, duration) = parse_day_line(line).unwrap();
        assert_eq!(name, "Rest");
        assert_eq!(duration, 0);
    }

    #[test]
    fn test_parse_day_line_long_run() {
        let line = "**Day 1:** Long Run (90 min)";
        let (name, duration) = parse_day_line(line).unwrap();
        assert_eq!(name, "Long Run");
        assert_eq!(duration, 90);
    }

    // Workout type parsing tests
    #[test]
    fn test_parse_workout_type_easy_run() {
        let wt = parse_workout_type("Easy Run");
        assert_eq!(wt, WorkoutType::EasyRun);
    }

    #[test]
    fn test_parse_workout_type_long_run() {
        let wt = parse_workout_type("Long Run");
        assert_eq!(wt, WorkoutType::LongRun);
    }

    #[test]
    fn test_parse_workout_type_recovery_run() {
        let wt = parse_workout_type("Recovery Run");
        assert_eq!(wt, WorkoutType::RecoveryRun);
    }

    #[test]
    fn test_parse_workout_type_intervals() {
        let wt = parse_workout_type("Intervals");
        match wt {
            WorkoutType::Intervals { .. } => {},
            _ => panic!("Expected Intervals"),
        }
    }

    #[test]
    fn test_parse_workout_type_tempo() {
        let wt = parse_workout_type("Tempo Run");
        match wt {
            WorkoutType::TempoRun { .. } => {},
            _ => panic!("Expected TempoRun"),
        }
    }

    #[test]
    fn test_parse_workout_type_hill_repeats() {
        let wt = parse_workout_type("Hill Repeats");
        match wt {
            WorkoutType::HillRepeats { .. } => {},
            _ => panic!("Expected HillRepeats"),
        }
    }

    #[test]
    fn test_parse_workout_type_technical_trail() {
        let wt = parse_workout_type("Technical Trail");
        assert_eq!(wt, WorkoutType::TechnicalTrail);
    }

    #[test]
    fn test_parse_workout_type_vertical() {
        let wt = parse_workout_type("Vertical Training");
        match wt {
            WorkoutType::VerticalTraining { .. } => {},
            _ => panic!("Expected VerticalTraining"),
        }
    }

    #[test]
    fn test_parse_workout_type_unknown() {
        let wt = parse_workout_type("Unknown Workout");
        assert_eq!(wt, WorkoutType::Rest);
    }

    // Update workout tests
    #[test]
    fn test_update_workout() {
        let mut plan = create_test_plan();
        update_workout(&mut plan, 0, 0, "Updated description".to_string());
        assert_eq!(plan.weeks[0].workouts[0].description, "Updated description");
    }

    #[test]
    fn test_update_workout_duration() {
        let mut plan = create_test_plan();
        let old_volume = plan.weeks[0].total_volume_minutes;
        update_workout_duration(&mut plan, 0, 0, 100);
        assert_eq!(plan.weeks[0].workouts[0].duration_minutes, 100);
        // Volume should be updated
        assert_ne!(plan.weeks[0].total_volume_minutes, old_volume);
    }

    #[test]
    fn test_update_workout_invalid_index() {
        let mut plan = create_test_plan();
        update_workout(&mut plan, 10, 0, "Won't update".to_string());
        // Should not panic, just do nothing
    }

    // Error handling tests
    #[test]
    fn test_file_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let file_err = FileError::from(io_err);
        let display = format!("{}", file_err);
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_file_error_display_parse() {
        let file_err = FileError::ParseError("invalid format".to_string());
        let display = format!("{}", file_err);
        assert!(display.contains("Parse error"));
        assert!(display.contains("invalid format"));
    }

    // Helper function
    fn create_test_plan() -> TrainingPlan {
        let profile = UserProfile {
            age: 30,
            target_distance: Distance::Marathon,
            race_type: RaceType::Road,
            workouts_per_week: 5,
        };
        let hr_zones = HeartRateZones::from_age(30);
        TrainingPlan {
            profile,
            hr_zones,
            weeks: vec![TrainingWeek {
                week_number: 1,
                phase: TrainingPhase::Base,
                workouts: vec![
                    Workout {
                        workout_type: WorkoutType::EasyRun,
                        duration_minutes: 45,
                        description: "Test workout".to_string(),
                    },
                ],
                total_volume_minutes: 45,
            }],
        }
    }
}
