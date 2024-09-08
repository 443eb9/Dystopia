use bevy::app::{App, Plugin};

pub mod cosmos_view;
pub mod focusing_body;

pub struct DystopiaTransitionPlugin;

impl Plugin for DystopiaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            cosmos_view::CosmosViewSceneTransitionPlugin,
            focusing_body::FocusingBodyTransitionPlugin,
        ));
    }
}

#[macro_export]
macro_rules! impl_transition_plugin {
    ($ty: ident, $state: expr, $on_enter: tt, $on_update: tt, $on_exit: tt) => {
        pub struct $ty;
            
        impl bevy::prelude::Plugin for $ty {
            fn build(&self, app: &mut bevy::prelude::App) {
                use bevy::prelude::IntoSystemConfigs;
                
                app.add_systems(bevy::prelude::OnEnter($state), $on_enter)
                    .add_systems(
                        bevy::prelude::Update,
                        $on_update.run_if(bevy::prelude::in_state($state)),
                    )
                    .add_systems(bevy::prelude::OnExit($state), $on_exit);
            }
        }
    };

    ($ty: ident, $state: expr, $on_enter: tt, $on_update: expr, $on_exit: tt) => {
        impl_transition_plugin!($ty, $state, $on_enter, $on_update, $on_exit);
    };
}
