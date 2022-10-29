use bevy::asset::{AssetLoader, Error, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::render::texture::{CompressedImageFormats, Image, ImageType, TextureError};
use bevy::utils::BoxedFuture;

#[derive(Default)]
pub struct NormalMappedImageTextureLoader;

impl AssetLoader for NormalMappedImageTextureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            // use the file extension for the image type
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let dyn_img = Image::from_buffer(
                bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::all(),
                false,
            )
            .unwrap();

            load_context.set_default_asset(LoadedAsset::new(dyn_img));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["norm"]
    }
}
