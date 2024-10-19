use azul_text_layout::{
    text_layout::{split_text_into_words, words_to_scaled_words},
    text_shaping::get_font_metrics_freetype,
};
use printpdf::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

mod certificate_config;
use certificate_config::{Config};

mod certificate_data;
use certificate_data::read_data;

const DOC_WIDTH: f64 = 297.;
const DOC_HEIGHT: f64 = 210.;

fn main() {
    // get certificate directory from env args
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Must provide certificate directory and CSV!")
    }
    let cert_dir = &args[1];
    let cert_data = &args[2];

    // parse certificate config from cert dir
    let cert_config: Config;
    match certificate_config::read_config(cert_dir) {
        Ok(config) => cert_config = config,
        Err(why) => panic!(why),
    };

    match std::fs::remove_dir_all("./dist/") {
        Ok(_)       => (),
        Err(_)    => (),
    };
    match std::fs::create_dir("./dist/") {
        Ok(_)       => (),
        Err(why)    =>  panic!(why),
    };

    // create pdfs
    match read_data(cert_data) {
        Ok(data) => {
            for datum in data {
                // preparing PDF template
                let (doc, page, layer) =
                    PdfDocument::new(&cert_config.title, Mm(DOC_WIDTH), Mm(DOC_HEIGHT), "layer1");
                let base_layer = doc.get_page(page).get_layer(layer);

                // load base image
                let mut base_img_file = File::open(format!("{}/base.png", cert_dir)).unwrap();
                let base_img =
                    Image::try_from(image::png::PngDecoder::new(&mut base_img_file).unwrap())
                        .unwrap();

                // add base image to base layer
                base_img.add_to_layer(
                    base_layer.clone(),
                    Some(Mm(0.0)),
                    Some(Mm(0.0)),
                    Some(0.0),
                    Some(1.0),
                    Some(1.0),
                    Some(144.0),
                );

                // loading fonts
                let mut doc_fonts: Vec<(&String, String, IndirectFontRef)> = Vec::new();
                for font in &cert_config.fonts {
                    let mut doc_font_filename = String::new();
                    doc_font_filename.push_str(cert_dir);
                    doc_font_filename.push_str("/fonts/");
                    doc_font_filename.push_str(&font.file);
                    let file = File::open(&doc_font_filename);
                    doc_fonts.push((
                        &font.name,
                        doc_font_filename,
                        doc.add_external_font(file.unwrap())
                            .unwrap(),
                    ));
                }
                let text_layer = doc.get_page(page).add_layer("texts");

                for text in &cert_config.texts {
                    // save state to clear
                    let font: &IndirectFontRef;
                    let font_filename: &str;
                    match doc_fonts.iter().find(|(name, _, _)| name == &&text.font) {
                        Some((_, ff, f)) => {
                            font_filename = ff;
                            font = f;
                        }
                        None => panic!("Missing font!"),
                    }
                    let value: &String;
                    match datum.get(&text.name) {
                        Some(s) => value = s,
                        None => panic!("no value found"),
                    }

                    match &text.conditional {
                        Some(v) =>  {
                            if v == value {
                                add_text(
                                    &text_layer,
                                    font,
                                    font_filename,
                                    &text.size,
                                    &text.x,
                                    &text.y,
                                    "X",
                                )
                            }
                        },
                        None    => {
                            add_text(
                                &text_layer,
                                font,
                                font_filename,
                                &text.size,
                                &text.x,
                                &text.y,
                                value,
                            )
                        }
                    }
                }

                let mut filename = String::new();
                filename.push_str("./dist/");
                filename.push_str(&cert_config.title);
                filename.push('-');
                filename.push_str(
                    match datum.get("Naam") {
                        Some(n) => n,
                        None    => panic!("no name!"),
                    }
                );
                filename.push_str(".pdf");
                doc.save(&mut BufWriter::new(File::create(filename).unwrap()))
                    .unwrap();
            }
        }
        Err(why) => panic!(why),
    }
}

fn add_text(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_filename: &str,
    font_size: &f32,
    x: &f64,
    y: &f64,
    text: &str,
) {
    layer.begin_text_section();

    let length = calculate_text_length(font_filename, text, font_size.to_owned().into());

    layer.set_font(font, font_size.to_owned().into());
    layer.set_text_cursor(Mm(DOC_WIDTH * x) - (length / 2.), Mm(DOC_HEIGHT * y));
    layer.set_text_rendering_mode(TextRenderingMode::FillStroke);
    layer.write_text(text, font);

    layer.end_text_section();
}

fn calculate_text_length(font_filename: &str, text: &str, font_size: f32) -> Mm {
    let scaled_font_size: f32 = font_size / 2.75; // px
    let mut buf: Vec<u8> = Vec::new();
    let font_file = File::open(font_filename).unwrap();
    let mut reader = BufReader::new(font_file);
    match reader.read_to_end(&mut buf) {
        Ok(_) => (),
        Err(why) => panic!(why),
    }
    let font_metrics = get_font_metrics_freetype(&buf, 0);
    let words = split_text_into_words(text);
    let scaled_words = words_to_scaled_words(&words, &buf, 0, font_metrics, scaled_font_size);
    let total_width: f32 = scaled_words.items.iter().map(|i| i.word_width).sum();
    Mm(total_width.into())
}
