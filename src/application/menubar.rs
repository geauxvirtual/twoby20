use crate::application::Message;
use iced::{
    button, Button, Container, HorizontalAlignment, Length, Row, Space, Text, VerticalAlignment,
};

#[derive(Debug, Default, Clone)]
pub struct MenuBar {
    workouts_button: button::State,
    devices_button: button::State,
    userprofile_button: button::State,
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

        let c1 = Container::new(
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
                    &mut self.userprofile_button,
                    "User Profile",
                    Message::ShowUserProfile,
                )),
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill);

        // TODO: Add pick list of available user profiles. Display the default
        // profile and list other available profiles to choose from.
        let c3 = Container::new(
            Row::new().width(Length::Fill).push(
                Text::new("User Profile:")
                    .size(16)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            ),
        )
        .padding(10)
        .width(Length::FillPortion(2))
        .height(Length::Fill);

        Row::new()
            .width(Length::Fill)
            .height(Length::Units(35))
            .push(c1)
            .push(Space::new(Length::FillPortion(2), Length::Shrink))
            .push(c3)
    }
}
