#[cfg(test)]
mod world_tests {
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .add_plugins(TransformPlugin);

        app
    }

    #[test]
    fn test_world_collision() {
        let mut app = setup_test_app();
        app.update();
    }
}
