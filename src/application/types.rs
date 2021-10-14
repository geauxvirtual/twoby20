// A collection of types used throughout the application. Allows cleaning
// up code and keeping all types creating in one module along with tests.
use derive_more::Add;
use serde_derive::Deserialize;
use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

// PowerTarget: Enum representing power for a segment either by an integer (u16)
// for watts or a float (f32) for a percentage of the user's FTP
#[derive(Copy, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum PowerTarget {
    Watts(u16),
    Percentage(f32),
}

impl FromStr for PowerTarget {
    // TODO Implement proper errors
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // First check to see if the &str is a u16 as a u16 can be
        // converted to a f32 but not vice versa.
        match value.parse::<u16>() {
            Ok(v) => Ok(v.into()),
            Err(_) => match value.parse::<f32>() {
                Ok(v) => {
                    if v.is_sign_negative() {
                        return Err("positive number integer for Watts or decimcal number for Percentage of FTP")
                    }
                    Ok(v.into())
                }
                Err(_) => Err("positive integer greater than 0 (i.e. 200) for Watts or decimal number (0.85) for Percentage of FTP"),
            }
        }
    }
}

// This allows calling into() on a u16 and getting a PowerTarget::Watts
impl From<u16> for PowerTarget {
    fn from(value: u16) -> Self {
        Self::Watts(value)
    }
}

// This allows calling into() on a f32 and getting a PowerTarget::Percentage
impl From<f32> for PowerTarget {
    fn from(value: f32) -> Self {
        Self::Percentage(value)
    }
}

// Duration is a u32 holding the number of seconds for the duration of a Workout,
// Interval, or Segment.
// TODO Have a display feature that will break down the number of seconds into HH::MM::SS
#[derive(Copy, Clone, Debug, Add, PartialEq, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Duration(pub u32);

impl FromStr for Duration {
    // TODO Add proper error messages
    type Err = &'static str;

    // Converts a string into a duration for the following formats
    // 1h
    // 1m
    // 1s
    // 1h10m
    // 10m30s
    // 1h10m30s
    // 1h30s
    // Output is a Duration with the number of seconds as a u32.
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
                _ => return Err("duration must be specified with a h, m, or s character"),
            };
            duration += s[sdi..i]
                .parse::<u32>()
                .map_err(|_| "requires valid unsigned integer")?
                * multiplier;
            sdi = i + 1;
        }
        Ok(duration)
    }
}

impl TryFrom<String> for Duration {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<u32> for Duration {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

// let x = Duration(0) + 5;
impl Add<u32> for Duration {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Self(self.0 + rhs)
    }
}

// let x = Duration(30) * Quantity(5);
impl Mul<Quantity> for Duration {
    type Output = Self;

    fn mul(self, rhs: Quantity) -> Self {
        Self(self.0 * rhs.0)
    }
}

// let x = Duration(30) * 5;
/*impl Mul<u32> for Duration {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        Self(self.0 * rhs)
    }
}*/

// let mut x = Duration(0);
// x += 30;
impl AddAssign<u32> for Duration {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

// let mut x = Duration(0);
// x += Duration(60);
impl AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs.0
    }
}

// StartTime is the start time for a segment. First segment is always 0 seconds.
// StartTime for each subsequent segment is duration of previous segment plus StartTime
// of previous segment
#[derive(Add, Copy, Clone, Debug, PartialEq)]
pub struct StartTime(pub u32);

impl From<u32> for StartTime {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

// let x = StartTime(0) + Duration(30);
impl Add<Duration> for StartTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self(self.0 + rhs.0)
    }
}

// let mut x = StartTime(0);
// x += Duration(60);
impl AddAssign<Duration> for StartTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs.0;
    }
}

// let mut x = StartTime(0);
// x += StartTime(60);
impl AddAssign<StartTime> for StartTime {
    fn add_assign(&mut self, rhs: StartTime) {
        self.0 += rhs.0;
    }
}

// A Quantity value that can be used for instance to provide
// the number of times an interval should be repeated.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
pub struct Quantity(pub u32);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_powertarget_watts_from_str() {
        let pt: PowerTarget = "200".parse().unwrap();
        assert_eq!(pt, PowerTarget::Watts(200));
    }

    #[test]
    fn test_powertarget_watts_from_u16() {
        let pt: PowerTarget = 200.into();
        assert_eq!(pt, PowerTarget::Watts(200));
    }

    #[test]
    fn test_powertarget_percetage_from_str() {
        let pt: PowerTarget = "0.85".parse().unwrap();
        assert_eq!(pt, PowerTarget::Percentage(0.85));
    }

    #[test]
    fn test_powertarget_percentage_from_f32() {
        let pt: PowerTarget = 0.85.into();
        assert_eq!(pt, PowerTarget::Percentage(0.85));
    }

    #[test]
    fn test_powertarget_negative() {
        assert!("-200".parse::<PowerTarget>().is_err());
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
        let d: Duration = "2h".parse().unwrap();
        assert_eq!(d, Duration(2 * 60 * 60));
    }

    #[test]
    fn test_duration_minutes_seconds() {
        let d: Duration = "30m15s".parse().unwrap();
        assert_eq!(d, Duration(30 * 60 + 15));
    }

    #[test]
    fn test_duration_hours_minutes() {
        let d: Duration = "2h30m".parse().unwrap();
        assert_eq!(d, Duration(2 * 60 * 60 + 30 * 60));
    }

    #[test]
    fn test_duration_hours_seconds() {
        let d: Duration = "2h30s".parse().unwrap();
        assert_eq!(d, Duration(2 * 60 * 60 + 30));
    }

    #[test]
    fn test_duration_error() {
        assert!("2h46d".parse::<Duration>().is_err());
    }

    #[test]
    fn test_duration_from() {
        let d1 = Duration::from(0);
        assert_eq!(d1, Duration(0));
        let d2: Duration = 0.into();
        assert_eq!(d2, Duration(0));
    }

    #[test]
    fn test_duration_add_u32() {
        assert_eq!(Duration(0) + 5, Duration(5));
    }

    #[test]
    fn test_duration_add_duration() {
        assert_eq!(Duration(5) + Duration(5), Duration(10));
    }

    #[test]
    fn test_duration_addassign_u32() {
        let mut x = Duration(0);
        x += 5;
        assert_eq!(x, Duration(5));
    }

    #[test]
    fn test_duration_addassign_duration() {
        let mut x = Duration(0);
        x += Duration(10);
        assert_eq!(x, Duration(10));
    }

    #[test]
    fn test_starttime_from_u32() {
        let x: StartTime = 0.into();
        assert_eq!(x, StartTime(0));
    }

    #[test]
    fn test_starttime_add() {
        assert_eq!(StartTime(0) + StartTime(60), StartTime(60));
    }

    #[test]
    fn test_starttime_add_duration() {
        assert_eq!(StartTime(0) + Duration(30), StartTime(30));
    }

    #[test]
    fn test_starttime_addassign_duration() {
        let mut x = StartTime(0);
        x += Duration(300);
        assert_eq!(x, StartTime(300));
    }

    #[test]
    fn test_starttime_addassign_starttime() {
        let mut x = StartTime(0);
        x += StartTime(60);
        assert_eq!(x, StartTime(60));
    }

    #[test]
    fn test_duration_times_quantity() {
        assert_eq!(Duration(30) * Quantity(3), Duration(30 * 3));
    }
}
