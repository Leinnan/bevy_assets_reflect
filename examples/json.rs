use bevy::prelude::*;

#[derive(Asset, Reflect, Debug)]
#[reflect(Debug, Default)]
struct DataAsset {
    pub some_data: String,
    pub another: u64,
    pub skipped_field: String,
}

impl Default for DataAsset {
    fn default() -> Self {
        Self {
            some_data: "".to_string(),
            another: 0,
            skipped_field: "This is a default value".to_string(),
        }
    }
}

#[derive(Deref, Resource, DerefMut)]
struct AssetStorage(Handle<DataAsset>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "examples/assets".to_string(),
            processed_file_path: "examples/imported_assets/Default".to_string(),
            ..Default::default()
        }))
        .register_type::<DataAsset>()
        .add_plugins(bevy_assets_reflect::ReflectAssetPlugin::<DataAsset>::new_json(&["data.json"]))
        .add_systems(Startup, setup)
        .add_systems(Update, on_loaded)
        .run();
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AssetStorage(asset_server.load("example.data.json")));
    commands.spawn(Camera2d);
}
fn on_loaded(
    mut asset_events: MessageReader<AssetEvent<DataAsset>>,
    assets: Res<Assets<DataAsset>>,
    mut commands: Commands,
) {
    for ev in asset_events.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else {
            continue;
        };
        let Some(asset) = assets.get(*id) else {
            continue;
        };
        commands.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                padding: UiRect::all(Val::Px(50.0)),
                ..Default::default()
            },
            children![
                (Text::new(format!("Some data field -> {}", asset.some_data))),
                (Text::new(format!("Another data field -> {}", asset.another))),
                (Text::new(format!("Skipped field -> {}", asset.skipped_field)))
            ],
        ));
    }
}
