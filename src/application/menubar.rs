use crate::application::Message;
use iced::{button, Button, HorizontalAlignment, Length, Row, Text, VerticalAlignment};

#[derive(Debug, Default, Clone)]
pub struct MenuBar {
    workouts_button: button::State,
    devices_button: button::State,
    userprofiles_button: button::State,
}

impl MenuBar {
    pub fn view(&mut self) -> Row<Message> {
        let menu_button = |state, label, message| {
            let label = Text::new(label)
                .size(16)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center);
            Button::new(state, label)
                .padding(8)
                .width(Length::Units(100))
                .min_width(100)
                .height(Length::Units(35))
                .min_height(35)
                .on_press(message)
        };

        Row::new()
            .width(Length::Fill)
            .push(menu_button(
                &mut self.workouts_button,
                "Workouts",
                Message::ShowWorkouts,
            ))
            .push(menu_button(
                &mut self.devices_button,
                "Devices",
                Message::ShowDevices,
            ))
            .push(menu_button(
                &mut self.userprofiles_button,
                "User Profiles",
                Message::ShowUserProfiles,
            ))
    }
}
