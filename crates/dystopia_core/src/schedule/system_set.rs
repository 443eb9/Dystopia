use bevy::prelude::SystemSet;

/// Runs when
/// - [`ProcessState::Prepare`]
/// - [`AssetState::Load`]
/// 
/// in [`Startup`]
/// 
/// Once the game process starts up. Generally for asset preparation.
/// 
/// [`ProcessState::Prepare`]: crate::schedule::state::ProcessState::Prepare
/// [`AssetState::Load`]: crate::schedule::state::AssetState::Load
/// [`Startup`]: bevy::prelude::Startup
#[derive(SystemSet, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PrepareSet;

/// Runs when
/// - [`ProcessState::InGame`]
/// - [`AssetState::Finish`]
/// - [`GameState::Initialize`]
/// 
/// in `OnEnter(GameState::Initialize)`
/// 
/// For the game and cosmos to load/generate.
/// 
/// [`ProcessState::InGame`]: crate::schedule::state::ProcessState::InGame
/// [`AssetState::Finish`]: crate::schedule::state::AssetState::Finish
/// [`GameState::Initialize`]: crate::schedule::state::GameState::Initialize
#[derive(SystemSet, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct InitializeSet;

/// Runs when
/// - [`ProcessState::InGame`]
/// - [`AssetState::Finish`]
/// - [`GameState::Simulate`]
/// 
/// in [`FixedUpdate`]
/// 
/// [`ProcessState::InGame`]: crate::schedule::state::ProcessState::InGame
/// [`AssetState::Finish`]: crate::schedule::state::AssetState::Finish
/// [`GameState::Simulate`]: crate::schedule::state::GameState::Simulate
/// [`FixedUpdate`]: bevy::prelude::FixedUpdate
#[derive(SystemSet, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct SimulationSet;
