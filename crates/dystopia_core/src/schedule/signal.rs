use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct InitializationSignal {
    /// This signal is only useful when we need to generate a
    /// new cosmos.
    pub cosmos_initialized: bool,

    /// For generated gardens, we need to wait for cosmos generation,
    /// but for loading gardens from file system, we don't.
    /// 
    /// If true, means the initialization finished.
    pub gardens_initialized: bool,
}
