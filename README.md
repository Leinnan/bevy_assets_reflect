# bevy_assets_reflect

[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)

Simple package for adding assets loaders for types that implement `Reflect` and `Asset` trait. It can be used for types that don't implement `Serialize` or `Deserialize` traits thanks to using the Bevy reflect serialization.


For adding use add plugins API, for example:
```rust
app.add_plugins(bevy_assets_reflect::JsonReflectAssetPlugin::<SomeType>::new(
            &["that_type.json"]
        ));
```

## License

`bevy_ehttp` is dual-licensed under MIT and Apache 2.0 at your option.

## Compatibility

| bevy | bevy_assets_reflect |
| ---: | ---------: |
| 0.17 |        0.1 |
