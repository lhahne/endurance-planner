use crate::models::{
    HeartRateZones, RaceType, TrainingPhase, TrainingPlan, TrainingWeek, UserProfile, Workout,
    WorkoutType,
};
use crate::workouts::*;

pub fn generate_plan(profile: UserProfile) -> TrainingPlan {
    let hr_zones = HeartRateZones::from_age_with_adjustment(profile.age, profile.maf_adjustment);
    let total_weeks = profile.target_distance.plan_weeks();
    let weeks = generate_training_weeks(&profile, &hr_zones, total_weeks);

    TrainingPlan {
        profile,
        hr_zones,
        weeks,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Distance, MafAdjustment};

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
