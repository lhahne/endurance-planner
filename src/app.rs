use std::path::PathBuf;

use crate::file_io::{load_plan_from_markdown, save_plan_to_markdown};
use crate::models::{
    Distance, HeartRateZones, RaceType, TrainingPhase, TrainingPlan, TrainingWeek, UserProfile,
    Workout, WorkoutType, RPE,
};

/// Current screen in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Welcome,
    AgeInput,
    DistanceSelect,
    RaceTypeSelect,
    WorkoutsPerWeekSelect,
    PlanView,
    SavePlan,
    LoadPlan,
    EditWorkout,
}

/// Application state
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    // User input state
    pub age_input: String,
    pub selected_distance_index: usize,
    pub selected_race_type_index: usize,
    pub selected_workouts_per_week: u8,

    // Generated plan
    pub training_plan: Option<TrainingPlan>,
    pub selected_week: usize,
    pub selected_workout: usize,

    // File operations
    pub file_path_input: String,
    pub status_message: Option<(String, bool)>, // (message, is_error)

    // Edit mode
    pub edit_buffer: String,
    pub edit_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::Welcome,
            should_quit: false,
            age_input: String::new(),
            selected_distance_index: 0,
            selected_race_type_index: 0,
            selected_workouts_per_week: 4,
            training_plan: None,
            selected_week: 0,
            selected_workout: 0,
            file_path_input: String::new(),
            status_message: None,
            edit_buffer: String::new(),
            edit_cursor: 0,
        }
    }

    pub fn next_screen(&mut self) {
        self.status_message = None;
        self.screen = match self.screen {
            Screen::Welcome => Screen::AgeInput,
            Screen::AgeInput => {
                if self.validate_age() {
                    Screen::DistanceSelect
                } else {
                    Screen::AgeInput
                }
            }
            Screen::DistanceSelect => Screen::RaceTypeSelect,
            Screen::RaceTypeSelect => Screen::WorkoutsPerWeekSelect,
            Screen::WorkoutsPerWeekSelect => {
                self.generate_plan();
                Screen::PlanView
            }
            Screen::PlanView => Screen::PlanView,
            Screen::SavePlan => Screen::SavePlan,
            Screen::LoadPlan => Screen::LoadPlan,
            Screen::EditWorkout => Screen::EditWorkout,
        };
    }

    pub fn previous_screen(&mut self) {
        self.status_message = None;
        self.screen = match self.screen {
            Screen::Welcome => Screen::Welcome,
            Screen::AgeInput => Screen::Welcome,
            Screen::DistanceSelect => Screen::AgeInput,
            Screen::RaceTypeSelect => Screen::DistanceSelect,
            Screen::WorkoutsPerWeekSelect => Screen::RaceTypeSelect,
            Screen::PlanView => Screen::WorkoutsPerWeekSelect,
            Screen::SavePlan => Screen::PlanView,
            Screen::LoadPlan => Screen::Welcome,
            Screen::EditWorkout => Screen::PlanView,
        };
    }

    pub fn go_to_save(&mut self) {
        if self.training_plan.is_some() {
            self.file_path_input = "training_plan.md".to_string();
            self.status_message = None;
            self.screen = Screen::SavePlan;
        }
    }

    pub fn go_to_load(&mut self) {
        self.file_path_input.clear();
        self.status_message = None;
        self.screen = Screen::LoadPlan;
    }

    pub fn go_to_edit(&mut self) {
        if let Some(plan) = &self.training_plan {
            if let Some(week) = plan.weeks.get(self.selected_week) {
                if let Some(workout) = week.workouts.get(self.selected_workout) {
                    self.edit_buffer = workout.description.clone();
                    self.edit_cursor = self.edit_buffer.len();
                    self.screen = Screen::EditWorkout;
                }
            }
        }
    }

    pub fn save_plan(&mut self) {
        if self.file_path_input.is_empty() {
            self.status_message = Some(("Please enter a file path".to_string(), true));
            return;
        }

        if let Some(plan) = &self.training_plan {
            let path = PathBuf::from(&self.file_path_input);
            match save_plan_to_markdown(plan, &path) {
                Ok(()) => {
                    self.status_message =
                        Some((format!("Plan saved to {}", self.file_path_input), false));
                }
                Err(e) => {
                    self.status_message = Some((format!("Error saving: {}", e), true));
                }
            }
        }
    }

    pub fn load_plan(&mut self) {
        if self.file_path_input.is_empty() {
            self.status_message = Some(("Please enter a file path".to_string(), true));
            return;
        }

        let path = PathBuf::from(&self.file_path_input);
        match load_plan_from_markdown(&path) {
            Ok(plan) => {
                self.training_plan = Some(plan);
                self.selected_week = 0;
                self.selected_workout = 0;
                self.status_message = Some(("Plan loaded successfully".to_string(), false));
                self.screen = Screen::PlanView;
            }
            Err(e) => {
                self.status_message = Some((format!("Error loading: {}", e), true));
            }
        }
    }

    pub fn apply_edit(&mut self) {
        if let Some(plan) = &mut self.training_plan {
            if let Some(week) = plan.weeks.get_mut(self.selected_week) {
                if let Some(workout) = week.workouts.get_mut(self.selected_workout) {
                    workout.description = self.edit_buffer.clone();
                }
            }
        }
        self.screen = Screen::PlanView;
    }

    pub fn select_next_workout(&mut self) {
        if let Some(plan) = &self.training_plan {
            if let Some(week) = plan.weeks.get(self.selected_week) {
                let active_workouts: Vec<usize> = week
                    .workouts
                    .iter()
                    .enumerate()
                    .filter(|(_, w)| w.duration_minutes > 0)
                    .map(|(i, _)| i)
                    .collect();

                if let Some(pos) = active_workouts
                    .iter()
                    .position(|&i| i == self.selected_workout)
                {
                    if pos + 1 < active_workouts.len() {
                        self.selected_workout = active_workouts[pos + 1];
                    }
                } else if !active_workouts.is_empty() {
                    self.selected_workout = active_workouts[0];
                }
            }
        }
    }

    pub fn select_prev_workout(&mut self) {
        if let Some(plan) = &self.training_plan {
            if let Some(week) = plan.weeks.get(self.selected_week) {
                let active_workouts: Vec<usize> = week
                    .workouts
                    .iter()
                    .enumerate()
                    .filter(|(_, w)| w.duration_minutes > 0)
                    .map(|(i, _)| i)
                    .collect();

                if let Some(pos) = active_workouts
                    .iter()
                    .position(|&i| i == self.selected_workout)
                {
                    if pos > 0 {
                        self.selected_workout = active_workouts[pos - 1];
                    }
                } else if !active_workouts.is_empty() {
                    self.selected_workout = active_workouts[0];
                }
            }
        }
    }

    fn validate_age(&self) -> bool {
        self.age_input
            .parse::<u8>()
            .map(|age| (10..=100).contains(&age))
            .unwrap_or(false)
    }

    pub fn get_age(&self) -> u8 {
        self.age_input.parse().unwrap_or(30)
    }

    pub fn selected_distance(&self) -> Distance {
        Distance::all()[self.selected_distance_index]
    }

    pub fn selected_race_type(&self) -> RaceType {
        RaceType::all()[self.selected_race_type_index]
    }

    pub fn select_next_distance(&mut self) {
        let len = Distance::all().len();
        self.selected_distance_index = (self.selected_distance_index + 1) % len;
    }

    pub fn select_prev_distance(&mut self) {
        let len = Distance::all().len();
        self.selected_distance_index = (self.selected_distance_index + len - 1) % len;
    }

    pub fn select_next_race_type(&mut self) {
        let len = RaceType::all().len();
        self.selected_race_type_index = (self.selected_race_type_index + 1) % len;
    }

    pub fn select_prev_race_type(&mut self) {
        let len = RaceType::all().len();
        self.selected_race_type_index = (self.selected_race_type_index + len - 1) % len;
    }

    pub fn increase_workouts(&mut self) {
        if self.selected_workouts_per_week < 7 {
            self.selected_workouts_per_week += 1;
        }
    }

    pub fn decrease_workouts(&mut self) {
        if self.selected_workouts_per_week > 2 {
            self.selected_workouts_per_week -= 1;
        }
    }

    pub fn scroll_plan_up(&mut self) {
        if self.selected_week > 0 {
            self.selected_week -= 1;
            self.selected_workout = 0;
        }
    }

    pub fn scroll_plan_down(&mut self) {
        if let Some(plan) = &self.training_plan {
            if self.selected_week < plan.weeks.len().saturating_sub(1) {
                self.selected_week += 1;
                self.selected_workout = 0;
            }
        }
    }

    fn generate_plan(&mut self) {
        let profile = UserProfile {
            age: self.get_age(),
            target_distance: self.selected_distance(),
            race_type: self.selected_race_type(),
            workouts_per_week: self.selected_workouts_per_week,
        };

        let hr_zones = HeartRateZones::from_age(profile.age);
        let total_weeks = profile.target_distance.plan_weeks();
        let weeks = generate_training_weeks(&profile, &hr_zones, total_weeks);

        self.training_plan = Some(TrainingPlan {
            profile,
            hr_zones,
            weeks,
        });
        self.selected_week = 0;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate training weeks based on user profile
fn generate_training_weeks(
    profile: &UserProfile,
    hr_zones: &HeartRateZones,
    total_weeks: u8,
) -> Vec<TrainingWeek> {
    let mut weeks = Vec::new();

    // Calculate phase boundaries
    let base_weeks = (total_weeks as f32 * 0.4).ceil() as u8;
    let build_weeks = (total_weeks as f32 * 0.3).ceil() as u8;
    let peak_weeks = (total_weeks as f32 * 0.2).ceil() as u8;
    let _taper_weeks = total_weeks - base_weeks - build_weeks - peak_weeks;

    for week_num in 1..=total_weeks {
        let phase = if week_num <= base_weeks {
            TrainingPhase::Base
        } else if week_num <= base_weeks + build_weeks {
            TrainingPhase::Build
        } else if week_num <= base_weeks + build_weeks + peak_weeks {
            TrainingPhase::Peak
        } else {
            TrainingPhase::Taper
        };

        let workouts = generate_week_workouts(profile, hr_zones, phase, week_num, total_weeks);
        let total_volume: u16 = workouts.iter().map(|w| w.duration_minutes).sum();

        weeks.push(TrainingWeek {
            week_number: week_num,
            phase,
            workouts,
            total_volume_minutes: total_volume,
        });
    }

    weeks
}

/// Generate workouts for a single week
fn generate_week_workouts(
    profile: &UserProfile,
    hr_zones: &HeartRateZones,
    phase: TrainingPhase,
    week_num: u8,
    total_weeks: u8,
) -> Vec<Workout> {
    let mut workouts = Vec::new();
    let workouts_per_week = profile.workouts_per_week;
    let is_trail = profile.race_type == RaceType::Trail;

    // Calculate base durations that progress through the plan
    let progression = week_num as f32 / total_weeks as f32;
    let base_easy_duration = match phase {
        TrainingPhase::Base => 30 + (progression * 15.0) as u16,
        TrainingPhase::Build => 40 + (progression * 10.0) as u16,
        TrainingPhase::Peak => 45,
        TrainingPhase::Taper => 30,
    };

    let base_long_duration = match phase {
        TrainingPhase::Base => 60 + (progression * 30.0) as u16,
        TrainingPhase::Build => 75 + (progression * 45.0) as u16,
        TrainingPhase::Peak => 90 + (profile.target_distance.kilometers() * 1.5) as u16,
        TrainingPhase::Taper => 60,
    };

    // Day 1: Always a long run (except in taper - make it moderate)
    workouts.push(create_long_run(
        hr_zones,
        base_long_duration,
        phase,
        is_trail,
    ));

    // Remaining workouts based on phase and workouts_per_week
    let remaining = workouts_per_week - 1;

    match phase {
        TrainingPhase::Base => {
            // Base phase: mostly easy aerobic runs
            for i in 0..remaining {
                if i == 0 && is_trail {
                    workouts.push(create_technical_trail(hr_zones, base_easy_duration));
                } else {
                    workouts.push(create_easy_run(hr_zones, base_easy_duration));
                }
            }
        }
        TrainingPhase::Build => {
            // Build phase: introduce intervals and tempo
            if remaining >= 1 {
                workouts.push(create_intervals(phase));
            }
            if remaining >= 2 {
                workouts.push(create_tempo_run(base_easy_duration));
            }
            if remaining >= 3 && is_trail {
                workouts.push(create_hill_repeats());
            } else if remaining >= 3 {
                workouts.push(create_easy_run(hr_zones, base_easy_duration));
            }
            for _ in 4..=remaining {
                workouts.push(create_easy_run(hr_zones, base_easy_duration - 10));
            }
        }
        TrainingPhase::Peak => {
            // Peak phase: quality workouts with recovery
            if remaining >= 1 {
                workouts.push(create_intervals(phase));
            }
            if remaining >= 2 {
                workouts.push(create_tempo_run(base_easy_duration + 10));
            }
            if remaining >= 3 && is_trail {
                workouts.push(create_vertical_training(profile.target_distance));
            } else if remaining >= 3 {
                workouts.push(create_hill_repeats());
            }
            for _ in 4..=remaining {
                workouts.push(create_recovery_run(hr_zones));
            }
        }
        TrainingPhase::Taper => {
            // Taper phase: reduced volume, maintain intensity
            if remaining >= 1 {
                workouts.push(create_easy_run(hr_zones, 25));
            }
            if remaining >= 2 {
                workouts.push(create_intervals(phase));
            }
            for _ in 3..=remaining {
                workouts.push(create_recovery_run(hr_zones));
            }
        }
    }

    // Fill rest days
    while workouts.len() < 7 {
        workouts.push(Workout {
            workout_type: WorkoutType::Rest,
            duration_minutes: 0,
            description: "Complete rest or light stretching".to_string(),
        });
    }

    workouts
}

fn create_easy_run(hr_zones: &HeartRateZones, duration: u16) -> Workout {
    let (min_hr, max_hr) = hr_zones.zone2_range();
    Workout {
        workout_type: WorkoutType::EasyRun,
        duration_minutes: duration,
        description: format!(
            "Easy aerobic run at MAF HR ({}-{} bpm). Conversational pace.",
            min_hr, max_hr
        ),
    }
}

fn create_long_run(
    hr_zones: &HeartRateZones,
    duration: u16,
    phase: TrainingPhase,
    is_trail: bool,
) -> Workout {
    let (min_hr, max_hr) = hr_zones.zone2_range();
    let terrain_note = if is_trail {
        " Include varied terrain."
    } else {
        ""
    };
    let phase_note = match phase {
        TrainingPhase::Taper => " Keep effort easy, focus on staying fresh.",
        _ => "",
    };

    Workout {
        workout_type: WorkoutType::LongRun,
        duration_minutes: duration,
        description: format!(
            "Long run at MAF HR ({}-{} bpm).{}{} Build endurance gradually.",
            min_hr, max_hr, terrain_note, phase_note
        ),
    }
}

fn create_recovery_run(hr_zones: &HeartRateZones) -> Workout {
    let (min_hr, max_hr) = hr_zones.zone1_range();
    Workout {
        workout_type: WorkoutType::RecoveryRun,
        duration_minutes: 20,
        description: format!(
            "Very easy recovery run at {}-{} bpm. Should feel effortless.",
            min_hr, max_hr
        ),
    }
}

fn create_intervals(phase: TrainingPhase) -> Workout {
    let (reps, work_mins, rest_mins, rpe) = match phase {
        TrainingPhase::Build => (4, 3, 2, RPE::Seven),
        TrainingPhase::Peak => (5, 4, 2, RPE::Eight),
        TrainingPhase::Taper => (3, 2, 2, RPE::Seven),
        _ => (3, 2, 2, RPE::Six),
    };

    let total_duration = (reps as u16 * (work_mins as u16 + rest_mins as u16)) + 20; // +20 for warm-up/cool-down

    Workout {
        workout_type: WorkoutType::Intervals {
            reps,
            work_minutes: work_mins,
            rest_minutes: rest_mins,
            target_rpe: rpe,
        },
        duration_minutes: total_duration,
        description: format!(
            "{}x{} min intervals at RPE {} ({}). {} min recovery jog between. \
            Include 10 min warm-up and cool-down.",
            reps,
            work_mins,
            rpe.value(),
            rpe.description(),
            rest_mins
        ),
    }
}

fn create_tempo_run(base_duration: u16) -> Workout {
    let tempo_duration = (base_duration as f32 * 0.6) as u16;
    Workout {
        workout_type: WorkoutType::TempoRun {
            duration_minutes: tempo_duration,
            target_rpe: RPE::Six,
        },
        duration_minutes: base_duration,
        description: format!(
            "{} min tempo at RPE 6 ({}). Comfortably hard, sustainable effort. \
            Warm up and cool down easy.",
            tempo_duration,
            RPE::Six.description()
        ),
    }
}

fn create_hill_repeats() -> Workout {
    Workout {
        workout_type: WorkoutType::HillRepeats {
            reps: 6,
            target_rpe: RPE::Eight,
        },
        duration_minutes: 40,
        description: format!(
            "6x 60-90 sec hill repeats at RPE 8 ({}). \
            Jog down for recovery. Focus on form and power.",
            RPE::Eight.description()
        ),
    }
}

fn create_technical_trail(hr_zones: &HeartRateZones, duration: u16) -> Workout {
    let (min_hr, max_hr) = hr_zones.zone2_range();
    Workout {
        workout_type: WorkoutType::TechnicalTrail,
        duration_minutes: duration,
        description: format!(
            "Technical trail run at MAF HR ({}-{} bpm). \
            Focus on footwork, agility, and terrain reading.",
            min_hr, max_hr
        ),
    }
}

fn create_vertical_training(distance: Distance) -> Workout {
    let elevation = match distance {
        Distance::FiftyK => 500,
        Distance::HundredK => 800,
        Distance::HundredMiles => 1000,
        _ => 300,
    };

    Workout {
        workout_type: WorkoutType::VerticalTraining {
            elevation_gain_meters: elevation,
        },
        duration_minutes: 60,
        description: format!(
            "Vertical training targeting {}m elevation gain. \
            Power hike uphills at RPE 7, easy jog or walk descents.",
            elevation
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.screen, Screen::Welcome);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_screen_navigation() {
        let mut app = App::new();
        app.age_input = "30".to_string();

        app.next_screen();
        assert_eq!(app.screen, Screen::AgeInput);

        app.next_screen();
        assert_eq!(app.screen, Screen::DistanceSelect);
    }

    #[test]
    fn test_plan_generation() {
        let mut app = App::new();
        app.age_input = "35".to_string();
        app.selected_distance_index = 3; // Marathon
        app.selected_race_type_index = 0; // Road
        app.selected_workouts_per_week = 5;

        app.screen = Screen::WorkoutsPerWeekSelect;
        app.next_screen();

        assert!(app.training_plan.is_some());
        let plan = app.training_plan.as_ref().unwrap();
        assert_eq!(plan.weeks.len(), 16); // Marathon = 16 weeks
        assert_eq!(plan.hr_zones.maf_hr, 145); // 180 - 35
    }
}
