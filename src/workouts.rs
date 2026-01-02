use crate::models::{Distance, HeartRateZones, TrainingPhase, Workout, WorkoutType, RPE};

pub fn create_easy_run(hr_zones: &HeartRateZones, duration: u16) -> Workout {
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

pub fn create_long_run(
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

pub fn create_recovery_run(hr_zones: &HeartRateZones) -> Workout {
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

pub fn create_intervals(phase: TrainingPhase) -> Workout {
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

pub fn create_tempo_run(base_duration: u16) -> Workout {
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

pub fn create_hill_repeats() -> Workout {
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

pub fn create_technical_trail(hr_zones: &HeartRateZones, duration: u16) -> Workout {
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

pub fn create_vertical_training(distance: Distance) -> Workout {
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
}
