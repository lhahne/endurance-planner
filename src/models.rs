/// Target race distances
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distance {
    FiveK,
    TenK,
    HalfMarathon,
    Marathon,
    FiftyK,
    HundredK,
    HundredMiles,
}

impl Distance {
    pub fn all() -> &'static [Distance] {
        &[
            Distance::FiveK,
            Distance::TenK,
            Distance::HalfMarathon,
            Distance::Marathon,
            Distance::FiftyK,
            Distance::HundredK,
            Distance::HundredMiles,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Distance::FiveK => "5K",
            Distance::TenK => "10K",
            Distance::HalfMarathon => "Half Marathon (21.1K)",
            Distance::Marathon => "Marathon (42.2K)",
            Distance::FiftyK => "50K Ultra",
            Distance::HundredK => "100K Ultra",
            Distance::HundredMiles => "100 Miles Ultra",
        }
    }

    /// Returns typical training plan duration in weeks
    pub fn plan_weeks(&self) -> u8 {
        match self {
            Distance::FiveK => 8,
            Distance::TenK => 10,
            Distance::HalfMarathon => 12,
            Distance::Marathon => 16,
            Distance::FiftyK => 16,
            Distance::HundredK => 20,
            Distance::HundredMiles => 24,
        }
    }

    /// Returns kilometers for the distance
    pub fn kilometers(&self) -> f32 {
        match self {
            Distance::FiveK => 5.0,
            Distance::TenK => 10.0,
            Distance::HalfMarathon => 21.1,
            Distance::Marathon => 42.2,
            Distance::FiftyK => 50.0,
            Distance::HundredK => 100.0,
            Distance::HundredMiles => 160.9,
        }
    }
}

/// Race terrain type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RaceType {
    Road,
    Trail,
}

impl RaceType {
    pub fn all() -> &'static [RaceType] {
        &[RaceType::Road, RaceType::Trail]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RaceType::Road => "Road",
            RaceType::Trail => "Trail",
        }
    }
}

/// Rate of Perceived Exertion scale (1-10)
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code, clippy::upper_case_acronyms)]
pub enum RPE {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl RPE {
    pub fn value(&self) -> u8 {
        match self {
            RPE::One => 1,
            RPE::Two => 2,
            RPE::Three => 3,
            RPE::Four => 4,
            RPE::Five => 5,
            RPE::Six => 6,
            RPE::Seven => 7,
            RPE::Eight => 8,
            RPE::Nine => 9,
            RPE::Ten => 10,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RPE::One => "Very light - barely any effort",
            RPE::Two => "Light - easy conversation",
            RPE::Three => "Moderate - comfortable pace",
            RPE::Four => "Somewhat hard - breathing harder",
            RPE::Five => "Hard - can speak in short sentences",
            RPE::Six => "Harder - conversation difficult",
            RPE::Seven => "Very hard - short phrases only",
            RPE::Eight => "Very, very hard - few words",
            RPE::Nine => "Extremely hard - max sustainable",
            RPE::Ten => "Maximum effort - all out sprint",
        }
    }
}

/// Heart rate zones based on Maffetone formula
#[derive(Debug, Clone)]
pub struct HeartRateZones {
    pub maf_hr: u16,    // Maximum Aerobic Function heart rate
    pub zone1_max: u16, // Recovery zone (MAF - 20 to MAF - 10)
    pub zone2_max: u16, // Aerobic base zone (MAF - 10 to MAF)
}

impl HeartRateZones {
    /// Calculate heart rate zones using Maffetone formula
    /// MAF = 180 - age (with adjustments)
    pub fn from_age(age: u8) -> Self {
        let maf_hr = Self::calculate_maf(age);
        HeartRateZones {
            maf_hr,
            zone1_max: maf_hr.saturating_sub(10),
            zone2_max: maf_hr,
        }
    }

    fn calculate_maf(age: u8) -> u16 {
        // Basic Maffetone formula: 180 - age
        // For simplicity, not applying adjustments here
        // Users could add their own adjustments based on fitness level
        180u16.saturating_sub(age as u16)
    }

    pub fn zone1_range(&self) -> (u16, u16) {
        (self.maf_hr.saturating_sub(20), self.zone1_max)
    }

    pub fn zone2_range(&self) -> (u16, u16) {
        (self.zone1_max + 1, self.zone2_max)
    }
}

/// Types of workouts
#[derive(Debug, Clone, PartialEq)]
pub enum WorkoutType {
    /// Easy aerobic run at MAF heart rate
    EasyRun,
    /// Long slow distance run
    LongRun,
    /// Recovery run (very easy)
    RecoveryRun,
    /// Interval training (uses RPE)
    Intervals {
        reps: u8,
        work_minutes: u8,
        rest_minutes: u8,
        target_rpe: RPE,
    },
    /// Tempo run (sustained effort)
    TempoRun {
        duration_minutes: u16,
        target_rpe: RPE,
    },
    /// Hill repeats
    HillRepeats { reps: u8, target_rpe: RPE },
    /// Trail-specific: technical terrain practice
    TechnicalTrail,
    /// Trail-specific: elevation gain focus
    VerticalTraining { elevation_gain_meters: u16 },
    /// Rest day
    Rest,
}

impl WorkoutType {
    pub fn name(&self) -> &'static str {
        match self {
            WorkoutType::EasyRun => "Easy Run",
            WorkoutType::LongRun => "Long Run",
            WorkoutType::RecoveryRun => "Recovery Run",
            WorkoutType::Intervals { .. } => "Intervals",
            WorkoutType::TempoRun { .. } => "Tempo Run",
            WorkoutType::HillRepeats { .. } => "Hill Repeats",
            WorkoutType::TechnicalTrail => "Technical Trail",
            WorkoutType::VerticalTraining { .. } => "Vertical Training",
            WorkoutType::Rest => "Rest",
        }
    }
}

/// A single workout session
#[derive(Debug, Clone)]
pub struct Workout {
    pub workout_type: WorkoutType,
    pub duration_minutes: u16,
    pub description: String,
}

/// A training week
#[derive(Debug, Clone)]
pub struct TrainingWeek {
    pub week_number: u8,
    pub phase: TrainingPhase,
    pub workouts: Vec<Workout>,
    pub total_volume_minutes: u16,
}

/// Training phases in periodization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrainingPhase {
    Base,
    Build,
    Peak,
    Taper,
}

impl TrainingPhase {
    pub fn name(&self) -> &'static str {
        match self {
            TrainingPhase::Base => "Base Building",
            TrainingPhase::Build => "Build Phase",
            TrainingPhase::Peak => "Peak Training",
            TrainingPhase::Taper => "Taper",
        }
    }
}

/// User's training preferences and parameters
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub age: u8,
    pub target_distance: Distance,
    pub race_type: RaceType,
    pub workouts_per_week: u8,
}

/// Complete training plan
#[derive(Debug, Clone)]
pub struct TrainingPlan {
    pub profile: UserProfile,
    pub hr_zones: HeartRateZones,
    pub weeks: Vec<TrainingWeek>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Distance tests
    #[test]
    fn test_distance_all() {
        let distances = Distance::all();
        assert_eq!(distances.len(), 7);
        assert_eq!(distances[0], Distance::FiveK);
        assert_eq!(distances[6], Distance::HundredMiles);
    }

    #[test]
    fn test_distance_name() {
        assert_eq!(Distance::FiveK.name(), "5K");
        assert_eq!(Distance::TenK.name(), "10K");
        assert_eq!(Distance::HalfMarathon.name(), "Half Marathon (21.1K)");
        assert_eq!(Distance::Marathon.name(), "Marathon (42.2K)");
        assert_eq!(Distance::FiftyK.name(), "50K Ultra");
        assert_eq!(Distance::HundredK.name(), "100K Ultra");
        assert_eq!(Distance::HundredMiles.name(), "100 Miles Ultra");
    }

    #[test]
    fn test_distance_plan_weeks() {
        assert_eq!(Distance::FiveK.plan_weeks(), 8);
        assert_eq!(Distance::TenK.plan_weeks(), 10);
        assert_eq!(Distance::HalfMarathon.plan_weeks(), 12);
        assert_eq!(Distance::Marathon.plan_weeks(), 16);
        assert_eq!(Distance::FiftyK.plan_weeks(), 16);
        assert_eq!(Distance::HundredK.plan_weeks(), 20);
        assert_eq!(Distance::HundredMiles.plan_weeks(), 24);
    }

    #[test]
    fn test_distance_kilometers() {
        assert_eq!(Distance::FiveK.kilometers(), 5.0);
        assert_eq!(Distance::TenK.kilometers(), 10.0);
        assert_eq!(Distance::HalfMarathon.kilometers(), 21.1);
        assert_eq!(Distance::Marathon.kilometers(), 42.2);
        assert_eq!(Distance::FiftyK.kilometers(), 50.0);
        assert_eq!(Distance::HundredK.kilometers(), 100.0);
        assert_eq!(Distance::HundredMiles.kilometers(), 160.9);
    }

    // RaceType tests
    #[test]
    fn test_race_type_all() {
        let types = RaceType::all();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], RaceType::Road);
        assert_eq!(types[1], RaceType::Trail);
    }

    #[test]
    fn test_race_type_name() {
        assert_eq!(RaceType::Road.name(), "Road");
        assert_eq!(RaceType::Trail.name(), "Trail");
    }

    // RPE tests
    #[test]
    fn test_rpe_value() {
        assert_eq!(RPE::One.value(), 1);
        assert_eq!(RPE::Two.value(), 2);
        assert_eq!(RPE::Three.value(), 3);
        assert_eq!(RPE::Four.value(), 4);
        assert_eq!(RPE::Five.value(), 5);
        assert_eq!(RPE::Six.value(), 6);
        assert_eq!(RPE::Seven.value(), 7);
        assert_eq!(RPE::Eight.value(), 8);
        assert_eq!(RPE::Nine.value(), 9);
        assert_eq!(RPE::Ten.value(), 10);
    }

    #[test]
    fn test_rpe_description() {
        assert_eq!(RPE::One.description(), "Very light - barely any effort");
        assert_eq!(RPE::Five.description(), "Hard - can speak in short sentences");
        assert_eq!(RPE::Ten.description(), "Maximum effort - all out sprint");
    }

    // HeartRateZones tests
    #[test]
    fn test_maffetone_calculation() {
        let zones = HeartRateZones::from_age(40);
        assert_eq!(zones.maf_hr, 140);
        assert_eq!(zones.zone1_max, 130);
        assert_eq!(zones.zone2_max, 140);
    }

    #[test]
    fn test_maffetone_young_athlete() {
        let zones = HeartRateZones::from_age(25);
        assert_eq!(zones.maf_hr, 155); // 180 - 25
    }

    #[test]
    fn test_maffetone_older_athlete() {
        let zones = HeartRateZones::from_age(60);
        assert_eq!(zones.maf_hr, 120); // 180 - 60
    }

    #[test]
    fn test_zone1_range() {
        let zones = HeartRateZones::from_age(40);
        let (min, max) = zones.zone1_range();
        assert_eq!(min, 120); // MAF - 20
        assert_eq!(max, 130); // MAF - 10
    }

    #[test]
    fn test_zone2_range() {
        let zones = HeartRateZones::from_age(40);
        let (min, max) = zones.zone2_range();
        assert_eq!(min, 131); // zone1_max + 1
        assert_eq!(max, 140); // MAF
    }

    #[test]
    fn test_heart_rate_zones_edge_case_very_young() {
        let zones = HeartRateZones::from_age(10);
        assert_eq!(zones.maf_hr, 170);
        let (z1_min, z1_max) = zones.zone1_range();
        assert_eq!(z1_min, 150);
        assert_eq!(z1_max, 160);
    }

    // WorkoutType tests
    #[test]
    fn test_workout_type_name() {
        assert_eq!(WorkoutType::EasyRun.name(), "Easy Run");
        assert_eq!(WorkoutType::LongRun.name(), "Long Run");
        assert_eq!(WorkoutType::RecoveryRun.name(), "Recovery Run");
        assert_eq!(WorkoutType::Rest.name(), "Rest");
        assert_eq!(
            WorkoutType::Intervals {
                reps: 5,
                work_minutes: 4,
                rest_minutes: 2,
                target_rpe: RPE::Seven
            }
            .name(),
            "Intervals"
        );
        assert_eq!(
            WorkoutType::TempoRun {
                duration_minutes: 20,
                target_rpe: RPE::Six
            }
            .name(),
            "Tempo Run"
        );
        assert_eq!(
            WorkoutType::HillRepeats {
                reps: 6,
                target_rpe: RPE::Eight
            }
            .name(),
            "Hill Repeats"
        );
        assert_eq!(WorkoutType::TechnicalTrail.name(), "Technical Trail");
        assert_eq!(
            WorkoutType::VerticalTraining {
                elevation_gain_meters: 500
            }
            .name(),
            "Vertical Training"
        );
    }

    // TrainingPhase tests
    #[test]
    fn test_training_phase_name() {
        assert_eq!(TrainingPhase::Base.name(), "Base Building");
        assert_eq!(TrainingPhase::Build.name(), "Build Phase");
        assert_eq!(TrainingPhase::Peak.name(), "Peak Training");
        assert_eq!(TrainingPhase::Taper.name(), "Taper");
    }
}
