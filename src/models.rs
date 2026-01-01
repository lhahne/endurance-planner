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

    #[test]
    fn test_maffetone_calculation() {
        let zones = HeartRateZones::from_age(40);
        assert_eq!(zones.maf_hr, 140);
        assert_eq!(zones.zone1_max, 130);
        assert_eq!(zones.zone2_max, 140);
    }

    #[test]
    fn test_distance_plan_weeks() {
        assert_eq!(Distance::FiveK.plan_weeks(), 8);
        assert_eq!(Distance::Marathon.plan_weeks(), 16);
    }
}
