use bevy::state::state::States;

/// The state of game process. The root state.
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ProcessState {
    /// Game preparation. The player should see nothing except a
    /// progress bar.
    #[default]
    Prepare,

    /// From the first frame of main menu to the last frame when
    /// player presses "exit" or the game crashes. 
    InGame,

    /// After the player presses "exit" or the game crashes. We should
    /// generate crash reports in this state.
    Exit,
}

/// The main game state. Controls simulation for entities, buildings etc.
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    /// The game haven't started yet. Including the menu and other interfaces
    /// like world generation settings. 
    #[default]
    Initialize,

    /// The game is stepping.
    Simulate,

    /// The game is paused and everything should be still, including the
    /// cosmos.
    Pause,
}

/// The asset state.
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum AssetState {
    /// Load all assets.
    #[default]
    Load,

    /// Loaded all assets.
    Finish,
}
