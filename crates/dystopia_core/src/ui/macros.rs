#[macro_export]
macro_rules! distributed_list_element {
    ($parent:expr, $($bundle:expr),+) => {
        {
            let mut entities = Vec::new();
            $parent
                .spawn(NodeBundle {
                    style: Style {
                        width: bevy::ui::Val::Percent(100.),
                        justify_content: bevy::ui::JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|kv_elem| {
                    $(entities.push(kv_elem.spawn($bundle).id());)+
                });
            entities
        }
    };
}

#[macro_export]
macro_rules! spawn_list {
    ($parent:expr, $($elem:expr),+) => {
        {
            let mut entities = Vec::new();
            $(entities.push($parent.spawn($elem).id());)+
            entities
        }
    };
}

#[macro_export]
macro_rules! merge_list {
    ($($elem:expr),+) => {
        {
            let mut entities = Vec::new();
            $(entities.extend($elem);)+
            entities
        }
    };
}
