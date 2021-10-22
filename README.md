## About
2by20 is an indoor cycling training application that allows for easy creation and recording of workouts.

## Development status
This project is under active development and in a very early stage of development and is currenlty non-functional with respect to the application goals.

The project is intended to work on Windows, Linux, and Mac OS X, but all current development and testing occurs on Mac OS X.

## Workout creation
Three intervals and one workout are built into the application.
```toml
[[ intervals ]]
name = "Warmup"
description = "Warming up the legs"
duration = "10m"
segments = [
  '5m @ 100',
  '1m @ 110',
  '1m @ 120',
  '1m @ 130',
  '2m @ 100'
]

[[ intervals ]]
name = "2x20"
description = "2x20 at Tempo (85%)
duration = "45m"
lap_each_segment = true
segments = [
  '20m @ 0.85',
  '5m @ 0.55',
  '20m @ 0.85'
]

[[ intervals ]]
name = "Cooldown"
description = "Cool on down"
duration = "5m"
segments = [
  '5m @ 0.55'
]

[[ workouts ]]
name = "Metcalfe"
description = "2by20...enough said"
duration = "1h"
lap_each_interval = true
intervals = [
  'Warmup',
  '2x20',
  'Cooldown'
]
```

Intervals and workouts are defined in TOML files with TOML syntax. For intervals, name, duration, and segments are required fields. Optional fields for intervals are description, repeat, and lap_each_segment (defaults to false).

Workouts are required to define name, duration, and intervals. Intervals for a workout can either be the name of an existing interval as shown above, can define a new interval, or can override segment values of an existing interval.

```TOML
[[ workouts ]]
name = "Example"
description = "Workout example"
duration = "1h"
lap_each_interval = true
intervals = [
  'Warmup',
  '10m @ 200', #defining power in watts
  { duration = "30m", power_start = 0.75, power_end = 0.90 }, #defining a segment that's a ramp
  { name = "Cooldown", duration = "10m", segments = ["0:10m@"] }, #redefinig the duration and segment for a named interval
]
```

The format for redefining a segment for a known interval is as follows

`<segment interval>:<time>@<power in watts or percentage of FTP>`

For example in the example workout above, the Cooldown interval redefines segment 0 to be 10 minutes long at the power defined in the existing cooldown interval. The segment could have also been defined as the following to extend the cooldown to 10 minutes at a power of 100w. 

`0:10m@100`

Power in watts is defined as integer value, and power as a percentage of the user profile's FTP is defined as a float.

## TODOs
- [ ] Add styling to the GUI
- [ ] Add support for reading in workouts/intervals from the file system.
- [ ] List all workouts and intervals available in the internal library.
- [ ] Select a workout to record
- [ ] Display a workout that can be overlayed with ANT+ device data.
- [ ] Record a workout based on duration of workout with ANT+ device data.
- [ ] Export a workout in .FIT format that can be imported to other applications (Strava, Golden Cheetah, etc)
- [ ] Maintain a history of workouts that can be viewed with calculated stats.
- [ ] Calculate stats from ANT+ data (averages for workout, averages per lap, etc)
