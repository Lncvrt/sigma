#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ansiterm::Colour::Yellow;
use clap::{Arg, Command};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
mod gui;

pub fn is_compressed(data: &[u8]) -> bool {
    let mut decoder = GzDecoder::new(data);
    let mut buffer = Vec::new();
    match decoder.read_to_end(&mut buffer) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn png_to_sigma(input_path: &str, output_path: &str, compress: bool) -> io::Result<()> {
    let img = image::open(&Path::new(input_path)).expect("Failed to open image");
    let (width, height) = img.dimensions();
    let rgba_image = img.to_rgba8();

    let mut data = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel: Rgba<u8> = *rgba_image.get_pixel(x, y);
            if pixel[3] == 0 {
                continue;
            };
            data.push(format!(
                "[{}, {}, {}, {}, {}, {}]",
                pixel[0], pixel[1], pixel[2], pixel[3], x, y
            ));
        }
    }

    let pixel_data = data.join(",");

    let mut file = File::create(output_path)?;

    if compress {
        let mut encoder = GzEncoder::new(file, Compression::new(9));
        writeln!(encoder, "{} {}", height, width)?;
        encoder.write_all(pixel_data.as_bytes())?;
        encoder.finish()?;
    } else {
        writeln!(file, "{} {}", height, width)?;
        file.write_all(pixel_data.as_bytes())?;
        println!("{}", Yellow.paint("Warning: It is recommended to keep compression enabled, this feature may not be updated in the future."));
    }

    println!("Converted the PNG file to be sigma {}", output_path);
    Ok(())
}

fn sigma_to_png(input_path: &str, output_path: &str) -> io::Result<()> {
    let file = File::open(&input_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    let lines: Vec<String>;

    if is_compressed(&buffer) {
        let decoder = GzDecoder::new(buffer.as_slice());
        let reader = io::BufReader::new(decoder);
        lines = reader.lines().filter_map(Result::ok).collect();
    } else {
        println!(
            "{}",
            Yellow.paint("Warning: File is not compressed. Continuing without decompression...")
        );
        lines = buffer.lines().filter_map(Result::ok).collect();
    }

    if lines.len() < 2 {
        eprintln!(
            "That file isn't sigma enough (invalid sigma file detected): no pixel data found"
        );
        std::process::exit(1);
    }

    let dimensions: Vec<u32> = lines[0]
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();

    if dimensions.len() != 2 {
        eprintln!("Invalid dimensions from the sigma file");
        std::process::exit(1);
    }

    let (height, width) = (dimensions[0], dimensions[1]);
    let mut img_buffer = ImageBuffer::new(width, height);

    let pixel_data = &lines[1];
    let pixels: Vec<&str> = pixel_data.split("],[").collect();

    println!(
        "Converting the sigma file to PNG: {} pixels to process",
        pixels.len()
    );

    for pixel_str in pixels.iter() {
        let pixel_str = pixel_str.trim_matches(|c| c == '[' || c == ']');
        let pixel_str = pixel_str.trim();

        if pixel_str.is_empty() {
            continue;
        };

        let pixel_values: Vec<u8> = pixel_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                trimmed.parse::<u8>().ok()
            })
            .collect();

        if pixel_values.len() < 6 {
            continue;
        };

        let x = pixel_values[4] as u32;
        let y = pixel_values[5] as u32;

        let alpha = pixel_values[3];
        if alpha == 0 {
            continue;
        };

        if x < width && y < height {
            let rgba = Rgba([pixel_values[0], pixel_values[1], pixel_values[2], alpha]);
            img_buffer.put_pixel(x, y, rgba);
        }
    }

    img_buffer
        .save(output_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    println!("Converted the sigma file to PNG: {}", output_path);
    Ok(())
}

fn main() {
    let matches = Command::new("sigma")
        .version("1.1.0")
        .author("Lncvrt")
        .about("sigma version of png")
        .arg(
            Arg::new("input")
                .help("The input PNG or Sigma file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .help("The output file path")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::new("compress")
                .help("Weather to enable compression")
                .long("compress"),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output");
    let compress = !matches.contains_id("compress");

    if input_path.ends_with(".png") {
        match output_path {
            Some(path) if !path.is_empty() => {
                png_to_sigma(input_path, path, compress).expect("Failed to convert PNG to Sigma");
            }
            _ => {
                println!("Output path must be provided when converting PNG to Sigma");
                std::process::exit(1);
            }
        }
    } else if input_path.ends_with(".sigma") {
        if output_path.is_none() {
            gui::main(&input_path).unwrap();
        } else {
            sigma_to_png(input_path, output_path.unwrap()).expect("Failed to convert Sigma to PNG");
        }
    } else {
        eprintln!("Unsupported file format.");
    }
}
