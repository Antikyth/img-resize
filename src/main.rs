#![recursion_limit = "256"]
#![feature(if_let_guard)]

mod cli;
mod extensions;

pub use extensions::IteratorExtensions;

use clap::{CommandFactory, Parser};
use clap_complete as completion;
use cli::Size;
use image::{imageops, GenericImage, GenericImageView, RgbaImage};
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let cli::Args {
        subcommand,
        resize_args,
    } = cli::Args::parse();

    if let Some(cli::Subcommands::Generate { shell }) = subcommand {
        // Generate completions for the given `shell` if it is used.

        completion::generate(
            shell,
            &mut cli::Args::command(),
            cli::NAME,
            &mut io::stdout(),
        );
    } else if let Some(resize_args) = resize_args {
        // If `generate` is not used, then do the image resize instead.

        let cli::ResizeArgs {
            input_path,
            output_path,

            fit_scale: Size(fit_width, fit_height),
            output_scale: Size(output_width, output_height),
        } = resize_args;

        // Read the image in.
        let image = image::io::Reader::open(&input_path)?.decode()?.into_rgba8();

        // Determine the scaled dimensions for the new image.
        let (width, height) = (
            (image.width() * output_width) / fit_width,
            (image.height() * output_height) / fit_height,
        );

        // Create the new image with the desired dimensions and copy the old one onto it.
        let mut new_image = RgbaImage::new(width, height);
        repeat(&mut new_image, &image);

        // Save the image.
        let new_path = output_path.unwrap_or(input_path);
        new_image.save(new_path)?;
    }

    Ok(())
}

/// Repeats the given `repeated` image across the given `base` image as many times as it will fit.
///
/// This is similar to [`imageops::tile`], but if the `repeated` image would be cut off, it is not
/// overlaid.
pub fn repeat<BaseImage, RepeatedImage>(base: &mut BaseImage, repeated: &RepeatedImage)
where
    BaseImage: GenericImage,
    RepeatedImage: GenericImageView<Pixel = BaseImage::Pixel>,
{
    // The number of horizontal repetitions of `repeated`.
    let horizontal = base.width() / repeated.width();
    // The number of vertical repetitions of `repeated`.
    let vertical = base.height() / repeated.height();

    // For each repetition position...
    for (i, j) in (0..horizontal).mix(0..vertical) {
        let (x, y) = (
            i64::from(i * repeated.width()),
            i64::from(j * repeated.height()),
        );

        // Overlay the repeated image.
        imageops::overlay(base, repeated, x, y);
    }
}
