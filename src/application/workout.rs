// Workouts can be defined in a TOML file to be loaded for a user to select
// and perform. A workout will consist of a name and laps with intervals at set
// power levels or percentage of FTP.
//
// Name is required.
// At least one lap is required for aggregating display data. Each lap can have
// one or more intervals that make up the lap. For example, an interval
// consisting of 10 30s on and 30s off intervals could be made up of 20
// laps with one 30s interval each or 1 lap with 20 30s intervals or any combination
// in between that the user wants. The purpose of the lap is just consolidation
// of data for display, such as lap average power, lap average HR, etc.
//
// Well this example is ugly and not what I'm trying to accomplish.
// There should be workouts and segments. A user could create a workout file
// that would be unique in that it defined all laps/intervals for the workout.
// A user should also be able to create segments that they could then piece
// together to create a workout.
// (Future) Could allow a user to create a workout in the UI by selecting
// existing segments. Could also allow users to create segments in the UI, but
// really trying to avoid having a workout creation that takes a while to create
// a workout such as 60 30/30 intervals. Want the program to create it from
// their desired input, but also give the user the capability to create something
// custom if they want to spend that much time creating a workout.
//
// // workout example
// name = 'Metcalfe'
// description = A 2x20 workout best done to 80s hair bands
// duration = 1h # 1h30m
// intervals = [
//   'Warmup',
//   '20m@.85', # '20m @ .85'
//   '5m@.55',
//   '20m@.85',
//   '5m@.55'
//  ]
//  lap_each_interval = true
//
// Interval example
// name = 'Warmup'
// description ='Warmup to get the legs ready'
// duration = '10m'
// segments = [
//   '5m@100', #5'@100
//   '1m@110',
//   '1m@120',
//   '1m@130',
//   '2m@100'
// ]
// lap_each_segment = false
//
// Another interval example
// name = '30 on/30 off'
// description = '30 seconds on at 320w, 30 seconds off'
// duration = '1m',
// segments = [
//   '30s@320', #30"@320
//   '30s@.55',
// ]
// lap_each_segment = true,
//
// Workout example
// name = '30x30x30'
// description = '30 by 30 seconds on with 30 seconds recovery'
// duration = '45m'
// intervals = [
//   'Warmup',
//   { name = '30 on/30 off', repeat = 30 },
//   '5m@.55' #5:30m 5m30s 1h10m30s
// ]
// lap_each_interval = true
//
// Workout example
// name = '30x30x30'
// description = '30 by 30 seconds on with 30 seconds recovery'
// duration = '45m'
// intervals = [
//   'Warmup',
//   { name = '30 on/30 off', segments = ["0:@1.5"], repeat = 30 },
//   '5m@.55'
// ]
// lap_each_interval = true
// Workout consists of one or more laps.
// Lap consists of one or more intervals.
// Interval consists of one or more segments.
// Segment has a duration and power target(s)

// Workout template takes in TOML file provided by user. Workout will take a
// reference to the workout template, validate, and generate the workout. This
// is to support referencing intervals by name which would be imported and validated
// prior to importing workouts.
//
#![allow(dead_code)]
use crate::application::types::{Duration, PowerTarget, Quantity, StartTime};

use serde::de::Unexpected;
use serde_derive::Deserialize;

use std::str::FromStr;

#[derive(Clone)]
struct Segment {
    duration: Duration,
    power_start: PowerTarget,
    power_end: PowerTarget,
    start_time: StartTime, //Start time in seconds.
}

impl<'de> serde::Deserialize<'de> for Segment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SegType {
            A(String),
            B {
                duration: Duration,
                power_start: PowerTarget,
                power_end: PowerTarget,
            },
        }

        let seg = SegType::deserialize(deserializer)?;
        let (duration, power_start, power_end) = match seg {
            SegType::A(value) => {
                // Let's split our string and remove whitespaces.
                let value_vec: Vec<&str> = value
                    .split('@')
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|x| x.trim())
                    .collect();
                // if value_vec.len() != 2 { return error }
                // TODO: May need to look into support a partial segment
                let duration: Duration = value_vec[0]
                    .parse()
                    .map_err(|e| Error::invalid_value(Unexpected::Str(value_vec[0]), &e))?;
                let power_target: PowerTarget = value_vec[1]
                    .parse()
                    .map_err(|e| Error::invalid_value(Unexpected::Str(value_vec[1]), &e))?;
                (duration, power_target, power_target)
            }
            SegType::B {
                duration,
                power_start,
                power_end,
            } => (duration, power_start, power_end),
        };
        Ok(Segment {
            duration,
            power_start,
            power_end,
            start_time: 0.into(),
        })
    }
}

// Interval example
// name = 'Warmup'
// description ='Warmup to get the legs ready'
// duration = '10m'
// segments = [
//   '5m@100', #5'@100
//   '1m@110',
//   '1m@120',
//   '1m@130',
//   '2m@100'
// ]
// lap_each_segment = false

#[derive(Clone, Deserialize)]
struct IntervalTemplate {
    name: Option<String>,
    description: Option<String>,
    duration: Duration,
    segments: Vec<Segment>,
    lap_each_segment: bool,
    repeat: Option<Quantity>,
}

#[derive(Clone)]
struct ValidateIntervalTemplate {
    name: String,
    duration: Option<Duration>,
    segments: Option<Vec<SegmentUpdate>>,
    lap_each_segment: Option<bool>,
    repeat: Option<Quantity>,
}

#[derive(Clone)]
struct SegmentUpdate {
    index: usize,
    duration: Option<Duration>,
    power_start: Option<PowerTarget>,
    power_end: Option<PowerTarget>,
}

#[derive(Clone, Deserialize)]
struct ShadowIntervalTemplate {
    name: String,
    description: String,
    duration: Duration,
    segments: Vec<Segment>,
    lap_each_segment: bool,
    repeat: Option<Quantity>,
}

//TODO implement proper errors
impl ShadowIntervalTemplate {
    fn validate(&self) -> Result<(), &'static str> {
        let seg_duration = self
            .segments
            .iter()
            .fold(Duration::from(0), |acc, x| acc + x.duration);
        if self.duration != seg_duration * self.repeat.unwrap_or(Quantity(1)) {
            return Err("duration does not match duration calculated from segments");
        }
        Ok(())
    }

    fn build(self) -> IntervalTemplate {
        IntervalTemplate {
            name: Some(self.name),
            description: Some(self.description),
            duration: self.duration,
            segments: self.segments,
            lap_each_segment: self.lap_each_segment,
            repeat: self.repeat,
        }
    }
}

// // workout example
// [[ workout ]]
// name = 'Metcalfe'
// description = "A 2x20 workout best done to 80s hair bands"
// duration = "1h" # 1h30m
// intervals = [
//   'Warmup',
//   '20m@.85', # '20m @ .85'
//   '5m@.55',
//   '20m@.85',
//   '5m@.55'
//  ]
//  lap_each_interval = true
//
// // workout example
// [[ workout ]]
// name = "30x30s"
// description = "A 2x20 workout best done to 80s hair bands"
// duration = "1h" # 1h30m
// intervals = [
//   'Warmup',
//   { name = "30on/30off', repeat = 30 },
//   '5m@.55'
// ]
//  lap_each_interval = true
//
// Use IntervalTemplateType during import to find what intervals to validate
// before pushing our WorkoutTemplate into the library. Any intervals that
// need to be validated will be from our map of intervals already created.
//
// This could look like:
// 'Warmup" # Just a string
// '10m@.95' # A segment to be turned into an IntervalType
// { name = '30on/30off', repeat = 30 } # a name interval with changed parameters
enum IntervalTemplateType {
    // A name of an Interval or a segment that can be parsed into a string.
    Validate(String),
    // An IntervalTemplate that needs to be validated by name and has fields
    // to update in a found IntervalTemplate
    ValidateAndUpdate(ValidateIntervalTemplate),
    // A valid IntervalTemplate
    IntervalTemplate(IntervalTemplate),
}

impl<'de> serde::Deserialize<'de> for IntervalTemplateType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum IntervalType {
            // 'Warmup' or '5m@.55'
            A(String),
            // Range { duration = '1m', power_start: .75, power_end: .85 }
            B {
                duration: Duration,
                power_start: PowerTarget,
                power_end: PowerTarget,
                repeat: Option<Quantity>,
                lap_each_segment: Option<bool>,
            },
            C {
                name: String,
                duration: Option<Duration>,
                lap_each_segment: Option<bool>,
                repeat: Option<Quantity>,
                segments: Option<Vec<String>>,
            },
        }

        match IntervalType::deserialize(deserializer)? {
            IntervalType::A(value) => {
                let v: Vec<&str> = value
                    .split('@')
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|x| x.trim())
                    .collect();
                // If len == 1, then set to Validate
                if v.len() == 1 {
                    return Ok(Self::Validate(String::from(v[0])));
                }
                // Parse our segment and crate an interval type
                let duration: Duration = v[0]
                    .parse()
                    .map_err(|e| Error::invalid_value(Unexpected::Str(v[0]), &e))?;
                let power_target: PowerTarget = v[1]
                    .parse()
                    .map_err(|e| Error::invalid_value(Unexpected::Str(v[1]), &e))?;
                let segment = Segment {
                    duration,
                    power_start: power_target,
                    power_end: power_target,
                    start_time: 0.into(),
                };
                let interval_template = IntervalTemplate {
                    name: Some(String::from("this should be optional")),
                    description: Some(String::from("this should be optional")),
                    duration,
                    lap_each_segment: false,
                    segments: vec![segment],
                    repeat: None,
                };
                Ok(Self::IntervalTemplate(interval_template))
            }
            IntervalType::B {
                duration,
                power_start,
                power_end,
                repeat,
                lap_each_segment,
            } => {
                let interval_duration = duration * repeat.unwrap_or(Quantity(1));
                let segment = Segment {
                    duration,
                    power_start,
                    power_end,
                    start_time: 0.into(),
                };
                // Need to validate IntervalTemplates that are created from segments
                // passed into the WorkoutTemplate
                Ok(Self::IntervalTemplate(IntervalTemplate {
                    name: Some(String::from("this should be optional")),
                    description: Some(String::from("this should be optional")),
                    duration: interval_duration,
                    lap_each_segment: lap_each_segment.unwrap_or(false),
                    segments: vec![segment],
                    repeat,
                }))
            }
            IntervalType::C {
                name,
                duration,
                repeat,
                lap_each_segment,
                segments,
            } => {
                // TODO catch all these unwrap(s)()
                let mut merge_segments: Vec<SegmentUpdate> = Vec::new();
                if let Some(segs) = segments {
                    for seg in segs {
                        // Split segment by ':' to find out which segment to update
                        let v1: Vec<&str> = seg
                            .split(':')
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|x| x.trim())
                            .collect::<Vec<&str>>();
                        // Split by '@' to find out if duration or power target is updated.
                        let v2: Vec<&str> = v1[1]
                            .split('@')
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|x| x.trim())
                            .collect::<Vec<&str>>();
                        let duration = if !v2[0].is_empty() {
                            Some(Duration::from_str(v2[0]).unwrap())
                        } else {
                            None
                        };

                        let power_target = if !v2[1].is_empty() {
                            Some(PowerTarget::from_str(v2[1]).unwrap())
                        } else {
                            None
                        };
                        merge_segments.push(SegmentUpdate {
                            index: v1[0].parse::<usize>().unwrap(),
                            duration,
                            power_start: power_target,
                            power_end: power_target,
                        });
                    }
                }
                Ok(Self::ValidateAndUpdate(ValidateIntervalTemplate {
                    name,
                    duration,
                    repeat,
                    lap_each_segment,
                    segments: if merge_segments.len() != 0 {
                        Some(merge_segments)
                    } else {
                        None
                    },
                }))
            }
        }
    }
}
#[derive(Deserialize)]
struct WorkoutTemplate {
    name: String,
    description: String,
    duration: Duration,
    lap_each_interval: bool,
    intervals: Vec<IntervalTemplate>,
}

#[derive(Deserialize)]
struct ShadowWorkoutTemplate {
    name: String,
    description: String,
    duration: Duration,
    lap_each_interval: bool,
    intervals: Vec<IntervalTemplateType>,
}
impl ShadowWorkoutTemplate {
    fn validate(
        &mut self,
        interval_templates: &BTreeMap<String, IntervalTemplate>,
    ) -> Result<(), &'static str> {
        // IntervalTemplate gets validated for duration upon creation. This
        // validates the WorkoutTemplate duration and transforms any
        // IntervalTemplateType::Validate to IntervalTemplateType::IntervalTemplate or
        // returns an error.
        // Keep track of our durations
        let mut duration = Duration(0);
        for interval_type in self.intervals.iter_mut() {
            match interval_type {
                IntervalTemplateType::Validate(value) => {
                    // Need to return an error here template can't be found
                    if let Some(template) = interval_templates.get(value) {
                        duration += template.duration;
                        *interval_type = IntervalTemplateType::IntervalTemplate(template.clone())
                    }
                }
                IntervalTemplateType::IntervalTemplate(template) => duration += template.duration,
                IntervalTemplateType::ValidateAndUpdate(validate_template) => {
                    match interval_templates.get(&validate_template.name) {
                        Some(template) => {
                            let mut t = template.clone();
                            if let Some(d) = validate_template.duration {
                                t.duration = d;
                            }
                            if let Some(les) = validate_template.lap_each_segment {
                                t.lap_each_segment = les;
                            }
                            if let Some(repeat) = validate_template.repeat {
                                t.repeat = Some(repeat);
                            }
                            if let Some(segments) = &validate_template.segments {
                                for seg in segments {
                                    if let Some(d) = seg.duration {
                                        t.segments[seg.index].duration = d;
                                    }
                                    if let Some(ps) = seg.power_start {
                                        t.segments[seg.index].power_start = ps;
                                    }
                                    if let Some(pe) = seg.power_end {
                                        t.segments[seg.index].power_end = pe;
                                    }
                                }
                            }
                            duration += t.duration;
                            *interval_type = IntervalTemplateType::IntervalTemplate(t.clone());
                        }
                        None => return Err("Template not found"), // return error
                    }
                }
            }
        }
        if self.duration != duration {
            return Err("defined duration does not match sum of interval durations");
        }
        Ok(())
    }

    fn build_workout_template(self) -> WorkoutTemplate {
        let intervals: Vec<IntervalTemplate> = self
            .intervals
            .iter()
            .map(|x| match x {
                IntervalTemplateType::IntervalTemplate(template) => template.clone(),
                _ => panic!("Invalid IntervalTemplateType when creating WorkoutTemplate"),
            })
            .collect();
        WorkoutTemplate {
            name: self.name,
            description: self.description,
            duration: self.duration,
            lap_each_interval: self.lap_each_interval,
            intervals,
        }
    }
}

//
// There should be an intermediate structure for reading in workouts (especially)
// so that workouts can be validated against known intervals if an interval
// is specified in a workout. Workouts and templates should be stored in Hash
// or BtreeMap for easy searching of intervals especially.
//
// Intervals and workouts can be listed in the same file or in separate files.
// Read in all intervals and validate the intervals.
// Read in all workouts, validate workouts against known intervals, and build
// a workout by expanding out
use std::collections::BTreeMap;
struct Library {
    intervals: BTreeMap<String, IntervalTemplate>,
    workouts: BTreeMap<String, WorkoutTemplate>,
}

#[derive(Deserialize)]
struct ShadowLibrary {
    intervals: Option<Vec<ShadowIntervalTemplate>>,
    workouts: Option<Vec<ShadowWorkoutTemplate>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_library_update_interval_info() {
        // Sample intervals like they would be read from multiple files.
        let s1 = r#"
        [[ intervals ]]
        name = "Warmup"
        description = "This needs to be optional"
        duration = "10m"
        lap_each_segment = false
        segments = [
          '5m@100',
          '1m@110',
          '1m@120',
          '1m@130',
          '2m@100',
        ]

        [[ intervals ]]
        name = "Cooldown"
        description = "This needs to be optional"
        duration = "5m"
        lap_each_segment = false
        segments = ['5m@.55']
        "#;

        let s2 = r#"
        [[ intervals ]]
        name = "2by20"
        description = "This needs to be optional"
        duration = "45m"
        lap_each_segment = true
        segments = [
          '20m@.85',
          '5m@.55',
          '20m@.85'
        ]

        [[ workouts ]]
        name = "Metcalfe"
        description = "2x20 at tempo"
        duration = "1h20m"
        lap_each_interval = true
        intervals = [
          'Warmup',
          { name = '2by20', duration = '1h5m', segments = ["0:30m@", "2:30m@"]},
          { duration = "5m", power_start = 0.85, power_end = 0.55 },
        ]"#;

        let mut library = Library {
            intervals: BTreeMap::new(),
            workouts: BTreeMap::new(),
        };
        for file in vec![s1, s2].iter() {
            // This is a simplied version of reading in file contents then
            // parsing the contents into our data structure.
            let sl: ShadowLibrary = toml::from_str(file).unwrap();
            // Loop through read in intervals
            if let Some(contents) = sl.intervals {
                for interval in contents {
                    if let Err(_e) = interval.validate() {
                        // log error
                        continue;
                    }
                    match library.intervals.contains_key(&interval.name) {
                        true => continue, //Also log error for duplicate key
                        false => {
                            library
                                .intervals
                                .insert(interval.name.clone(), interval.build());
                        }
                    }
                }
            }
            if let Some(contents) = sl.workouts {
                for mut shadow_workout in contents {
                    // Need a build and a validate method. Validate the duration
                    // and any interval templates passed in. Merge interval templates, etc.
                    if let Err(_e) = shadow_workout.validate(&library.intervals) {
                        // log error
                        continue;
                    }
                    let workout = shadow_workout.build_workout_template();
                    library.workouts.insert(workout.name.clone(), workout);
                }
            }
        }
        assert_eq!(library.intervals.len(), 3);
        assert_eq!(library.workouts.len(), 1);
        let sample_workout = library.workouts.get("Metcalfe").unwrap();
        assert_eq!(
            sample_workout.duration,
            Duration::from_str("1h20m").unwrap()
        );
        assert_eq!(
            sample_workout.intervals[1].segments[0].duration,
            Duration::from_str("30m").unwrap()
        );
        assert_eq!(sample_workout.intervals[0].segments.len(), 5);
        assert_eq!(sample_workout.intervals[1].segments.len(), 3);
        assert_eq!(
            sample_workout.intervals[2].segments[0].power_start,
            PowerTarget::Percentage(0.85)
        );
    }
    #[test]
    fn test_library() {
        // Sample intervals like they would be read from multiple files.
        let s1 = r#"
        [[ intervals ]]
        name = "Warmup"
        description = "This needs to be optional"
        duration = "10m"
        lap_each_segment = false
        segments = [
          '5m@100',
          '1m@110',
          '1m@120',
          '1m@130',
          '2m@100',
        ]

        [[ intervals ]]
        name = "Cooldown"
        description = "This needs to be optional"
        duration = "5m"
        lap_each_segment = false
        segments = ['5m@.55']
        "#;

        let s2 = r#"
        [[ intervals ]]
        name = "2by20"
        description = "This needs to be optional"
        duration = "45m"
        lap_each_segment = true
        segments = [
          '20m@.85',
          '5m@.55',
          '20m@.85'
        ]

        [[ workouts ]]
        name = "Metcalfe"
        description = "2x20 at tempo"
        duration = "1h"
        lap_each_interval = true
        intervals = [
          'Warmup',
          '2by20',
          { duration = "5m", power_start = 0.85, power_end = 0.55 },
        ]"#;

        let mut library = Library {
            intervals: BTreeMap::new(),
            workouts: BTreeMap::new(),
        };
        for file in vec![s1, s2].iter() {
            // This is a simplied version of reading in file contents then
            // parsing the contents into our data structure.
            let sl: ShadowLibrary = toml::from_str(file).unwrap();
            // Loop through read in intervals
            if let Some(contents) = sl.intervals {
                for interval in contents {
                    if let Err(_e) = interval.validate() {
                        // log error
                        continue;
                    }
                    match library.intervals.contains_key(&interval.name) {
                        true => continue, //Also log error for duplicate key
                        false => {
                            library
                                .intervals
                                .insert(interval.name.clone(), interval.build());
                        }
                    }
                }
            }
            if let Some(contents) = sl.workouts {
                for mut shadow_workout in contents {
                    // Need a build and a validate method. Validate the duration
                    // and any interval templates passed in. Merge interval templates, etc.
                    if let Err(_e) = shadow_workout.validate(&library.intervals) {
                        // log error
                        continue;
                    }
                    let workout = shadow_workout.build_workout_template();
                    library.workouts.insert(workout.name.clone(), workout);
                }
            }
        }
        assert_eq!(library.intervals.len(), 3);
        assert_eq!(library.workouts.len(), 1);
        let sample_workout = library.workouts.get("Metcalfe").unwrap();
        assert_eq!(sample_workout.duration, Duration::from_str("1h").unwrap());
        assert_eq!(sample_workout.intervals[0].segments.len(), 5);
        assert_eq!(sample_workout.intervals[1].segments.len(), 3);
        assert_eq!(
            sample_workout.intervals[2].segments[0].power_start,
            PowerTarget::Percentage(0.85)
        );
    }

    #[test]
    fn test_library_single_workout() {
        // Sample intervals like they would be read from multiple files.
        let s1 = r#"
        [[ workouts ]]
        name = "Sample workout"
        description = "Test sample"
        duration = "1h"
        lap_each_interval = true
        intervals = [
          '10m@.55',
          '40m@225',
          '10m@.55'
        ]"#;

        let mut library = Library {
            intervals: BTreeMap::new(),
            workouts: BTreeMap::new(),
        };

        let sl: ShadowLibrary = toml::from_str(s1).unwrap();
        if let Some(contents) = sl.workouts {
            for shadow_workout in contents {
                let workout = shadow_workout.build_workout_template();
                library.workouts.insert(workout.name.clone(), workout);
            }
        }
        assert_eq!(library.workouts.len(), 1);
        let sample_workout = library.workouts.get("Sample workout").unwrap();
        assert_eq!(sample_workout.duration, Duration::from_str("1h").unwrap());
        assert_eq!(sample_workout.intervals.len(), 3);
        assert_eq!(
            sample_workout.intervals[0].duration,
            Duration::from_str("10m").unwrap()
        );
        assert_eq!(
            sample_workout.intervals[0].segments[0].power_start,
            PowerTarget::Percentage(0.55)
        );
    }

    #[test]
    fn test_shadow_interval_template() {
        let it_str = r#"
        name = "Warmup"
        description = "Warming up the legs"
        duration = "10m"
        lap_each_segment = false
        segments = [
          '5m@100',
          '1m@110',
          '1m@120',
          '1m@130',
          '2m@100',
        ]"#;

        let foo: ShadowIntervalTemplate = toml::from_str(it_str).unwrap();
        assert_eq!(foo.name, String::from("Warmup"));
        assert_eq!(foo.description, String::from("Warming up the legs"));
        assert_eq!(foo.duration, Duration::from_str("10m").unwrap());
        assert!(!foo.lap_each_segment);
        assert_eq!(foo.segments.len(), 5);
    }

    #[test]
    fn test_shadow_interval_template_with_repeat() {
        let it_str = r#"
        name = "30on/30off"
        description = "30s on @ 1.2, 30s off @ .55"
        duration = "10m"
        lap_each_segment = true
        segments = [
          '30s@1.2',
          '30s@.55',
        ]
        repeat = 10
        "#;

        let foo: ShadowIntervalTemplate = toml::from_str(it_str).unwrap();
        assert!(foo.validate().is_ok());
        assert_eq!(foo.name, String::from("30on/30off"));
        assert_eq!(foo.description, String::from("30s on @ 1.2, 30s off @ .55"));
        assert_eq!(foo.duration, Duration::from_str("10m").unwrap());
        assert!(foo.lap_each_segment);
        assert_eq!(foo.segments.len(), 2);
        assert_eq!(foo.repeat, Some(Quantity(10)));
    }

    #[test]
    fn test_shadow_interval_template_validate() {
        let it_str = r#"
        name = "Warmup"
        description = "Warming up the legs"
        duration = "10m"
        lap_each_segment = false
        segments = [
          '5m@100',
          '1m@110',
          '1m@120',
          '1m@130',
          '2m@100',
        ]"#;

        let foo: ShadowIntervalTemplate = toml::from_str(it_str).unwrap();
        assert!(foo.validate().is_ok());
    }
    #[test]
    fn test_segment() {
        // Test our use cases for writing out a segment.
        // Whitespace should be ignored.
        let seg_str = r#"
        segments = [
          '2m@.85',
          '3m@150',
          '2m30s @ .85',
          '3m4s @ 150',
          " 1h6m30s@ 150 ",
          { duration = '3m50s', power_start = 200, power_end = 250 },
          { duration = '30s', power_start = 0.55, power_end = 0.85 },
        ]"#;

        #[derive(Deserialize)]
        struct Foo {
            segments: Vec<Segment>,
        }

        let foo: Foo = toml::from_str(seg_str).unwrap();

        assert_eq!(foo.segments.len(), 7);
        assert_eq!(foo.segments[0].duration, Duration(120));
        assert_eq!(foo.segments[0].power_start, PowerTarget::Percentage(0.85));
        assert_eq!(foo.segments[3].power_end, PowerTarget::Watts(150));
        assert_eq!(foo.segments[4].duration, Duration(3600 + 6 * 60 + 30));
        assert_eq!(foo.segments[5].power_start, PowerTarget::Watts(200));
        assert_eq!(foo.segments[6].power_end, PowerTarget::Percentage(0.85));
    }
}
