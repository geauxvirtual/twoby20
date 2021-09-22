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
    // Active field for setting active profile when SavingState. Default will
    // be last active profile.
    active: bool,
    //state: UserProfileState,

    // These fields are used for capturing input data prior to saving.
    // Can also have a previous field if someone doesn't want to change
    // a field. (future)
    //name_input: String,
    //ftp_input: String,
}

impl UserProfile {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            ..Default::default()
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    // Set this up for Into to take a &str or String
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string()
    }

    pub fn set_ftp(&mut self, ftp: u16) {
        self.ftp = ftp
    }

    pub fn update(&mut self, message: UserProfileMessage) {
        match message {
            UserProfileMessage::SaveProfile(name, ftp) => {
                self.name = name;
                self.ftp = ftp;
            }
            _ => {}
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct UserProfileState {
    name_input: String,
    ftp_input: String,
    name_input_field: text_input::State,
    ftp_input_field: text_input::State,
    save_button: button::State,
    delete_button: button::State,
    editing: bool,
}

#[derive(Debug, Clone)]
pub enum UserProfileMessage {
    NameInputChanged(String),
    FtpInputChanged(String),
    SaveProfile(String, u16),
    DeleteProfile,
    Editing(bool),
    Clear,
}

impl UserProfileState {
    // Create a dummy account that user will update
    //pub fn new(active: bool) -> Self {
    //    Self {
    //        active: active,
    //        state: Default::default(),
    //        ..Default::default()
    //    }
    //}

    //pub fn set_active(&mut self, active: bool) {
    //    self.active = active;
    //}

    //pub fn active(&self) -> bool {
    //    self.active
    //}

    pub fn update(&mut self, message: UserProfileMessage) {
        match message {
            UserProfileMessage::NameInputChanged(value) => {
                self.editing = true;
                self.name_input = value;
            }
            UserProfileMessage::FtpInputChanged(value) => {
                self.editing = true;
                match value.parse::<u16>() {
                    Ok(_) => self.ftp_input = value,
                    Err(_) => {
                        if value.len() == 0 {
                            self.ftp_input = value;
                        }
                    }
                }
            }
            UserProfileMessage::Clear => {
                self.editing = false;
                self.name_input.clear();
                self.ftp_input.clear();
            }
            UserProfileMessage::Editing(editing) => {
                self.editing = editing;
            }
            //UserProfileMessage::SaveProfile(_, _) => {
            //    self.name = self.name_input.clone();
            //    match self.ftp_input.parse::<u16>() {
            //        Ok(v) => self.ftp = v,
            //        Err(_) => {
            //            //TODO Should display error message on screen
            //            // Should possibly disabling changing screen state
            //            // until valid value is entered
            //            error!("Invalid FTP value");
            //        }
            //    }
            //    self.name_input.clear();
            //    self.ftp_input.clear();
            //}
            // SaveProfile and Delete Profile are handled by the main application
            // update() method
            _ => {}
        }
    }

    pub fn view(&mut self, profile: &UserProfile) -> Element<UserProfileMessage> {
        let field_text = |text| Text::new(text).size(16).width(Length::Units(50));
        // If we our name_input field is empty, then set name_input to current
        // profile name
        if !self.editing {
            if self.name_input.len() == 0 {
                self.name_input = profile.name.clone();
            }
            if self.ftp_input.len() == 0 && profile.ftp != 0 {
                self.ftp_input = profile.ftp.to_string();
            }
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
                                &mut self.name_input_field,
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
                                &mut self.ftp_input_field,
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
                            Button::new(&mut self.save_button, Text::new("Save").size(16))
                                .on_press(UserProfileMessage::SaveProfile(
                                    self.name_input.clone(),
                                    self.ftp_input.parse::<u16>().unwrap_or(0),
                                ))
                                .padding(8),
                        )
                        .push(
                            Button::new(&mut self.delete_button, Text::new("Delete").size(16))
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
