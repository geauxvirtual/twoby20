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
use serde_derive::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
enum PowerTarget {
    Watts(u16),
    Percentage(f32),
}

struct Segment {
    duration: u32,
    power_start: PowerTarget,
    power_end: PowerTarget,
    start_time: u32, //Start time in seconds.
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
                duration: String,
                power_start: PowerTarget,
                power_end: PowerTarget,
            },
        }

        let seg = SegType::deserialize(deserializer)?;
        match seg {
            SegType::A(value) => {
                // Let's split our string and remove whitespaces.
                let value_vec: Vec<_> = value
                    .split("@")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|x| x.trim())
                    .collect();
                // if value_vec.len() != 2 { return error }
                // TODO: May need to look into support a partial segment
                let duration_str = value_vec[0];
                let mut sdi = 0;
                let mut dur_secs = 0;
                for (i, c) in duration_str.chars().enumerate() {
                    if c.is_digit(10) {
                        continue;
                    };
                    let multiplier = match c {
                        'h' => 3600,
                        'm' => 60,
                        's' => 1,
                        _ => panic!("invalid character"),
                    };
                    dur_secs += duration_str[sdi..i].parse::<u32>().unwrap() * multiplier;
                    sdi = i + 1;
                }
                let power_target_str = value_vec[1];
                let power_target = match power_target_str.parse::<u16>() {
                    Ok(v) => PowerTarget::Watts(v),
                    Err(_) => match power_target_str.parse::<f32>() {
                        Ok(v) => PowerTarget::Percentage(v),
                        Err(_) => panic!("Invalid value"), //return error here
                    },
                };
                Ok(Segment {
                    duration: dur_secs,
                    power_start: power_target.clone(),
                    power_end: power_target,
                    start_time: 0,
                })
            }
            SegType::B {
                duration,
                power_start,
                power_end,
            } => Ok(Segment {
                duration: 0,
                power_start,
                power_end,
                start_time: 0,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_segment() {
        // Test our use cases for writing out a segment.
        // Whitespace should be ignored.
        let seg_str = r#"
        segments = [
          '2m@.85',
          '3m@150',
          '2m30s @ .85',
          '3m45s @ 150',
          " 1h6m30s@ 150 ",
        ]"#;

        #[derive(Deserialize)]
        struct Foo {
            segments: Vec<Segment>,
        }

        let foo: Foo = toml::from_str(seg_str).unwrap();

        assert_eq!(foo.segments.len(), 5);
        assert_eq!(foo.segments[0].duration, 120);
        assert_eq!(foo.segments[0].power_start, PowerTarget::Percentage(0.85));
        assert_eq!(foo.segments[3].power_end, PowerTarget::Watts(150));
        assert_eq!(foo.segments[4].duration, 1 * 3600 + 6 * 60 + 30);
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
