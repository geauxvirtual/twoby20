use iced::{
    button, text_input, Align, Button, Column, Container, Element, Length, Row, Text, TextInput,
};

use log::error;

// UserProfile to allow multiple users of the software. Allows for a user
// to easily have workouts adjusted based on their FTP setting.

// TODO Improve the styling.
// TODO Capture tabs to change focus of input fields
#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    // First three fields will be serialized into a TOML file
    // Name field
    name: String,
    // FTP field
    ftp: u16,
    // Active field for if
    active: bool,
    state: UserProfileState,

    // These fields are used for capturing input data prior to saving.
    // Can also have a previous field if someone doesn't want to change
    // a field. (future)
    name_input: String,
    ftp_input: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserProfileState {
    name_input_field: text_input::State,
    ftp_input_field: text_input::State,
    save_button: button::State,
    delete_button: button::State,
}

#[derive(Debug, Clone)]
pub enum UserProfileMessage {
    NameInputChanged(String),
    FtpInputChanged(String),
    SaveProfile,
    DeleteProfile,
}

impl UserProfile {
    // Create a dummy account that user will update
    pub fn new(active: bool) -> Self {
        Self {
            active: active,
            state: Default::default(),
            ..Default::default()
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn update(&mut self, message: UserProfileMessage) {
        match message {
            UserProfileMessage::NameInputChanged(value) => self.name_input = value,
            UserProfileMessage::FtpInputChanged(value) => self.ftp_input = value,
            UserProfileMessage::SaveProfile => {
                self.name = self.name_input.clone();
                match self.ftp_input.parse::<u16>() {
                    Ok(v) => self.ftp = v,
                    Err(_) => {
                        //TODO Should display error message on screen
                        // Should possibly disabling changing screen state
                        // until valid value is entered
                        error!("Invalid FTP value");
                    }
                }
                self.name_input.clear();
                self.ftp_input.clear();
            }
            // SaveProfile and Delete Profile are handled by the main application
            // update() method
            _ => {}
        }
    }

    pub fn view(&mut self) -> Element<UserProfileMessage> {
        let field_text = |text| Text::new(text).size(16).width(Length::Units(50));
        // If we are not a new profile, then set the inputs to the current profile
        if self.name != "" {
            self.name_input = self.name.clone();
        }
        if self.ftp != 0 {
            self.ftp_input = self.ftp.to_string();
        }
        Container::new(
            Column::new()
                .spacing(10)
                .width(Length::Units(300))
                .height(Length::Shrink)
                .push(Text::new("User Profile").size(30))
                .push(
                    Row::new()
                        .push(field_text("Name:"))
                        .push(
                            TextInput::new(
                                &mut self.state.name_input_field,
                                "John Doe",
                                &self.name_input,
                                UserProfileMessage::NameInputChanged,
                            )
                            .padding(8)
                            .width(Length::Units(100))
                            .size(16),
                        )
                        .spacing(10)
                        .align_items(Align::Center)
                        .width(Length::Units(200)),
                )
                .push(
                    Row::new()
                        .push(field_text("FTP:"))
                        .push(
                            TextInput::new(
                                &mut self.state.ftp_input_field,
                                "200",
                                &self.ftp_input,
                                UserProfileMessage::FtpInputChanged,
                            )
                            .padding(8)
                            .width(Length::Units(100))
                            .size(16),
                        )
                        .spacing(10)
                        .align_items(Align::Center)
                        .width(Length::Units(200)),
                )
                .push(
                    Row::new()
                        .spacing(20)
                        .width(Length::Units(200))
                        .push(
                            Button::new(&mut self.state.save_button, Text::new("Save").size(16))
                                .on_press(UserProfileMessage::SaveProfile)
                                .padding(8),
                        )
                        .push(
                            Button::new(
                                &mut self.state.delete_button,
                                Text::new("Delete").size(16),
                            )
                            .padding(8)
                            .on_press(UserProfileMessage::DeleteProfile),
                        ),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
