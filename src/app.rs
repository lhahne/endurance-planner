use std::path::PathBuf;

use crate::file_io::{load_plan_from_markdown, save_plan_to_markdown};
use crate::models::{
    Distance, MafAdjustment, RaceType, TrainingPlan, UserProfile,
};
use crate::planner;

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

        self.training_plan = Some(planner::generate_plan(profile));
        self.selected_week = 0;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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
}
