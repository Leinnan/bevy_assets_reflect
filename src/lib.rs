use std::{any::TypeId, marker::PhantomData};

use bevy_app::{App, Plugin};
use bevy_asset::{io::Reader, AssetApp};
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_reflect::{serde::TypedReflectDeserializer, TypeRegistryArc};
use serde::de::DeserializeSeed;
use thiserror::Error;

/// Plugin to load your asset type `A` from json files.
pub struct JsonReflectAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
}

impl<A: bevy_reflect::Reflect + bevy_asset::Asset> Plugin for JsonReflectAssetPlugin<A> {
    fn build(&self, app: &mut App) {
        let loader = ReflectionAssetLoader::<A> {
            phantom: PhantomData,
            registry: app.world().resource::<AppTypeRegistry>().0.clone(),
            extensions: self.extensions.clone(),
        };
        app.init_asset::<A>().register_asset_loader(loader);
    }
}

impl<A: bevy_reflect::Reflect + bevy_asset::Asset> JsonReflectAssetPlugin<A> {
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
        }
    }
}

pub struct ReflectionAssetLoader<T> {
    phantom: PhantomData<T>,
    registry: TypeRegistryArc,
    extensions: Vec<&'static str>,
}

#[derive(Debug, Error)]
pub enum ReflectLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
    /// Type not registered
    #[error("Type not registered")]
    TypeNotRegistered,
    /// Failed to downcast
    #[error("Failed to downcast")]
    FailedToDowncast,
}

impl<T: bevy_reflect::Reflect + bevy_asset::Asset> bevy_asset::AssetLoader
    for ReflectionAssetLoader<T>
{
    type Asset = T;
    type Settings = ();
    type Error = ReflectLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let type_registry = self.registry.read();
        let Some(registration) = type_registry.get(TypeId::of::<T>()) else {
            return Err(ReflectLoaderError::TypeNotRegistered);
        };
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
        let reflect_value = deserializer.deserialize(value).unwrap();
        reflect_value
            .try_take::<T>()
            .map_err(|_| ReflectLoaderError::FailedToDowncast)
    }

    fn extensions(&self) -> &[&str] {
        self.extensions.as_slice()
    }
}
