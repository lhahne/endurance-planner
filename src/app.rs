use std::path::PathBuf;

use crate::file_io::{load_plan_from_markdown, save_plan_to_markdown};
use crate::models::{
    Distance, HeartRateZones, MafAdjustment, RaceType, TrainingPhase, TrainingPlan, TrainingWeek,
    UserProfile, Workout, WorkoutType, RPE,
};

/// Current screen in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    PlanView,
    SavePlan,
    LoadPlan,
    EditWorkout,
}

/// Which panel has focus in PlanView
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanFocus {
    Summary, // Left panel - editing summary fields
    Weeks,   // Right panel - navigating weeks/workouts
}

/// Summary fields that can be edited in the left panel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SummaryField {
    Age,
    Distance,
    RaceType,
    WorkoutsPerWeek,
    MafAdjustment,
}

impl SummaryField {
    pub fn next(self) -> SummaryField {
        match self {
            SummaryField::Age => SummaryField::MafAdjustment,
            SummaryField::MafAdjustment => SummaryField::Distance,
            SummaryField::Distance => SummaryField::RaceType,
            SummaryField::RaceType => SummaryField::WorkoutsPerWeek,
            SummaryField::WorkoutsPerWeek => SummaryField::Age,
        }
    }

    pub fn prev(self) -> SummaryField {
        match self {
            SummaryField::Age => SummaryField::WorkoutsPerWeek,
            SummaryField::WorkoutsPerWeek => SummaryField::RaceType,
            SummaryField::RaceType => SummaryField::Distance,
            SummaryField::Distance => SummaryField::MafAdjustment,
            SummaryField::MafAdjustment => SummaryField::Age,
        }
    }
}

/// Application state
pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    // User input state
    pub age: u8,
    pub selected_distance_index: usize,
    pub selected_race_type_index: usize,
    pub selected_workouts_per_week: u8,
    pub selected_maf_adjustment_index: usize,

    // Summary editing state
    pub plan_focus: PlanFocus,
    pub selected_summary_field: SummaryField,

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
        let mut app = App {
            screen: Screen::PlanView,
            should_quit: false,
            age: 39,
            selected_distance_index: 2,  // Half Marathon
            selected_race_type_index: 0, // Road
            selected_workouts_per_week: 4,
            selected_maf_adjustment_index: 2, // MafAdjustment::None (index 2)
            plan_focus: PlanFocus::Summary,
            selected_summary_field: SummaryField::Age,
            training_plan: None,
            selected_week: 0,
            selected_workout: 0,
            file_path_input: String::new(),
            status_message: None,
            edit_buffer: String::new(),
            edit_cursor: 0,
        };
        app.generate_plan();
        app
    }

    /// Toggle focus between summary and weeks panels
    pub fn toggle_focus(&mut self) {
        self.plan_focus = match self.plan_focus {
            PlanFocus::Summary => PlanFocus::Weeks,
            PlanFocus::Weeks => PlanFocus::Summary,
        };
    }

    /// Move to next summary field
    pub fn select_next_summary_field(&mut self) {
        self.selected_summary_field = self.selected_summary_field.next();
    }

    /// Move to previous summary field
    pub fn select_prev_summary_field(&mut self) {
        self.selected_summary_field = self.selected_summary_field.prev();
    }

    /// Modify the currently selected summary field (increment/cycle forward)
    pub fn increment_summary_field(&mut self) {
        match self.selected_summary_field {
            SummaryField::Age => {
                if self.age < 100 {
                    self.age += 1;
                    self.regenerate_plan();
                }
            }
            SummaryField::Distance => {
                self.select_next_distance();
                self.regenerate_plan();
            }
            SummaryField::RaceType => {
                self.select_next_race_type();
                self.regenerate_plan();
            }
            SummaryField::WorkoutsPerWeek => {
                self.increase_workouts();
                self.regenerate_plan();
            }
            SummaryField::MafAdjustment => {
                self.select_next_maf_adjustment();
                self.regenerate_plan();
            }
        }
    }

    /// Modify the currently selected summary field (decrement/cycle backward)
    pub fn decrement_summary_field(&mut self) {
        match self.selected_summary_field {
            SummaryField::Age => {
                if self.age > 10 {
                    self.age -= 1;
                    self.regenerate_plan();
                }
            }
            SummaryField::Distance => {
                self.select_prev_distance();
                self.regenerate_plan();
            }
            SummaryField::RaceType => {
                self.select_prev_race_type();
                self.regenerate_plan();
            }
            SummaryField::WorkoutsPerWeek => {
                self.decrease_workouts();
                self.regenerate_plan();
            }
            SummaryField::MafAdjustment => {
                self.select_prev_maf_adjustment();
                self.regenerate_plan();
            }
        }
    }

    /// Regenerate plan and reset week/workout selection
    pub fn regenerate_plan(&mut self) {
        self.generate_plan();
        self.selected_week = 0;
        self.selected_workout = 0;
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
                // Sync app state from loaded plan
                self.age = plan.profile.age;
                self.selected_distance_index = Distance::all()
                    .iter()
                    .position(|d| *d == plan.profile.target_distance)
                    .unwrap_or(0);
                self.selected_race_type_index = RaceType::all()
                    .iter()
                    .position(|r| *r == plan.profile.race_type)
                    .unwrap_or(0);
                self.selected_workouts_per_week = plan.profile.workouts_per_week;
                self.selected_maf_adjustment_index = MafAdjustment::all()
                    .iter()
                    .position(|a| *a == plan.profile.maf_adjustment)
                    .unwrap_or(2); // Default to None (index 2)

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

    pub fn selected_maf_adjustment(&self) -> MafAdjustment {
        MafAdjustment::all()[self.selected_maf_adjustment_index]
    }

    pub fn select_next_maf_adjustment(&mut self) {
        let len = MafAdjustment::all().len();
        self.selected_maf_adjustment_index = (self.selected_maf_adjustment_index + 1) % len;
    }

    pub fn select_prev_maf_adjustment(&mut self) {
        let len = MafAdjustment::all().len();
        self.selected_maf_adjustment_index = (self.selected_maf_adjustment_index + len - 1) % len;
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
            age: self.age,
            target_distance: self.selected_distance(),
            race_type: self.selected_race_type(),
            workouts_per_week: self.selected_workouts_per_week,
            maf_adjustment: self.selected_maf_adjustment(),
        };

        let hr_zones =
            HeartRateZones::from_age_with_adjustment(profile.age, profile.maf_adjustment);
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
    let _taper_weeks = total_weeks
        .saturating_sub(base_weeks)
        .saturating_sub(build_weeks)
        .saturating_sub(peak_weeks);

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

    // App creation tests
    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.screen, Screen::PlanView);
        assert!(!app.should_quit);
        assert!(app.training_plan.is_some()); // Plan generated on startup
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.screen, Screen::PlanView);
        assert_eq!(app.selected_workouts_per_week, 4);
        assert_eq!(app.age, 39);
    }

    #[test]
    fn test_app_starts_with_plan() {
        let app = App::new();
        assert!(app.training_plan.is_some());
        let plan = app.training_plan.as_ref().unwrap();
        assert_eq!(plan.hr_zones.maf_hr, 141); // 180 - 39 = 141
        assert_eq!(plan.weeks.len(), 12); // Half Marathon = 12 weeks
    }

    // Focus toggle tests
    #[test]
    fn test_toggle_focus() {
        let mut app = App::new();
        assert_eq!(app.plan_focus, PlanFocus::Summary);

        app.toggle_focus();
        assert_eq!(app.plan_focus, PlanFocus::Weeks);

        app.toggle_focus();
        assert_eq!(app.plan_focus, PlanFocus::Summary);
    }

    // Summary field navigation tests
    #[test]
    fn test_summary_field_navigation() {
        let mut app = App::new();
        assert_eq!(app.selected_summary_field, SummaryField::Age);

        app.select_next_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::MafAdjustment);

        app.select_next_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::Distance);

        app.select_next_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::RaceType);

        app.select_next_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::WorkoutsPerWeek);

        app.select_next_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::Age); // Wraps around

        app.select_prev_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::WorkoutsPerWeek);

        app.select_prev_summary_field();
        assert_eq!(app.selected_summary_field, SummaryField::RaceType);
    }

    // Age field editing tests
    #[test]
    fn test_age_increment() {
        let mut app = App::new();
        app.selected_summary_field = SummaryField::Age;
        let original_age = app.age;

        app.increment_summary_field();
        assert_eq!(app.age, original_age + 1);

        // Check plan was regenerated with new MAF
        let plan = app.training_plan.as_ref().unwrap();
        assert_eq!(plan.hr_zones.maf_hr, 180 - (original_age + 1) as u16);
    }

    #[test]
    fn test_age_decrement() {
        let mut app = App::new();
        app.selected_summary_field = SummaryField::Age;
        let original_age = app.age;

        app.decrement_summary_field();
        assert_eq!(app.age, original_age - 1);
    }

    #[test]
    fn test_age_limits() {
        let mut app = App::new();
        app.selected_summary_field = SummaryField::Age;

        app.age = 100;
        app.increment_summary_field();
        assert_eq!(app.age, 100); // Should not exceed 100

        app.age = 10;
        app.decrement_summary_field();
        assert_eq!(app.age, 10); // Should not go below 10
    }

    // Distance field editing tests
    #[test]
    fn test_distance_change_regenerates_plan() {
        let mut app = App::new();
        app.selected_summary_field = SummaryField::Distance;
        let original_weeks = app.training_plan.as_ref().unwrap().weeks.len();

        app.increment_summary_field(); // Change to next distance
        let new_weeks = app.training_plan.as_ref().unwrap().weeks.len();

        // Distance changed should affect plan duration
        assert_ne!(original_weeks, new_weeks);
    }

    // Distance selection tests
    #[test]
    fn test_selected_distance() {
        let mut app = App::new();
        app.selected_distance_index = 0;
        assert_eq!(app.selected_distance(), Distance::FiveK);

        app.selected_distance_index = 3;
        assert_eq!(app.selected_distance(), Distance::Marathon);
    }

    #[test]
    fn test_select_next_distance() {
        let mut app = App::new();
        app.selected_distance_index = 0;

        app.select_next_distance();
        assert_eq!(app.selected_distance_index, 1);

        // Test wrap-around
        app.selected_distance_index = 6;
        app.select_next_distance();
        assert_eq!(app.selected_distance_index, 0);
    }

    #[test]
    fn test_select_prev_distance() {
        let mut app = App::new();
        app.selected_distance_index = 1;

        app.select_prev_distance();
        assert_eq!(app.selected_distance_index, 0);

        // Test wrap-around
        app.selected_distance_index = 0;
        app.select_prev_distance();
        assert_eq!(app.selected_distance_index, 6);
    }

    // Race type selection tests
    #[test]
    fn test_selected_race_type() {
        let mut app = App::new();
        app.selected_race_type_index = 0;
        assert_eq!(app.selected_race_type(), RaceType::Road);

        app.selected_race_type_index = 1;
        assert_eq!(app.selected_race_type(), RaceType::Trail);
    }

    #[test]
    fn test_select_next_race_type() {
        let mut app = App::new();
        app.selected_race_type_index = 0;

        app.select_next_race_type();
        assert_eq!(app.selected_race_type_index, 1);

        // Test wrap-around
        app.select_next_race_type();
        assert_eq!(app.selected_race_type_index, 0);
    }

    #[test]
    fn test_select_prev_race_type() {
        let mut app = App::new();
        app.selected_race_type_index = 1;

        app.select_prev_race_type();
        assert_eq!(app.selected_race_type_index, 0);

        // Test wrap-around
        app.select_prev_race_type();
        assert_eq!(app.selected_race_type_index, 1);
    }

    // Workouts per week tests
    #[test]
    fn test_increase_workouts() {
        let mut app = App::new();
        app.selected_workouts_per_week = 4;

        app.increase_workouts();
        assert_eq!(app.selected_workouts_per_week, 5);

        // Test upper limit
        app.selected_workouts_per_week = 7;
        app.increase_workouts();
        assert_eq!(app.selected_workouts_per_week, 7);
    }

    #[test]
    fn test_decrease_workouts() {
        let mut app = App::new();
        app.selected_workouts_per_week = 4;

        app.decrease_workouts();
        assert_eq!(app.selected_workouts_per_week, 3);

        // Test lower limit
        app.selected_workouts_per_week = 2;
        app.decrease_workouts();
        assert_eq!(app.selected_workouts_per_week, 2);
    }

    // Plan generation tests
    #[test]
    fn test_plan_generation() {
        let mut app = App::new();
        app.age = 35;
        app.selected_distance_index = 3; // Marathon
        app.selected_race_type_index = 0; // Road
        app.selected_workouts_per_week = 5;

        app.regenerate_plan();

        assert!(app.training_plan.is_some());
        let plan = app.training_plan.as_ref().unwrap();
        assert_eq!(plan.weeks.len(), 16); // Marathon = 16 weeks
        assert_eq!(plan.hr_zones.maf_hr, 145); // 180 - 35
    }

    #[test]
    fn test_plan_generation_trail() {
        let mut app = App::new();
        app.age = 40;
        app.selected_distance_index = 4; // 50K
        app.selected_race_type_index = 1; // Trail
        app.selected_workouts_per_week = 4;

        app.regenerate_plan();

        let plan = app.training_plan.as_ref().unwrap();
        assert_eq!(plan.profile.race_type, RaceType::Trail);
        assert_eq!(plan.weeks.len(), 16); // 50K = 16 weeks
    }

    // Plan view navigation tests
    #[test]
    fn test_scroll_plan_down() {
        let app = App::new();
        assert_eq!(app.selected_week, 0);

        let mut app = app;
        app.scroll_plan_down();
        assert_eq!(app.selected_week, 1);
    }

    #[test]
    fn test_scroll_plan_up() {
        let mut app = App::new();
        app.selected_week = 2;
        app.scroll_plan_up();
        assert_eq!(app.selected_week, 1);

        // Test lower limit
        app.selected_week = 0;
        app.scroll_plan_up();
        assert_eq!(app.selected_week, 0);
    }

    // Workout helper function tests
    #[test]
    fn test_create_easy_run() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_easy_run(&hr_zones, 45);

        assert_eq!(workout.workout_type, WorkoutType::EasyRun);
        assert_eq!(workout.duration_minutes, 45);
        assert!(workout.description.contains("MAF HR"));
    }

    #[test]
    fn test_create_long_run() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_long_run(&hr_zones, 90, TrainingPhase::Base, false);

        assert_eq!(workout.workout_type, WorkoutType::LongRun);
        assert_eq!(workout.duration_minutes, 90);
    }

    #[test]
    fn test_create_long_run_trail() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_long_run(&hr_zones, 90, TrainingPhase::Base, true);

        assert!(workout.description.contains("varied terrain"));
    }

    #[test]
    fn test_create_long_run_taper() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_long_run(&hr_zones, 60, TrainingPhase::Taper, false);

        assert!(workout.description.contains("staying fresh"));
    }

    #[test]
    fn test_create_recovery_run() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_recovery_run(&hr_zones);

        assert_eq!(workout.workout_type, WorkoutType::RecoveryRun);
        assert_eq!(workout.duration_minutes, 20);
    }

    #[test]
    fn test_create_intervals_build_phase() {
        let workout = create_intervals(TrainingPhase::Build);

        match workout.workout_type {
            WorkoutType::Intervals {
                reps,
                work_minutes,
                rest_minutes,
                target_rpe,
            } => {
                assert_eq!(reps, 4);
                assert_eq!(work_minutes, 3);
                assert_eq!(rest_minutes, 2);
                assert_eq!(target_rpe, RPE::Seven);
            }
            _ => panic!("Expected Intervals workout type"),
        }
    }

    #[test]
    fn test_create_intervals_peak_phase() {
        let workout = create_intervals(TrainingPhase::Peak);

        match workout.workout_type {
            WorkoutType::Intervals { reps, .. } => {
                assert_eq!(reps, 5);
            }
            _ => panic!("Expected Intervals workout type"),
        }
    }

    #[test]
    fn test_create_tempo_run() {
        let workout = create_tempo_run(40);

        match workout.workout_type {
            WorkoutType::TempoRun {
                duration_minutes,
                target_rpe,
            } => {
                assert_eq!(duration_minutes, 24); // 40 * 0.6
                assert_eq!(target_rpe, RPE::Six);
            }
            _ => panic!("Expected TempoRun workout type"),
        }
        assert_eq!(workout.duration_minutes, 40);
    }

    #[test]
    fn test_create_hill_repeats() {
        let workout = create_hill_repeats();

        match workout.workout_type {
            WorkoutType::HillRepeats { reps, target_rpe } => {
                assert_eq!(reps, 6);
                assert_eq!(target_rpe, RPE::Eight);
            }
            _ => panic!("Expected HillRepeats workout type"),
        }
        assert_eq!(workout.duration_minutes, 40);
    }

    #[test]
    fn test_create_technical_trail() {
        let hr_zones = HeartRateZones::from_age(40);
        let workout = create_technical_trail(&hr_zones, 50);

        assert_eq!(workout.workout_type, WorkoutType::TechnicalTrail);
        assert_eq!(workout.duration_minutes, 50);
    }

    #[test]
    fn test_create_vertical_training() {
        let workout = create_vertical_training(Distance::HundredK);

        match workout.workout_type {
            WorkoutType::VerticalTraining {
                elevation_gain_meters,
            } => {
                assert_eq!(elevation_gain_meters, 800);
            }
            _ => panic!("Expected VerticalTraining workout type"),
        }
    }

    #[test]
    fn test_create_vertical_training_hundred_miles() {
        let workout = create_vertical_training(Distance::HundredMiles);

        match workout.workout_type {
            WorkoutType::VerticalTraining {
                elevation_gain_meters,
            } => {
                assert_eq!(elevation_gain_meters, 1000);
            }
            _ => panic!("Expected VerticalTraining workout type"),
        }
    }

    // Training week generation tests
    #[test]
    fn test_generate_training_weeks_periodization() {
        let profile = UserProfile {
            age: 30,
            target_distance: Distance::Marathon,
            race_type: RaceType::Road,
            workouts_per_week: 5,
            maf_adjustment: MafAdjustment::None,
        };
        let hr_zones = HeartRateZones::from_age(30);
        let weeks = generate_training_weeks(&profile, &hr_zones, 16);

        assert_eq!(weeks.len(), 16);

        // Check phase distribution (40% base, 30% build, 20% peak, remaining taper)
        let base_count = weeks
            .iter()
            .filter(|w| w.phase == TrainingPhase::Base)
            .count();
        let build_count = weeks
            .iter()
            .filter(|w| w.phase == TrainingPhase::Build)
            .count();
        let peak_count = weeks
            .iter()
            .filter(|w| w.phase == TrainingPhase::Peak)
            .count();
        let taper_count = weeks
            .iter()
            .filter(|w| w.phase == TrainingPhase::Taper)
            .count();

        // Verify phases are present and in expected proportions
        assert!(base_count >= 5); // Roughly 40% of 16 = 6.4 -> ceil = 7
        assert!(build_count >= 4); // Roughly 30% of 16 = 4.8 -> ceil = 5
        assert!(peak_count >= 3); // Roughly 20% of 16 = 3.2 -> ceil = 4
                                  // Taper might be 0 due to ceiling operations
        assert_eq!(base_count + build_count + peak_count + taper_count, 16);
    }

    #[test]
    fn test_generate_training_weeks_has_workouts() {
        let profile = UserProfile {
            age: 35,
            target_distance: Distance::HalfMarathon,
            race_type: RaceType::Road,
            workouts_per_week: 4,
            maf_adjustment: MafAdjustment::None,
        };
        let hr_zones = HeartRateZones::from_age(35);
        let weeks = generate_training_weeks(&profile, &hr_zones, 12);

        // Each week should have 7 days (workouts + rest)
        for week in weeks {
            assert_eq!(week.workouts.len(), 7);
            assert!(week.total_volume_minutes > 0);
        }
    }
}
