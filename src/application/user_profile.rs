// UserProfile to allow multiple users of the software. Allows for a user
// to easily have workouts adjusted based on their FTP setting.

// TODO Improve the styling.
// TODO Capture tabs to change focus of input fields
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct UserProfile {
    // First three fields will be serialized into a TOML file
    // Name field
    pub name: String,
    // FTP field
    pub ftp: u16,
    // Active field for setting active profile when SavingState. Default will
    // be last active profile.
    pub active: bool,
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_user_profile_create_active() {
        let user = UserProfile::new(true);
        assert!(user.active);
    }

    #[test]
    fn test_user_profile_create_inactive() {
        let user = UserProfile::new(false);
        assert!(!user.active);
    }
}
