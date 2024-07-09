use std::path::{PathBuf, Path};

use bevy::render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, ShaderType, TextureDimension, TextureFormat}, texture::Image};
use image::{
    buffer::ConvertBuffer, DynamicImage, GenericImageView, ImageBuffer, ImageError, Pixel, RgbImage, Rgba, RgbaImage
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildTextureArrayError {
    #[error("Texture dimensions did not match arguments.")]
    IncorrectDimensions,
    #[error("Image Error: {0}")]
    ImageError(#[from] ImageError)
}

pub fn create_texture_array<I: Into<DynamicImage>, It: Iterator<Item = I>>(width: u32, height: u32, images: It) -> Result<Image, BuildTextureArrayError> {
    let images: Vec<DynamicImage> = images.map(|img| img.into()).collect();
    let layer_count = images.len();
    let mut stack_2d = RgbaImage::new(width, height * images.len() as u32);
    images.into_iter().enumerate().try_for_each(|(index, layer)| {
        if layer.width() != width
        || layer.height() != height {
            return Err(BuildTextureArrayError::IncorrectDimensions);
        }
        let y_start = index as u32 * height;
        for y in 0..height {
            let stack_y = y_start + y;
            for x in 0..width {
                let pixel = layer.get_pixel(x, y);
                stack_2d.put_pixel(x, stack_y, pixel);
            }
        }
        Ok(())
    })?;
    let stack_2d_rgbaf32: image::Rgba32FImage = stack_2d.convert();
    let mut buffer = Vec::with_capacity(stack_2d.width() as usize * stack_2d.height() as usize * 4 * 4);
    stack_2d.into_raw()
        .into_iter()
        .for_each(|component| {
            #[cfg(target_endian = "little")]
            let bytes = component.to_le_bytes();
            #[cfg(not(target_endian = "little"))]
            let bytes = component.to_be_bytes();
            buffer.extend(bytes);
        });
    Ok(Image::new(
        Extent3d { width: width, height: height, depth_or_array_layers: layer_count as u32 },
        TextureDimension::D2,
        buffer,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::all(),
    ))
}

pub fn create_texture_array_from_paths<P: AsRef<Path>>(width: u32, height: u32, paths: Vec<P>) -> Result<Image, BuildTextureArrayError> {
    let mut stack_2d = RgbaImage::new(width, height * paths.len() as u32);
    paths.iter().enumerate().try_for_each(|(index, path)| {
        let layer = image::open(path.as_ref())?;
        if layer.width() != width
        || layer.height() != height {
            return Err(BuildTextureArrayError::IncorrectDimensions);
        }
        let y_start = index as u32 * height;
        for y in 0..height {
            let stack_y = y_start + y;
            for x in 0..width {
                let pixel = layer.get_pixel(x, y);
                stack_2d.put_pixel(x, stack_y, pixel);
            }
        }
        Ok(())
    })?;
    let stack_2d: image::Rgba32FImage = stack_2d.convert();
    let mut buffer = Vec::with_capacity(stack_2d.width() as usize * stack_2d.height() as usize * 4 * 4);
    stack_2d.into_raw()
        .into_iter()
        .for_each(|component| {
            #[cfg(target_endian = "little")]
            let bytes = component.to_le_bytes();
            #[cfg(not(target_endian = "little"))]
            let bytes = component.to_be_bytes();
            buffer.extend(bytes);
        });
    Ok(Image::new(
        Extent3d { width: width, height: height, depth_or_array_layers: paths.len() as u32 },
        TextureDimension::D2,
        buffer,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::all(),
    ))
}