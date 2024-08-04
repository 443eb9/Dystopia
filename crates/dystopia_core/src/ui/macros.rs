#[macro_export]
macro_rules! key_value_list_element {
    ($parent: expr, $height: expr, $bundle_l: expr, $bundle_r: expr) => {
        $parent
            .spawn(NodeBundle {
                style: Style {
                    height: $height,
                    width: bevy::ui::Val::Percent(100.),
                    justify_content: bevy::ui::JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|kv_elem| {
                kv_elem.spawn($bundle_l);
                kv_elem.spawn($bundle_r);
            });
    };
}
