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
//   { name = '30 on/30 off', repeat = '30' },
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
//   { name = '30 on/30 off', segments = { 0 = '@1.5'}, repeat = '30' },
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

use derive_more::Add;
use serde::de::Unexpected;
use serde_derive::Deserialize;

use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

#[derive(Copy, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
enum PowerTarget {
    Watts(u16),
    Percentage(f32),
}

impl FromStr for PowerTarget {
    type Err = &'static str;
    fn from_str(power_target_str: &str) -> Result<PowerTarget, Self::Err> {
        match power_target_str.parse::<u16>() {
            Ok(v) => Ok(v.into()),
            Err(_) => match power_target_str.parse::<f32>() {
                Ok(v) => {
                    if v.is_sign_negative() {
                        return Err("positive number integer for Watts or floating point number for Percentage")
                    }
                    Ok(v.into())
                }
                Err(_) => return Err("integer greater than 0 (i.e. 200) for Watts or floating point number (0.85) for Percentage"),
            },
        }
    }
}

impl From<u16> for PowerTarget {
    fn from(value: u16) -> Self {
        Self::Watts(value)
    }
}

impl From<f32> for PowerTarget {
    fn from(value: f32) -> Self {
        Self::Percentage(value)
    }
}

#[derive(Copy, Clone, Debug, Add, PartialEq, Deserialize)]
#[serde(try_from = "String", into = "String")]
struct Duration(u32);

impl From<u32> for Duration {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Add<u32> for Duration {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Self(self.0 + rhs)
    }
}

impl Mul<u32> for Duration {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Quantity> for Duration {
    type Output = Self;
    fn mul(self, rhs: Quantity) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl AddAssign<u32> for Duration {
    fn add_assign(&mut self, rhs: u32) {
        self.0 = self.0 + rhs
    }
}

impl AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 = self.0 + rhs.0
    }
}

impl FromStr for Duration {
    type Err = &'static str;

    // A method that takes a string in the format the following formats:
    // 1h
    // 1m
    // 1s
    // 1h10m
    // 10m30s
    // 1h10m30s
    // and outputs the number of seconds as u32. This returns a Result with
    // either the u32 equaling the number of seconds descibed by the string
    // or a serde::de::Unexpected enum with the corresponding error message
    // for serde's unexpected/expected Error return.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sdi = 0;
        let mut duration = Duration(0);
        for (i, c) in s.chars().enumerate() {
            if c.is_digit(10) {
                continue;
            };
            let multiplier = match c {
                'h' => 3600,
                'm' => 60,
                's' => 1,
                _ => return Err("required matching characters h, m, or s"),
            };
            duration += s[sdi..i]
                .parse::<u32>()
                .map_err(|_| "required valid u32 integer")?
                * multiplier;
            sdi = i + 1;
        }
        Ok(duration)
    }
}

use std::convert::TryFrom;
impl TryFrom<String> for Duration {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct StartTime(u32);

impl From<u32> for StartTime {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Add<Duration> for StartTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<Duration> for StartTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 = self.0 + rhs.0
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
struct Quantity(u32);

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
                    .split("@")
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
                (duration, power_target.clone(), power_target)
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
    name: String,
    description: String,
    duration: Duration,
    segments: Vec<Segment>,
    lap_each_segment: bool,
    repeat: Option<Quantity>,
}

//TODO implement proper errors
impl IntervalTemplate {
    fn validate(&self) -> Result<(), &'static str> {
        let seg_duration = self
            .segments
            .iter()
            .fold(Duration::from(0), |acc, x| acc + x.duration.clone());
        if self.duration != seg_duration * self.repeat.clone().unwrap_or(Quantity(1)) {
            return Err("duration does not match duration calculated from segments");
        }
        Ok(())
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
        }

        match IntervalType::deserialize(deserializer)? {
            IntervalType::A(value) => {
                let v: Vec<&str> = value
                    .split("@")
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
                    name: String::from("this should be optional"),
                    description: String::from("this should be optional"),
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
                let interval_duration = duration.clone() * repeat.clone().unwrap_or(Quantity(1));
                let segment = Segment {
                    duration: duration.clone(),
                    power_start,
                    power_end,
                    start_time: 0.into(),
                };
                // Need to validate IntervalTemplates that are created from segments
                // passed into the WorkoutTemplate
                Ok(Self::IntervalTemplate(IntervalTemplate {
                    name: String::from("this should be optional"),
                    description: String::from("this should be optional"),
                    duration: interval_duration,
                    lap_each_segment: lap_each_segment.unwrap_or(false),
                    segments: vec![segment],
                    repeat,
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
                    if let Some(template) = interval_templates.get(value) {
                        duration += template.duration.clone();
                        *interval_type = IntervalTemplateType::IntervalTemplate(template.clone())
                    }
                }
                IntervalTemplateType::IntervalTemplate(template) => {
                    duration += template.duration.clone()
                }
            }
        }
        if self.duration != duration {
            return Err("defined duration does not match sum of interval durations");
        }
        Ok(())
    }

    fn build_workout(self) -> WorkoutTemplate {
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
    intervals: Option<Vec<IntervalTemplate>>,
    workouts: Option<Vec<ShadowWorkoutTemplate>>,
}

#[cfg(test)]
mod test {
    use super::*;

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
                            library.intervals.insert(interval.name.clone(), interval);
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
                    let workout = shadow_workout.build_workout();
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
    fn test_interval_template() {
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

        let foo: IntervalTemplate = toml::from_str(it_str).unwrap();
        assert_eq!(foo.name, String::from("Warmup"));
        assert_eq!(foo.description, String::from("Warming up the legs"));
        assert_eq!(foo.duration, Duration::from_str("10m").unwrap());
        assert_eq!(foo.lap_each_segment, false);
        assert_eq!(foo.segments.len(), 5);
    }

    #[test]
    fn test_interval_template_with_repeat() {
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

        let foo: IntervalTemplate = toml::from_str(it_str).unwrap();
        assert!(foo.validate().is_ok());
        assert_eq!(foo.name, String::from("30on/30off"));
        assert_eq!(foo.description, String::from("30s on @ 1.2, 30s off @ .55"));
        assert_eq!(foo.duration, Duration::from_str("10m").unwrap());
        assert_eq!(foo.lap_each_segment, true);
        assert_eq!(foo.segments.len(), 2);
        assert_eq!(foo.repeat, Some(Quantity(10)));
    }

    #[test]
    fn test_interval_template_validate() {
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

        let foo: IntervalTemplate = toml::from_str(it_str).unwrap();
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
        assert_eq!(foo.segments[4].duration, Duration(1 * 3600 + 6 * 60 + 30));
        assert_eq!(foo.segments[5].power_start, PowerTarget::Watts(200));
        assert_eq!(foo.segments[6].power_end, PowerTarget::Percentage(0.85));
    }

    #[test]
    fn test_duration_seconds() {
        let d: Duration = "30s".parse().unwrap();
        assert_eq!(d, Duration(30));
    }
    #[test]
    fn test_duration_minutes() {
        let d: Duration = "30m".parse().unwrap();
        assert_eq!(d, Duration(30 * 60));
    }
    #[test]
    fn test_duration_hours() {
        let d: Duration = "1h".parse().unwrap();
        assert_eq!(d, Duration(1 * 3600));
    }
    #[test]
    fn test_duration_minutes_seconds() {
        let d: Duration = "1m30s".parse().unwrap();
        assert_eq!(d, Duration(1 * 60 + 30));
    }
    #[test]
    fn test_duration_hours_minutes() {
        let d: Duration = "1h30m".parse().unwrap();
        assert_eq!(d, Duration(1 * 3600 + 30 * 60));
    }
    #[test]
    fn test_duration_hours_seconds() {
        let d: Duration = "2h30s".parse().unwrap();
        assert_eq!(d, Duration(2 * 3600 + 30));
    }
    #[test]
    fn test_duration_hours_minutes_seconds() {
        let d: Duration = "2h46m30s".parse().unwrap();
        assert_eq!(d, Duration(2 * 3600 + 46 * 60 + 30));
    }
    #[test]
    fn test_duration_time_character_error() {
        assert!("2h46m30d".parse::<Duration>().is_err());
    }

    #[test]
    fn test_powertarget_watts() {
        let pt: PowerTarget = "200".parse().unwrap();
        assert_eq!(pt, PowerTarget::Watts(200));
    }

    #[test]
    fn test_powertarget_percentage() {
        let pt: PowerTarget = "1.2".parse().unwrap();
        assert_eq!(pt, PowerTarget::Percentage(1.2));
    }

    #[test]
    fn test_powertarget_negative() {
        assert!("-200".parse::<PowerTarget>().is_err());
    }

    #[test]
    fn test_duration_from() {
        let d1 = Duration::from(0);
        assert_eq!(d1, Duration(0));
        let d2: Duration = 0.into();
        assert_eq!(d2, Duration(0));
    }

    #[test]
    fn test_add_duration() {
        let d1: Duration = 5.into();
        let d2: Duration = 10.into();
        assert_eq!(d1 + d2, Duration(15));
    }

    #[test]
    fn test_mul_duration() {
        let d1: Duration = 5.into();
        assert_eq!(d1 * 5, Duration(25));
    }

    #[test]
    fn test_add_assign_duration() {
        let mut d1: Duration = 10.into();
        d1 += 30;
        assert_eq!(d1, Duration(40));
    }

    #[test]
    fn test_add_duration_to_starttime() {
        let s1: StartTime = 0.into();
        let d1: Duration = 10.into();
        assert_eq!(s1 + d1, StartTime(10));
    }

    #[test]
    fn test_add_assign_duration_to_starttime() {
        let mut s1: StartTime = 0.into();
        let d1: Duration = 10.into();
        s1 += d1;
        assert_eq!(s1, StartTime(10));
    }
}

//struct SegmentTemplate {
//    duration: std::time::Duration, //Could this be u64? Convert whatever is passed in to seconds?
//    power_target: PowerTarget,
//}

//enum PowerTarget {
//    Power(PowerTargetType)
//    PowerRange { start: PowerTargetType, end: PowerTargetType }
//}

//enum PowerTargetType {
//    Percentage(f32),
//    Watts(u16)
//}

//struct IntervalTemplate {
//    name: String,
//    description: String,
//    duration: u64, // Validate all segments associated with template match duration
//    segments: Vec<SegmentTemplate>,
//    lap_each_segment: bool,
//}
//
// struct Interval {
//   name: Option<String>, // Intervals can be created from workout template but not saved
//   description: Option<String>,
//   duration: u64, // Can always be overridden
//   segments: Vec<Segment>,
//   start_time: u64, // default to 0 but can be overridden
//   lap_each_segment: bool
//   repeat: Option<u16>,
// }
//
//
// struct WorkoutTemplate {
//   name: String,
//   description: String,
//   duration: u64,
//   intervals: Vec<IntervalTemplate>
//   lap_each_interval: bool,
// }
//
// struct Workout {
//   name: String,
//   description: String,
//   duration: u64,
//   intervals: Vec<Interval>
//   lap_each_interval: bool
// }
