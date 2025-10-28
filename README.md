# bevy_assets_reflect

Simple package for adding assets loaders for types that implement `Reflect` and `Asset` trait. It can be used for types that don't implement `Serialize` or `Deserialize` traits thanks to using the Bevy reflect serialization.


For adding use add plugins API, for example:
```rust
app.add_plugins(bevy_assets_reflect::JsonReflectAssetPlugin::<SomeType>::new(
            &["that_type.json"]
        ));
```
