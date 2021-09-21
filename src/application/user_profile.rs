use iced::{
    button, text_input, Align, Button, Column, Container, Element, Length, Row, Text, TextInput,
};

use log::error;

// UserProfile to allow multiple users of the software. Allows for a user
// to easily have workouts adjusted based on their FTP setting.

// TODO Improve the styling.
// TODO Capture tabs to change focus of input fields
#[derive(Debug, Clone)]
pub struct UserProfile {
    name: String,
    ftp: u16,
    active: bool,
    state: UserProfileState,
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
            name: "John Doe".to_string(),
            ftp: 200,
            active: active,
            state: Default::default(),
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
            UserProfileMessage::NameInputChanged(value) => self.name = value,
            UserProfileMessage::FtpInputChanged(value) => {
                match value.parse::<u16>() {
                    Ok(v) => self.ftp = v,
                    Err(_) => error!("Invalid value entered for FTP"),
                }
                //TODO: Have an error field that can be displayed to user when
                //and invalid value is entered
            }
            // SaveProfile and Delete Profile are handled by the main application
            // update() method
            _ => {}
        }
    }

    pub fn view(&mut self) -> Element<UserProfileMessage> {
        let field_text = |text| Text::new(text).size(16).width(Length::Units(50));
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
                                "Name",
                                &self.name,
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
                                "FTP",
                                &self.ftp.to_string(),
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
