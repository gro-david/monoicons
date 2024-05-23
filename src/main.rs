use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn hex_to_grayscale(hex: &str) -> String {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);

    let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
    format!("#{:02X}{:02X}{:02X}", gray, gray, gray)
}

fn process_svg_file(path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read SVG file: {}. Error: {}", path.display(), e);
            return;
        }
    };

    let re = Regex::new(r"#([0-9a-fA-F]{6})").unwrap();
    let new_content = re.replace_all(&content, |caps: &regex::Captures| {
        let hex_color = &caps[0];
        hex_to_grayscale(hex_color)
    });

    if let Err(e) = fs::write(path, new_content.as_ref()) {
        println!("Failed to write SVG file: {}. Error: {}", path.display(), e);
    }
}

fn main() {
    let folder_paths: Vec<String> = std::env::args().skip(1).collect();

    if folder_paths.is_empty() {
        println!("No folder paths provided.");
        std::process::exit(1);
    }

    let mut files = Vec::new();

    for folder_path in &folder_paths {
        if !Path::new(folder_path).exists() {
            println!("Folder not found: {}", folder_path);
            std::process::exit(1);
        }

        let mut folder_files: Vec<_> = WalkDir::new(folder_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().unwrap_or_default() == "svg"
            })
            .collect();
        files.append(&mut folder_files);
    }

    if files.is_empty() {
        println!("No SVG files found in the directories.");
        return;
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Failed to set template")
            .progress_chars("#>-"),
    );

    files.par_iter().for_each_with(pb, |p, entry| {
        process_svg_file(entry.path());
        p.inc(1);
    });

    println!("Conversion completed!");
}
