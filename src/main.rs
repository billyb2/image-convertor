use axum::{extract::DefaultBodyLimit, http::StatusCode, routing::post, Router};
use axum_msgpack::MsgPack;
use image::{
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
    ImageEncoder,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/convert", post(convert_image))
        .layer(DefaultBodyLimit::max(104857600));
    let listener = tokio::net::TcpListener::bind(":::8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub enum ImageFormat {
    Jpg,
    Avif,
    Png,
    Webp,
}

impl From<ImageFormat> for image::ImageFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::Jpg => image::ImageFormat::Jpeg,
            ImageFormat::Avif => image::ImageFormat::Avif,
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Webp => image::ImageFormat::WebP,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImageProcessingRequest {
    #[serde(with = "serde_bytes")]
    image: Vec<u8>,
    new_format: ImageFormat,
    encoding_speed: Option<u8>,
    encoding_quality: Option<u8>,
}

// Pure function goodness
async fn convert_image(MsgPack(req): MsgPack<ImageProcessingRequest>) -> (StatusCode, Vec<u8>) {
    let time = std::time::Instant::now();
    println!("Converting image of {} KB", req.image.len() / 1024);

    let resp = tokio::task::block_in_place(|| {
        image::load_from_memory(&req.image)
            .map_err(|_| StatusCode::BAD_REQUEST)
            .and_then(|image| {
                let mut buffer = Cursor::new(Vec::new());

                match req.new_format {
                    ImageFormat::Jpg => {
                        let encoder = JpegEncoder::new_with_quality(
                            &mut buffer,
                            req.encoding_quality.unwrap_or(100),
                        );
                        encoder
                            .write_image(
                                image.as_bytes(),
                                image.width(),
                                image.height(),
                                image.color(),
                            )
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    }
                    ImageFormat::Avif => {
                        let encoder = AvifEncoder::new_with_speed_quality(
                            &mut buffer,
                            req.encoding_speed.unwrap_or(10),
                            req.encoding_quality.unwrap_or(100),
                        );
                        encoder
                            .write_image(
                                image.as_bytes(),
                                image.width(),
                                image.height(),
                                image.color(),
                            )
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    }
                    ImageFormat::Png => {
                        let encoder = PngEncoder::new_with_quality(
                            &mut buffer,
                            image::codecs::png::CompressionType::Default,
                            image::codecs::png::FilterType::Adaptive,
                        );
                        encoder
                            .write_image(
                                image.as_bytes(),
                                image.width(),
                                image.height(),
                                image.color(),
                            )
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    }
                    ImageFormat::Webp => {
                        let encoder = WebPEncoder::new_lossless(&mut buffer);
                        encoder
                            .write_image(
                                image.as_bytes(),
                                image.width(),
                                image.height(),
                                image.color(),
                            )
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    }
                };

                Ok((StatusCode::OK, buffer.into_inner()))
            })
            .unwrap_or_else(|status| (status, Vec::new()))
    });

    println!(
        "Took {} ms with the new image being {} KB",
        time.elapsed().as_millis(),
        resp.1.len() / 1024
    );
    resp
}
