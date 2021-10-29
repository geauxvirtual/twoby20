use iced::{
    button, text_input, Align, Button, Column, Container, Element, Length, Row, Text, TextInput,
};

//use log::error;

// UserProfile to allow multiple users of the software. Allows for a user
// to easily have workouts adjusted based on their FTP setting.

// TODO Improve the styling.
// TODO Capture tabs to change focus of input fields
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct UserProfile {
    // First three fields will be serialized into a TOML file
    // Name field
    name: String,
    // FTP field
    ftp: u16,
    // Active field for setting active profile when SavingState. Default will
    // be last active profile.
    active: bool,
}

impl std::fmt::Display for UserProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //One issue here is that any time Display is called for this
        //struct, the initial default user will be displayed as "New..."
        //This can be worked around by only displaying profiles from [1..]
        //if we ever need to loop through profiles and display them.
        let name = if self.name.is_empty() {
            "New..."
        } else {
            &self.name
        };
        write!(f, "{}", name)
    }
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

    /*pub fn update(&mut self, message: UserProfileMessage) {
    if let UserProfileMessage::SaveProfile(name, ftp) = message {
        self.name = name;
        self.ftp = ftp;
    }*/
    /*match message {
        UserProfileMessage::SaveProfile(name, ftp) => {
            self.name = name;
            self.ftp = ftp;
        }
        _ => {}
    }*/
    //}
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
                        if value.is_empty() {
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
            _ => {}
        }
    }

    pub fn view(&mut self, profile: &UserProfile) -> Element<UserProfileMessage> {
        let field_text = |text| Text::new(text).size(16).width(Length::Units(50));
        // If we our name_input field is empty, then set name_input to current
        // profile name
        if !self.editing {
            if self.name_input.is_empty() {
                self.name_input = profile.name.clone();
            }
            if self.ftp_input.is_empty() && profile.ftp != 0 {
                self.ftp_input = profile.ftp.to_string();
            }
        }

        // Closure for creating a button to only need to specify options once.
        let button = |state, label| Button::new(state, Text::new(label).size(16)).padding(8);
        let mut save_button = button(&mut self.save_button, "Save");

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
            save_button = save_button.on_press(UserProfileMessage::SaveProfile(
                self.name_input.clone(),
                self.ftp_input.parse::<u16>().unwrap_or(0),
            ))
        }

        let mut delete_button = button(&mut self.delete_button, "Delete");
        if !profile.name.is_empty() {
            delete_button = delete_button.on_press(UserProfileMessage::DeleteProfile);
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
