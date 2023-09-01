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
            draw_path(canvas, &mut paint);
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

fn draw_path(canvas: &mut Canvas, paint: &mut Paint) {
    paint.set_color(Color::BLACK);
    let path_def = r#"
M437.02,74.981c48.352,48.352,74.98,112.64,74.98,181.02s-26.629,132.667-74.98,181.019C388.667,485.371,324.38,512,256,512
s-132.667-26.629-181.02-74.98C26.629,388.668,0,324.381,0,256.001s26.627-132.668,74.98-181.02S187.62,0,256,0
S388.667,26.629,437.02,74.981z M414.392,414.393c31.529-31.529,52.493-70.804,61.137-113.531
c-6.737,9.918-13.182,13.598-17.172-8.603c-4.11-36.195-37.354-13.073-58.259-25.93c-22.002,14.829-71.453-28.831-63.049,20.412
c12.967,22.211,70.004-29.726,41.574,17.271c-18.137,32.809-66.321,105.466-60.053,143.129c0.791,54.872-56.067,11.442-75.657-6.76
c-13.178-36.46-4.491-100.188-38.949-118.043c-37.401-1.624-69.502-5.023-83.997-46.835c-8.723-29.914,9.282-74.447,41.339-81.322
c46.925-29.483,63.687,34.527,107.695,35.717c13.664-14.297,50.908-18.843,53.996-34.875c-28.875-5.095,36.634-24.279-2.764-35.191
c-21.735,2.556-35.739,22.537-24.185,39.479c-42.119,9.821-43.468-60.952-83.955-38.629c-1.029,35.295-66.111,11.443-22.518,4.286
c14.978-6.544-24.43-25.508-3.14-22.062c10.458-0.568,45.666-12.906,36.138-21.201c19.605-12.17,36.08,29.145,55.269-0.941
c13.854-23.133-5.81-27.404-23.175-15.678c-9.79-10.962,17.285-34.638,41.166-44.869c7.959-3.41,15.561-5.268,21.373-4.742
c12.029,13.896,34.275,16.303,35.439-1.671C322.855,39.537,290.008,32,256,32c-48.811,0-95.235,15.512-133.654,44.195
c10.325,4.73,16.186,10.619,6.239,18.148c-7.728,23.027-39.085,53.938-66.612,49.562c-14.293,24.648-23.706,51.803-27.73,80.264
c23.056,7.628,28.372,22.725,23.418,27.775c-11.748,10.244-18.968,24.765-22.688,40.662c7.505,45.918,29.086,88.237,62.635,121.787
C139.916,456.7,196.167,480,256,480C315.832,480,372.084,456.7,414.392,414.393z
    "#;
    canvas.save();
    canvas.translate((12.0, 12.0));
    canvas.scale((0.45, 0.45));
    if let Some(path) = SkPath::from_svg(path_def) {
        canvas.draw_path(&path, paint);
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
