// The User Profile screen provides a way to create, edit, or delete the current
// active profile. Profiles are switched via the drop down in the menubar. If no
// user profiles exist, then this screen will be the default starting screen for
// the application. Once a user profile exists, then the starting screen will
// default to the Library screen.
//
use crate::application::user_profile::UserProfile;
use iced::{
    button, text_input, Align, Button, Column, Container, Element, Length, Row, Text, TextInput,
};

#[derive(Debug, Clone, Default)]
pub struct State {
    name_input: String,
    ftp_input: String,
    name_input_field: text_input::State,
    ftp_input_field: text_input::State,
    save_button: button::State,
    delete_button: button::State,
    editing: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInputChanged(String),
    FtpInputChanged(String),
    SaveProfile(String, u16),
    DeleteProfile,
    Editing(bool),
    Clear,
}

impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::NameInputChanged(value) => {
                self.editing = true;
                self.name_input = value;
            }
            Message::FtpInputChanged(value) => {
                self.editing = true;
                match value.parse::<u16>() {
                    Ok(_) => self.ftp_input = value,
                    Err(_) => {
                        if value.is_empty() {
                            self.ftp_input = value;
                        }
                    }
                }
            }
            Message::Clear => {
                self.editing = false;
                self.name_input.clear();
                self.ftp_input.clear();
            }
            Message::Editing(editing) => self.editing = editing,
            _ => {}
        }
    }

    pub fn view(&mut self, profile: &UserProfile) -> Element<Message> {
        // Closure for consistent label for text fields in the view
        let field_text = |text| Text::new(text).size(16).width(Length::Units(50));

        // Set fields to current user profile if editing is false.
        if !self.editing {
            if self.name_input.is_empty() {
                self.name_input = profile.name.clone();
            }
            if self.ftp_input.is_empty() && profile.ftp != 0 {
                self.ftp_input = profile.ftp.to_string();
            }
        }

        // Closure for creating a button in order to specify options once.
        let button = |state, label| Button::new(state, Text::new(label).size(16)).padding(8);
        let mut save_button = button(&mut self.save_button, "Save");

        // Only enable the save button if there is data in both the name and ftp
        // fields as long as the data is different from current user profile. If
        // any field isn't filled out, do not enable the button.
        if (!self.name_input.is_empty()
            && !self.ftp_input.is_empty()
            && self.name_input != profile.name
            && self.ftp_input != profile.ftp.to_string())
            || (self.name_input != profile.name
                && !self.name_input.is_empty()
                && self.ftp_input == profile.ftp.to_string())
            || (self.name_input == profile.name
                && self.ftp_input != profile.ftp.to_string()
                && !self.ftp_input.is_empty())
        {
            save_button = save_button.on_press(Message::SaveProfile(
                self.name_input.clone(),
                self.ftp_input.parse::<u16>().unwrap_or(0),
            ))
        }

        let mut delete_button = button(&mut self.delete_button, "Delete");
        if !profile.name.is_empty() {
            delete_button = delete_button.on_press(Message::DeleteProfile);
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
                                Message::NameInputChanged,
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
                                Message::FtpInputChanged,
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
                        .push(save_button)
                        .push(delete_button),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
