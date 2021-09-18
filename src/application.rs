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

mod menubar;
use menubar::MenuBar;
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
        flags: flags,
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

// Main application structure for handling state changes and views of the
// application.
struct Application {
    state: AppState,
    should_exit: bool,
    ant_request_tx: libant::Sender<Request>,
    user_profiles: Vec<UserProfile>,
    workouts: Vec<Workout>,
    menubar: MenuBar,
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
    ShowUserProfiles,
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
                should_exit: false,
                ant_request_tx: flags
                    .ant_request_tx
                    .expect("Error 001: Application misconfigured"),
                user_profiles: vec![],
                workouts: vec![], //There will be a single default workout always loaded. For now just created an empty vec.
                menubar: MenuBar::default(),
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
                    if let Some(workouts) = state.workouts {
                        self.workouts.extend_from_slice(&workouts);
                    }
                    if let Some(user_profiles) = state.user_profiles {
                        self.user_profiles.extend_from_slice(&user_profiles);
                    }
                    self.state = AppState::Ready;
                }
                Message::Loaded(Err(_)) => {}
                _ => {}
            },
            AppState::Ready => {
                match message {
                    Message::Tick(_) => {} //do nothing for now
                    Message::EventOccurred(event) => {
                        // May want to look into how to filter events before getting to this update
                        if let Event::Window(window::Event::CloseRequested) = event {
                            log::info!("Exiting application");
                            // Send quit request to ANT+ run thread
                            self.ant_request_tx.send(Request::Quit).unwrap();
                            thread::sleep(Duration::from_millis(500));
                            self.should_exit = true;
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
                if self.user_profiles.is_empty() {}
                let content = Column::new()
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .push(self.menubar.view());
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
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
struct UserProfile;
#[derive(Debug, Clone)]
struct Workout;

#[derive(Debug, Clone)]
pub struct SavedState {
    user_profiles: Option<Vec<UserProfile>>,
    workouts: Option<Vec<Workout>>,
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
        Ok(SavedState {
            user_profiles: None,
            workouts: None,
        })
    }

    // Save will be able to save user profiles. Single toml file.
    // fn save()

    // save_activity will save .fit file to file system
    // fn save_activity()
}
