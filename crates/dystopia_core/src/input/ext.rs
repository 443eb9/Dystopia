use bevy::{app::App, ecs::schedule::ScheduleLabel, prelude::IntoSystemConfigs};

use crate::input::event::condition::keyboard_event_activating;

pub trait KeyboardEventApp {
    fn register_keyboard_event<const E: usize, M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self;
}

impl KeyboardEventApp for App {
    fn register_keyboard_event<const E: usize, M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        self.add_systems(schedule, systems.run_if(keyboard_event_activating(E)))
    }
}
