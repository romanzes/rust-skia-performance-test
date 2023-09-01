#![allow(unused)]

use clap::Parser;
use skia_safe::canvas::SrcRectConstraint;
use skia_safe::svg::Dom;
use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
};
use skia_safe::{
    surfaces, Canvas, Color, CubicResampler, Data, EncodedImageFormat, FilterMode, Image,
    MipmapMode, Paint, Path as SkPath, Rect, SamplingOptions, Surface, Typeface,
};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CANVAS_SIZE: i32 = 512;

#[derive(Parser)]
struct Cli {
    #[arg(long = "dir")]
    dir_path: std::path::PathBuf,
    #[arg(long = "loop", default_value_t = 1)]
    loop_count: u8,
    #[arg(long = "path")]
    draw_path: bool,
    #[arg(long = "raster")]
    draw_raster: bool,
    #[arg(long = "text")]
    draw_text: bool,
    #[arg(long = "svg")]
    draw_svg: bool,
    #[arg(long = "save")]
    save: bool,
    #[arg(long = "scale", default_value_t = 1)]
    scale: u8,
}

fn main() {
    let mut args = Cli::parse();

    if !(args.draw_path || args.draw_raster || args.draw_text || args.draw_svg || args.save) {
        args.draw_path = true;
        args.draw_raster = true;
        args.draw_text = true;
        args.draw_svg = true;
        args.save = true;
    }

    for _ in 0..args.loop_count {
        performance_test(
            &args.dir_path,
            args.draw_path,
            args.draw_raster,
            args.draw_text,
            args.draw_svg,
            args.save,
            args.scale,
        );
    }
}

fn performance_test(
    working_path: &PathBuf,
    path: bool,
    raster: bool,
    text: bool,
    svg: bool,
    save: bool,
    scale: u8,
) {
    if let Some(mut surface) =
        surfaces::raster_n32_premul((CANVAS_SIZE * scale as i32, CANVAS_SIZE * scale as i32))
    {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let canvas = surface.canvas();
        canvas.clear(Color::WHITE);
        canvas.scale((scale as f32, scale as f32));
        if path {
            let path_path = check_file_exists(working_path.join("path.txt"));
            draw_path(canvas, &mut paint, &path_path);
        }
        if raster {
            let raster_path = check_file_exists(working_path.join("mars.jpg"));
            draw_raster(canvas, &mut paint, &raster_path);
        }
        if text {
            let font_path = check_file_exists(working_path.join("Adigiana_Ultra.ttf"));
            draw_text(canvas, &font_path);
        }
        if svg {
            let svg_path = check_file_exists(working_path.join("pinocchio.svg"));
            draw_svg(canvas, &svg_path);
        }
        if save {
            let output_path = working_path.join("output-rust.png");
            save_to_png(&mut surface, &output_path);
        }
    }
}

fn check_file_exists(path: PathBuf) -> PathBuf {
    if !path.exists() {
        panic!("File doesn't exist: {:?}", path);
    }
    path
}

fn draw_path(canvas: &mut Canvas, paint: &mut Paint, path_path: &PathBuf) {
    paint.set_color(Color::BLACK);
    canvas.save();
    canvas.translate((12.0, 12.0));
    canvas.scale((0.45, 0.45));
    if let Ok(path_def) = std::fs::read_to_string(path_path) {
        if let Some(path) = SkPath::from_svg(path_def) {
            canvas.draw_path(&path, paint);
        }
    }
    canvas.restore();
}

fn draw_raster(canvas: &mut Canvas, paint: &mut Paint, raster_path: &PathBuf) {
    canvas.save();
    canvas.translate((250.0, 0.0));
    canvas.scale((0.05, 0.05));
    if let Ok(bitmap_data) = data_from_file_path(raster_path) {
        if let Some(bitmap) = Image::from_encoded(bitmap_data) {
            let rect = Rect::from_wh(bitmap.width() as f32, bitmap.height() as f32);
            canvas.draw_image_rect_with_sampling_options(
                bitmap,
                Some((&rect, SrcRectConstraint::Fast)),
                &rect,
                SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear),
                &paint,
            );
        }
    }
    canvas.restore();
}

fn draw_text(canvas: &mut Canvas, font_path: &PathBuf) {
    let mut typeface_provider = TypefaceFontProvider::new();
    if let Ok(data) = data_from_file_path(font_path) {
        if let Some(font) = Typeface::from_data(data, None) {
            typeface_provider.register_typeface(font, Some("Adigiana"));
        }
    }
    let mut font_collection = FontCollection::new();
    font_collection.set_asset_font_manager(Some(typeface_provider.into()));

    let mut style = ParagraphStyle::new();
    let mut text_style = TextStyle::new();
    text_style.set_color(Color::from_rgb(0, 0, 0));
    text_style.set_font_size(15.0);
    text_style.set_font_families(&["Adigiana"]);
    style.set_text_style(&text_style);
    let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection);
    paragraph_builder.add_text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, ");
    text_style.set_color(Color::from_rgb(255, 0, 0));
    paragraph_builder.push_style(&text_style);
    paragraph_builder
        .add_text("sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ");
    text_style.set_color(Color::from_rgb(0, 255, 0));
    paragraph_builder.push_style(&text_style);
    paragraph_builder
        .add_text("Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut ");
    text_style.set_color(Color::from_rgb(0, 0, 255));
    paragraph_builder.push_style(&text_style);
    paragraph_builder
        .add_text("aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in ");
    text_style.set_color(Color::from_rgb(255, 255, 0));
    paragraph_builder.push_style(&text_style);
    paragraph_builder
        .add_text("voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint ");
    text_style.set_color(Color::from_rgb(0, 255, 255));
    paragraph_builder.push_style(&text_style);
    paragraph_builder.add_text("occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n");

    let mut paragraph = paragraph_builder.build();
    paragraph.layout(225.0);

    paragraph.paint(canvas, (25.0, 275.0));
}

fn draw_svg(canvas: &mut Canvas, svg_path: &PathBuf) {
    canvas.save();
    canvas.translate((350.0, 275.0));
    canvas.scale((0.22, 0.22));
    if let Ok(svg_data) = bytes_from_file_path(svg_path) {
        if let Ok(svg) = Dom::from_bytes(&svg_data) {
            svg.render(canvas);
        }
    }
    canvas.restore();
}

fn data_from_file_path(file_path: &Path) -> std::io::Result<Data> {
    bytes_from_file_path(file_path).map(|bytes| Data::new_copy(&bytes.as_slice()))
}

fn bytes_from_file_path(file_path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_path).unwrap();
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).map(|_| bytes)
}

fn save_to_png(surface: &mut Surface, output_path: &PathBuf) {
    let image = surface.image_snapshot();
    let mut context = surface.direct_context();
    if let Some(data) = image.encode(context.as_mut(), EncodedImageFormat::PNG, None) {
        let mut file = File::create(output_path).unwrap();
        let bytes = data.as_bytes();
        file.write_all(bytes).unwrap();
    }
}
