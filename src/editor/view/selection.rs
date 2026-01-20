use super::Location;

pub struct Selection {
    pub anchor: Location,
    pub active: bool,
}

impl Selection {
    pub fn default() -> Self {
        Self {
            anchor: Location::default(),
            active: false,
        }
    }
}
