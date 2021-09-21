use iced::{
    button, text_input, Button, Checkbox, Column, Container, Element, Length, Row, Text, TextInput,
};

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

    pub fn view(&mut self) -> Element<UserProfileMessage> {
        Container::new(
            Column::new()
                .width(Length::Shrink)
                .height(Length::Shrink)
                .push(Text::new("User Profile").size(30))
                .push(
                    Row::new()
                        .push(Text::new("Name: ").size(16))
                        .push(TextInput::new(
                            &mut self.state.name_input_field,
                            "Name",
                            &self.name,
                            UserProfileMessage::NameInputChanged,
                        )),
                )
                .push(
                    Row::new()
                        .push(Text::new("FTP: ").size(16))
                        .push(TextInput::new(
                            &mut self.state.ftp_input_field,
                            "FTP",
                            &self.ftp.to_string(),
                            UserProfileMessage::FtpInputChanged,
                        )),
                )
                //                .push(Checkbox::new(
                //                    self.active,
                //                    "Set as active profile",
                //                    UserProfileMessage::ProfileActive,
                //                ))
                .push(
                    Row::new().push(
                        Button::new(&mut self.state.delete_button, Text::new("Delete").size(16))
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
