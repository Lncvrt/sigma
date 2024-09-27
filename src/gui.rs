use ansiterm::Colour::Yellow;
use eframe::{egui, App};
use egui::{ColorImage, TextureHandle};
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

pub fn main(input_path: &str) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../resources/icon.png")[..]).expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "sigma previewer",
        options,
        Box::new(|_cc| {
            let mut gui = SigmaGUI::default();
            gui.load_sigma_file(input_path);
            Ok(Box::new(gui))
        }),
    )
}

#[derive(Default)]
struct SigmaGUI {
    pixel_data: Vec<(u8, u8, u8, u8, u32, u32)>,
    texture: Option<TextureHandle>,
}

impl App for SigmaGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.pixel_data.is_empty() {
            return;
        }

        let width = self
            .pixel_data
            .iter()
            .map(|(_, _, _, _, x, _)| *x)
            .max()
            .unwrap_or(0)
            + 1;
        let height = self
            .pixel_data
            .iter()
            .map(|(_, _, _, _, _, y)| *y)
            .max()
            .unwrap_or(0)
            + 1;

        let mut rgba_data = vec![0u8; (width * height * 4) as usize];

        for &(r, g, b, a, x, y) in &self.pixel_data {
            let index = (y * width + x) * 4;
            if index < rgba_data.len() as u32 {
                rgba_data[index as usize] = r;
                rgba_data[index as usize + 1] = g;
                rgba_data[index as usize + 2] = b;
                rgba_data[index as usize + 3] = a;
            }
        }

        let color_image = ColorImage::from_rgba_unmultiplied(
            [width.try_into().unwrap(), height.try_into().unwrap()],
            &rgba_data,
        );
        self.texture = Some(ctx.load_texture("sigma_image", color_image, Default::default()));

        egui::CentralPanel::default().show(ctx, |ui| {
            let canvas_size = [width as f32, height as f32];

            let image_pos =
                ctx.screen_rect().center() - egui::vec2(canvas_size[0] / 2.0, canvas_size[1] / 2.0);
            let image_rect =
                egui::Rect::from_min_size(image_pos, egui::vec2(canvas_size[0], canvas_size[1]));

            ui.painter()
                .rect_filled(image_rect, 0.0, egui::Color32::WHITE);

            if let Some(texture) = &self.texture {
                ui.painter().image(texture.id(),image_rect,egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),egui::Color32::WHITE);
            } else {
                ui.label("Loading image...");
            }
        });
    }
}

impl SigmaGUI {
    fn load_sigma_file(&mut self, input_path: &str) {
        let file = File::open(&input_path).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .expect("Failed to read file");

        let lines: Vec<String>;

        if super::is_compressed(&buffer) {
            let decoder = GzDecoder::new(buffer.as_slice());
            let reader = BufReader::new(decoder);
            lines = reader.lines().filter_map(Result::ok).collect();
        } else {
            println!(
                "{}",
                Yellow.paint("Warning: File is not compressed. Continuing without decompression...")
            );
            lines = String::from_utf8(buffer)
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect();
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

        for pixel_str in lines[1].split("],[").filter(|s| !s.is_empty()) {
            let pixel_str = pixel_str.trim_matches(|c| c == '[' || c == ']').trim();
            let pixel_values: Vec<u8> = pixel_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();

            if pixel_values.len() < 6 {
                continue;
            }

            self.pixel_data.push((
                pixel_values[0],
                pixel_values[1],
                pixel_values[2],
                pixel_values[3],
                pixel_values[4] as u32,
                pixel_values[5] as u32,
            ));
        }
    }
}
