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
    executor, time, Application as IcedApplication, Clipboard, Column, Command, Element,
    Subscription,
};
use iced_native::{subscription, Event};
// Run() is the main function to call. This handles starting up all the
// threads and configuring the channels.
pub fn run() {
    // Used for sending messages to ANT+ devices. (Open channel, Close channel,
    // request data, etc.
    let (_ant_request_tx, ant_request_rx) = libant::unbounded();
    // Used for receiving ANT+ broadcast and channel messages
    let (ant_message_tx, ant_message_rx) = libant::unbounded();
    // Usend for sending messages to the application frontend
    //    let (_app_tx, _app_rx) = libant::unbounded();

    let ant_run_handle = thread::spawn(move || libant::ant::run(ant_request_rx, ant_message_tx));

    //This is just for test purposes initially. There will be a separate
    //process thread for handling messages and sending the real time data
    //up to the GUI.
    loop {
        match ant_message_rx.recv() {
            Ok(libant::Response::Error(e)) => {
                log::error!("Error message received: {:?}", e);
            }
            Ok(mesg) => log::debug!("Debugging message received: {:?}", mesg),
            Err(e) => {
                log::error!("Error receiving from Ant run thread: {:?}", e);
                break;
            }
        }
    }
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
}

// Message enum for configuring subscriptions and updates in the application.
// Tick will be used with a subscription to trigger refreshes of the
// application view on a set.
// EventOccurred watches for different events (could be mouse, key, window, etc)
// and acts on the events accordingly.
#[derive(Debug)]
enum Message {
    Tick(Instant),
    EventOccurred(Event),
}

// AppFlags are used to pass channels into the application for communication
// between the GUI and the backend threads that receive and send data
// to ANT+ devices.
struct AppFlags {}

impl IcedApplication for Application {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Application {
                state: AppState::Starting,
                should_exit: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("2by20")
    }

    fn update(&mut self, _message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
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
        Column::new().into()
    }
}
