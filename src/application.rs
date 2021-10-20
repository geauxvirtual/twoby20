// Application workflow consists of reading data from an external source
// (ANT+ stick and/or Bluetooth), process the messages, and then display
// the data to the user. While the frontend is async based, the backend
// reading from USB is still synchronous. Multiple threads are used
// with data being sent between threads via channels.
use std::{
    thread,
    time::{Duration, Instant},
};

use iced::{
    executor, time, Application as IcedApplication, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Settings, Subscription, Text,
};
use iced_native::{subscription, window, Event};
use libant::Request;
use log::{error, info};

mod menubar;
mod types;
mod user_profile;
mod workout;
use menubar::MenuBar;
use user_profile::{UserProfile, UserProfileMessage, UserProfileState};
use workout::{Library, ShadowLibrary};

// Run() is the main function to call. This handles starting up all the
// threads and configuring the channels.
pub fn run() {
    // Used for sending messages to ANT+ devices. (Open channel, Close channel,
    // request data, etc.
    let (ant_request_tx, ant_request_rx) = libant::unbounded();
    // Used for receiving ANT+ broadcast and channel messages
    let (ant_message_tx, _ant_message_rx) = libant::unbounded();
    // Usend for sending messages to the application frontend
    //    let (_app_tx, _app_rx) = libant::unbounded();

    let ant_run_handle = thread::spawn(move || libant::ant::run(ant_request_rx, ant_message_tx));

    let flags = AppFlags {
        ant_request_tx: Some(ant_request_tx),
    };

    let window_settings = iced::window::Settings {
        min_size: Some((1280, 768)),
        ..Default::default()
    };
    Application::run(Settings {
        flags,
        window: window_settings,
        exit_on_close_request: false,
        ..Settings::default()
    })
    .unwrap();
    // From my testing, this never gets executed beyond this point as
    // Iced memory drops the interface.
    ant_run_handle.join().unwrap();
}

// AppState
// Starting
//   - Load user profiles
//   - Load workouts
//   - Load history (future)
// Ready
//   - Select user profile
//   - Update user profile
//   - List available ANT+ devices (Bluetooth future maybe...)
//   - List and select workout
//   - Start activity
// ActivityInProgress
//   - Record activity with data from ANT+ devices
//   - Graph device data on top of workout display
// ActivityCompleted
//   - Save activity in history
//   - Auto export of .fit file to filesystem
//   - (future) Upload activity to Strava
//   - Go back to Ready state
// ActivityEnded
//   - Prompt user to save or discard workout
//   - Go back to Ready state
enum AppState {
    Starting,
    Ready,
    //    ActivityInProgress
    //    ActivityCompleted
    //    ActivityEnded
}

enum ScreenState {
    UserProfile,
    Workouts,
    Devices,
}
// Main application structure for handling state changes and views of the
// application.
// TODO: (Future) Add history tracking of any selected workout from workouts
// screen so user can be brought back to selected workout if they click
// on another screen button.
// (9/23) UserProfiles are created with a default "new" profile as profile 0.
// This profile is never removed. It's there so there is a "New..." or "Create.."
// field in the pick list (if pick list is maintained). First user profile will
// always be 1.
// (10/6) Workouts should be changed to Library which is a collection of intervals
// that can be used to create workouts and a collection of available workouts.
// (Future) Possibly add a way to select intervals in the UI and create a workout from
// the selection.
struct Application {
    state: AppState,
    screen_state: ScreenState,
    should_exit: bool,
    ant_request_tx: libant::Sender<Request>,
    active_user_profile: usize,
    user_profiles: Vec<UserProfile>,
    library: Library,
    workouts: Vec<Workout>,
    menubar: MenuBar,
    user_profile_screen: UserProfileState,
}

// Message enum for configuring subscriptions and updates in the application.
// Tick will be used with a subscription to trigger refreshes of the
// application view on a set.
// EventOccurred watches for different events (could be mouse, key, window, etc)
// and acts on the events accordingly.
#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, LoadError>),
    Tick(Instant),
    EventOccurred(Event),
    ShowWorkouts,
    ShowDevices,
    ShowUserProfile,
    UserProfileMessage(usize, UserProfileMessage),
    UserProfileSelected(UserProfile),
}

// AppFlags are used to pass channels into the application for communication
// between the GUI and the backend threads that receive and send data
// to ANT+ devices.
struct AppFlags {
    ant_request_tx: Option<libant::Sender<Request>>,
}

impl Default for AppFlags {
    fn default() -> Self {
        Self {
            ant_request_tx: None,
        }
    }
}

impl IcedApplication for Application {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Application {
                state: AppState::Starting,
                screen_state: ScreenState::Workouts,
                should_exit: false,
                ant_request_tx: flags
                    .ant_request_tx
                    .expect("Error 001: Application misconfigured"),
                user_profiles: vec![UserProfile::new(true)],
                active_user_profile: 0,
                library: Library::default(),
                workouts: vec![],
                menubar: MenuBar::default(),
                user_profile_screen: UserProfileState::default(),
            },
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("2by20")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        // This will need to be updated to handle state of the application with
        // what happens more than likely, but that will get built out as the
        // application evolves.
        match self.state {
            AppState::Starting => match message {
                Message::Loaded(Ok(state)) => {
                    // Load the library of workouts/intervals that were retrieved
                    // from local files.
                    if let Some(intervals) = state.shadow_library.intervals {
                        for interval in intervals {
                            if let Err(_e) = interval.validate() {
                                // log error
                                continue;
                            }
                            let name = interval.name.clone().unwrap();
                            match self.library.intervals.contains_key(&name) {
                                true => continue, //log error for duplicate key
                                false => self.library.intervals.insert(name, interval),
                            };
                        }
                    }

                    if let Some(workouts) = state.shadow_library.workouts {
                        for mut shadow_workout in workouts {
                            if let Err(_e) = shadow_workout.validate(&self.library.intervals) {
                                // log error
                                continue;
                            }
                            let workout = shadow_workout.build_workout_template();
                            self.library.workouts.insert(workout.name.clone(), workout);
                        }
                    }

                    if let Some(workouts) = state.workouts {
                        self.workouts.extend_from_slice(&workouts);
                    }
                    if let Some(user_profiles) = state.user_profiles {
                        self.user_profiles.extend_from_slice(&user_profiles);
                        for (i, profile) in self.user_profiles.iter_mut().enumerate() {
                            if profile.active() {
                                if self.active_user_profile != 0 {
                                    error!("Multiple user profiles set as active. Leaving first profile set as active");
                                    profile.set_active(false);
                                }
                                self.active_user_profile = i;
                            }
                        }
                        // We loaded profiles from saved state, but no profiles
                        // were set to active. Set first profile loaded to active
                        if self.active_user_profile == 0 {
                            self.active_user_profile = 1;
                            self.user_profiles[1].set_active(true);
                        }
                    }
                    // If no user profiles were loaded, set screen state to
                    // UserProfile so a user profile can be created
                    if self.active_user_profile == 0 {
                        info!("Setting screen_state to ScreenState::UserProfile");
                        self.screen_state = ScreenState::UserProfile;
                    }
                    self.state = AppState::Ready;
                }
                Message::Loaded(Err(_)) => {}
                _ => {}
            },
            AppState::Ready => {
                match message {
                    Message::Tick(_) => {} //do nothing for now
                    Message::EventOccurred(Event::Window(window::Event::CloseRequested)) => {
                        // May want to look into how to filter events before getting to this update
                        //if let Event::Window(window::Event::CloseRequested) = event {
                        log::info!("Exiting application");
                        // Send quit request to ANT+ run thread
                        self.ant_request_tx.send(Request::Quit).unwrap();
                        thread::sleep(Duration::from_millis(500));
                        self.should_exit = true;
                        //}
                    }
                    Message::ShowUserProfile => self.screen_state = ScreenState::UserProfile,
                    Message::ShowWorkouts => self.screen_state = ScreenState::Workouts,
                    Message::ShowDevices => self.screen_state = ScreenState::Devices,
                    Message::UserProfileMessage(i, UserProfileMessage::SaveProfile(name, ftp)) => {
                        // Check to see if we are creating or updating a profile.
                        // i will be 0 when creating a profile.
                        // TODO Validate name isn't empty. Return an error back
                        // to UserProfile screen to display instead. Or disable
                        // Save button unless input field has valid data.
                        if i == 0 {
                            // Create a new profile
                            info!("Creating user profile {}", self.user_profiles.len());
                            let mut profile = UserProfile::new(true);
                            profile.set_name(&name);
                            profile.set_ftp(ftp);
                            self.user_profiles.push(profile);
                            self.active_user_profile = self.user_profiles.len() - 1;
                        } else {
                            info!("Saving user profile {}", i);
                            if let Some(profile) = self.user_profiles.get_mut(i) {
                                profile.set_name(&name);
                                profile.set_ftp(ftp);
                            }
                        }
                        self.user_profile_screen
                            .update(UserProfileMessage::Editing(false));
                    }
                    Message::UserProfileMessage(i, UserProfileMessage::DeleteProfile) => {
                        // We delete the requested profile. If this leaves no profiles,
                        // then we create a new profile. The application requires
                        // a profile in order to function
                        // TODO: Decide if we should show a list of available profiles
                        // and allow for actions to occur outside of current
                        // active profile
                        // TODO: Disable delete button when creating a profile. For now
                        // just add a check to remove later.
                        if i == 0 {
                            error!("Trying to delete default profile");
                            self.user_profile_screen.update(UserProfileMessage::Clear);
                            return Command::none();
                        }
                        info!("Removing user profile {}", i);
                        // We always set the active profile to the first available
                        // profile after deletion. We are always deleting the current
                        // active profile (in current iteration)
                        self.user_profiles.remove(i);
                        if self.user_profiles.len() == 1 {
                            // All user profiles have been deleted
                            self.active_user_profile = 0;
                        } else {
                            self.active_user_profile = 1;
                        }
                        self.user_profile_screen.update(UserProfileMessage::Clear);
                    }
                    Message::UserProfileMessage(_, user_profile_message) => {
                        self.user_profile_screen.update(user_profile_message)
                    }
                    Message::UserProfileSelected(profile) => {
                        // May be a better way to do this to get index of profile
                        // sent with profile because we only care about the index.
                        // Find user profile. Set active_user_profile to selected profile.
                        for (i, p) in self.user_profiles.iter().enumerate() {
                            if profile == *p {
                                self.user_profile_screen.update(UserProfileMessage::Clear);
                                self.active_user_profile = i;
                                // If profile is default profile, change screen state
                                // for a user to be created.
                                if i == 0 {
                                    self.screen_state = ScreenState::UserProfile;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            time::every(Duration::from_millis(125)).map(Message::Tick),
            subscription::events().map(Message::EventOccurred),
        ])
    }

    fn view(&mut self) -> Element<Message> {
        match self.state {
            AppState::Starting => initializing_message(),
            AppState::Ready => {
                // If we don't have any user profiles, load screen to
                // create a user profile. If we have user profiles,
                // select the default user profile and load the workouts
                // page for user to select a workout.

                // TODO: Use the screen state enum to generate main
                // screen page from either workouts, devices, or user profile.
                // If no user profiles are loaded from a saved state, create
                // a new user profile and show the user the user profile page
                // so they can edit the fields and save the data.
                // Each page returns a Container::new() that will be sized to
                // fill all the available space.
                //
                let main_screen = match self.screen_state {
                    ScreenState::UserProfile => {
                        // By default, we create a dummy user account if none
                        // are found in the SavedState. When loading from SavedState,
                        // at least one profile should be set as active. We'll
                        // unwrap this value to throw a panic to show we missed
                        // something in our code.
                        //let active_user_profile = self
                        //    .active_user_profile
                        //    .expect("Active user profile should not be set to none at this point");
                        let active_user_profile = self.active_user_profile;
                        self.user_profile_screen
                            .view(&self.user_profiles[active_user_profile])
                            .map(move |message| {
                                Message::UserProfileMessage(active_user_profile, message)
                            })
                        //self.user_profiles[active_user_profile]
                        //    .view()
                        //    .map(move |message| {
                        //        Message::UserProfileMessage(active_user_profile, message)
                        //    })
                    }
                    _ => Container::new(
                        Column::new().push(Text::new("This shouldn't be seen yet").size(40)),
                    )
                    .into(),
                };

                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .push(
                        self.menubar
                            .view(&self.user_profiles, self.active_user_profile),
                    )
                    .push(main_screen)
                    .into()
            }
        }
    }
}

fn initializing_message<'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("2by20 is initializing...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

// Persistence
// On startup, application will check for required directories
// $HOME_DIR/2by20/{workouts, profiles, activities}
// If the directories are not there, they will be created. If the directories
// are there, workouts and profiles will be loaded.
// (Future) Acitivity history will probably be stored in an internal DB to
// easily load and save. Profiles may or may not move to the internal DB as well
// but for now they will just be a toml file.
//
// Example
// [[profile]]
// name = "Justina
// weight = "140"
// weight_unit = "lbs""
// ftp = "285"
// theme = "dark"
// default = true
//
// TODO move these to their own files under application/
#[derive(Debug, Clone)]
struct Workout;

#[derive(Debug, Clone)]
pub struct SavedState {
    user_profiles: Option<Vec<UserProfile>>,
    workouts: Option<Vec<Workout>>,
    shadow_library: ShadowLibrary,
}

// TODO: Implement application error logic. Doing this for now.
#[derive(Debug, Clone)]
pub enum LoadError {
    DirectoryError,
    FileError,
}

impl SavedState {
    // Init should verify and if needed created the following directories
    // $HOME_DIR/Documents/2by20/profiles
    // $HOME_DIR/Documents/2by20/workouts
    // $HOME_DIR/Documents/2by20/activities
    // At some point may also include applications settings.
    // fn init()
    async fn load() -> Result<SavedState, LoadError> {
        // - Call init which will verify the directories exist, and if they
        // don't exist created them.
        // - Load profiles from $HOME_DIR/Documents/2by20/profiles
        // - Load workouts from $HOME_DIR/Documents/2by20/workouts
        let sl = ShadowLibrary::default();
        Ok(SavedState {
            user_profiles: None,
            workouts: None,
            shadow_library: sl,
        })
    }

    // Save will be able to save user profiles. Single toml file.
    // fn save()

    // save_activity will save .fit file to file system
    // fn save_activity()
}
