#![doc = include_str!("../README.md")]

extern crate core;

use core::{any::TypeId, marker::PhantomData};

use bevy_app::{App, Plugin};
use bevy_asset::{
    AssetApp, AsyncWriteExt, io::Reader, processor::LoadTransformAndSave,
    transformer::IdentityAssetTransformer,
};
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_reflect::{
    TypePath, TypeRegistryArc,
    serde::{TypedReflectDeserializer, TypedReflectSerializer},
};
use serde::{Deserialize, Serialize, de::DeserializeSeed};
use thiserror::Error;

/// Plugin to load your asset type `A` from files.
pub struct ReflectAssetPlugin<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>,
    default_save_format: Option<AssetFormat>,
    default_load_format: Option<AssetFormat>,
    register_default_processors: bool,
}

impl<A: bevy_reflect::Reflect + bevy_asset::Asset> Plugin for ReflectAssetPlugin<A> {
    fn build(&self, app: &mut App) {
        let registry = app.world().resource::<AppTypeRegistry>();
        #[cfg(feature = "extra_checks")]
        {
            if !registry.read().contains(TypeId::of::<A>()) {
                bevy_log::error!(
                    "Asset type {} is not registered, trying to load assets will fail.",
                    core::any::type_name::<A>()
                );
            }
        }
        let loader = self.loader(registry);
        let saver = self.saver(registry);
        app.init_asset::<A>()
            .register_asset_loader(loader)
            .register_asset_processor::<LoadTransformAndSave<
                ReflectionAssetLoader<A>,
                IdentityAssetTransformer<A>,
                ReflectionAssetSaver<A>,
            >>(LoadTransformAndSave::new(
                IdentityAssetTransformer::<A>::default(),
                saver,
            ));

        if self.register_default_processors {
            for extension in self.extensions.iter() {
                app.set_default_asset_processor::<LoadTransformAndSave<
                    ReflectionAssetLoader<A>,
                    bevy_asset::transformer::IdentityAssetTransformer<A>,
                    ReflectionAssetSaver<A>,
                >>(extension);
            }
        }
    }
}

impl<A: bevy_reflect::Reflect + bevy_asset::Asset> ReflectAssetPlugin<A> {
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_owned(),
            _marker: PhantomData,
            default_load_format: None,
            default_save_format: None,
            register_default_processors: false,
        }
    }
    #[cfg(feature = "json")]
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new_json(extensions: &[&'static str]) -> Self {
        Self::new(extensions)
            .with_load_format(AssetFormat::Json)
            .with_save_format(AssetFormat::Json)
    }
    #[cfg(feature = "postcard")]
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn new_postcard(extensions: &[&'static str]) -> Self {
        Self::new(extensions)
            .with_load_format(AssetFormat::Postcard)
            .with_save_format(AssetFormat::Postcard)
    }
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn with_save_format(self, default_format: AssetFormat) -> Self {
        Self {
            default_save_format: Some(default_format),
            ..self
        }
    }
    /// Create a new plugin that will load assets from files with the given extensions.
    pub fn with_load_format(self, default_format: AssetFormat) -> Self {
        Self {
            default_load_format: Some(default_format),
            ..self
        }
    }

    pub fn with_default_assets_processors(self, register_default_processors: bool) -> Self {
        Self {
            register_default_processors,
            ..self
        }
    }

    /// Create a new loader that will load assets from files with the given extensions.
    pub fn loader(&self, registry: &AppTypeRegistry) -> ReflectionAssetLoader<A> {
        ReflectionAssetLoader {
            phantom: PhantomData,
            registry: registry.0.clone(),
            extensions: self.extensions.clone(),
            default_format: self.default_load_format,
        }
    }
    /// Create a new saver that will save assets to files with the given extensions.
    pub fn saver(&self, registry: &AppTypeRegistry) -> ReflectionAssetSaver<A> {
        ReflectionAssetSaver {
            phantom: PhantomData,
            registry: registry.0.clone(),
            default_format: self.default_save_format,
        }
    }
}

/// struct that loads assets from files with the given extensions.
#[derive(TypePath)]
pub struct ReflectionAssetLoader<T> {
    phantom: PhantomData<T>,
    registry: TypeRegistryArc,
    extensions: Vec<&'static str>,
    default_format: Option<AssetFormat>,
}

impl<T> ReflectionAssetLoader<T> {
    fn get_format(
        &self,
        settings: &ReflectionAssetSettings,
        load_ctx: &bevy_asset::LoadContext,
    ) -> AssetFormat {
        if let Some(format) = settings.format.or(self.default_format) {
            return format;
        }
        let Some(extension) = load_ctx.path().get_full_extension() else {
            return AssetFormat::Json;
        };

        #[cfg(feature = "postcard")]
        if extension.ends_with("postcard") {
            return AssetFormat::Postcard;
        }
        #[cfg(feature = "json")]
        {
            AssetFormat::Json
        }
        #[cfg(not(feature = "json"))]
        {
            AssetFormat::None
        }
    }
}
/// Error that can occur when loading an asset.
#[derive(Debug, Error)]
pub enum ReflectLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
    /// Type not registered
    #[error("Type not registered. Please register the type using `app.register_type::<T>()`")]
    TypeNotRegistered,
    /// Failed to downcast
    #[error("Failed to downcast")]
    FailedToDowncast,
    /// Failed to write to writer
    #[error("Failed to write to writer")]
    WriteError(std::io::Error),
    /// Failed to write in postcard format
    #[error("Failed to write in postcard format")]
    WritePostcardError,
    /// No format specified for save
    #[error("No format specified for save")]
    NoFormatSpecified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetFormat {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "json")]
    JsonPretty,
    #[cfg(feature = "postcard")]
    Postcard,
    None,
}
impl Default for AssetFormat {
    fn default() -> Self {
        if cfg!(feature = "json") {
            AssetFormat::Json
        } else if cfg!(feature = "postcard") {
            AssetFormat::Postcard
        } else {
            AssetFormat::None
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ReflectionAssetSettings {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub format: Option<AssetFormat>,
}

impl<T: bevy_reflect::Reflect + bevy_asset::Asset> bevy_asset::AssetLoader
    for ReflectionAssetLoader<T>
{
    type Asset = T;
    type Settings = ReflectionAssetSettings;
    type Error = ReflectLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let type_registry = self.registry.read();
        let Some(registration) = type_registry.get(TypeId::of::<T>()) else {
            return Err(ReflectLoaderError::TypeNotRegistered);
        };
        let deserializer = TypedReflectDeserializer::new(registration, &type_registry);

        let reflect_value = match self.get_format(settings, load_context) {
            #[cfg(feature = "postcard")]
            AssetFormat::Postcard => {
                let mut value = postcard::Deserializer::from_bytes(&bytes[..]);
                deserializer
                    .deserialize(&mut value)
                    .map_err(|_| ReflectLoaderError::WritePostcardError)?
            }
            #[cfg(feature = "json")]
            AssetFormat::Json | AssetFormat::JsonPretty => {
                let value: serde_json::Value = serde_json::from_slice(&bytes)?;
                deserializer.deserialize(value)?
            }
            _ => {
                return Err(ReflectLoaderError::NoFormatSpecified);
            }
        };
        reflect_value
            .try_take::<T>()
            .map_err(|_| ReflectLoaderError::FailedToDowncast)
    }

    fn extensions(&self) -> &[&str] {
        self.extensions.as_slice()
    }
}

/// struct that saves assets to files with the given extensions.
#[derive(TypePath)]
pub struct ReflectionAssetSaver<T> {
    phantom: PhantomData<T>,
    registry: TypeRegistryArc,
    default_format: Option<AssetFormat>,
}

impl<A: bevy_asset::Asset + bevy_reflect::Reflect> bevy_asset::saver::AssetSaver
    for ReflectionAssetSaver<A>
{
    type Asset = A;
    type Settings = ReflectionAssetSettings;
    type OutputLoader = ReflectionAssetLoader<A>;
    type Error = ReflectLoaderError;

    async fn save(
        &self,
        writer: &mut bevy_asset::io::Writer,
        asset: bevy_asset::saver::SavedAsset<'_, Self::Asset>,
        settings: &Self::Settings,
    ) -> Result<<Self::OutputLoader as bevy_asset::AssetLoader>::Settings, Self::Error> {
        let Some(format) = settings.format.or(self.default_format) else {
            return Err(ReflectLoaderError::NoFormatSpecified);
        };
        let reflect_serialize = {
            let type_registry = self.registry.read();
            let serializer =
                TypedReflectSerializer::new(asset.as_partial_reflect(), &type_registry);

            match format {
                #[cfg(feature = "postcard")]
                AssetFormat::Postcard => {
                    let bytes = postcard::to_stdvec(&serializer)
                        .map_err(|_| ReflectLoaderError::WritePostcardError)?;
                    Ok(bytes)
                }
                #[cfg(feature = "json")]
                AssetFormat::Json => serde_json::to_vec(&serializer),
                #[cfg(feature = "json")]
                AssetFormat::JsonPretty => serde_json::to_vec_pretty(&serializer),
                _ => return Err(ReflectLoaderError::NoFormatSpecified),
            }
        }?;
        writer
            .write_all(&reflect_serialize)
            .await
            .map_err(ReflectLoaderError::WriteError)?;
        Ok(ReflectionAssetSettings {
            format: Some(format),
        })
    }
}
