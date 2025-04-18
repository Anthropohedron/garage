use std::{collections::HashMap, sync::LazyLock};

pub static STATUS: LazyLock<HashMap<DoorStatus, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (DoorStatus::Closed, "Closed"),
        (DoorStatus::Open, "Open"),
        (DoorStatus::Indeterminate, "Indeterminate"),
        (DoorStatus::Invalid, "Invalid"),
    ])
});

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum DoorStatus {
    Closed,
    Open,
    Indeterminate,
    Invalid,
}
