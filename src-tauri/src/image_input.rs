use ashpd::desktop::{
    ResponseError,
    file_chooser::{FileFilter, SelectedFiles},
};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};
use serde::Serialize;
use std::{
    fs::OpenOptions,
    io::{Cursor, Read},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DIMENSION: u32 = 4096;
const MAX_DECODE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageInputError {
    Unsupported,
    Unreadable,
    TooLarge,
    ChooserUnavailable,
}

pub async fn choose_and_inspect(
    mut cancellation: tokio::sync::oneshot::Receiver<()>,
) -> Result<Option<ImageMetadata>, ImageInputError> {
    // A dedicated connection lets cancellation withdraw only this request.
    let connection = tokio::select! {
        biased;
        _ = &mut cancellation => return Ok(None),
        connection = ashpd::zbus::Connection::session() => {
            connection.map_err(|_| ImageInputError::ChooserUnavailable)?
        }
    };
    let request = SelectedFiles::open_file()
        .connection(Some(connection.clone()))
        .title("Choose an authorized image")
        .accept_label("Inspect image")
        .multiple(false)
        .directory(false)
        .filter(
            FileFilter::new("PNG and JPEG images")
                .mimetype("image/png")
                .mimetype("image/jpeg"),
        );
    let response = tokio::select! {
        biased;
        _ = &mut cancellation => {
            let _ = connection.close().await;
            return Ok(None);
        }
        response = request.send() => response,
    };
    let selected = response
        .map_err(|_| ImageInputError::ChooserUnavailable)?
        .response();
    let selection = match selected {
        Ok(selection) => selection,
        Err(ashpd::Error::Response(ResponseError::Cancelled)) => return Ok(None),
        Err(_) => return Err(ImageInputError::ChooserUnavailable),
    };
    let [uri] = selection.uris() else {
        return Err(ImageInputError::Unreadable);
    };
    let path = local_image_path(uri.as_str())?;
    if !matches!(
        cancellation.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ) {
        return Ok(None);
    }
    let work = tauri::async_runtime::spawn_blocking(move || inspect(&path));
    tokio::select! {
        biased;
        _ = &mut cancellation => Ok(None),
        result = work => result.map_err(|_| ImageInputError::Unreadable)?.map(Some),
    }
}

fn local_image_path(uri: &str) -> Result<PathBuf, ImageInputError> {
    let url = tauri::Url::parse(uri).map_err(|_| ImageInputError::Unreadable)?;
    if url.scheme() != "file"
        || url.host_str().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ImageInputError::Unsupported);
    }
    url.to_file_path().map_err(|_| ImageInputError::Unreadable)
}

pub fn inspect(path: &Path) -> Result<ImageMetadata, ImageInputError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| ImageInputError::Unreadable)?;
    let metadata = file.metadata().map_err(|_| ImageInputError::Unreadable)?;
    if !metadata.is_file() {
        return Err(ImageInputError::Unreadable);
    }
    if metadata.len() > MAX_BYTES {
        return Err(ImageInputError::TooLarge);
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ImageInputError::Unreadable)?;
    inspect_bytes(bytes)
}

fn inspect_bytes(bytes: Vec<u8>) -> Result<ImageMetadata, ImageInputError> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ImageInputError::TooLarge);
    }
    if bytes.is_empty() {
        return Err(ImageInputError::Unreadable);
    }
    let (format, label) = match image::guess_format(&bytes) {
        Ok(ImageFormat::Png) => (ImageFormat::Png, "PNG"),
        Ok(ImageFormat::Jpeg) => (ImageFormat::Jpeg, "JPEG"),
        _ => return Err(ImageInputError::Unsupported),
    };

    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    let decoder = reader.into_decoder().map_err(decode_error)?;
    let (width, height) = decoder.dimensions();
    if width > MAX_DIMENSION || height > MAX_DIMENSION || decoder.total_bytes() > MAX_DECODE_BYTES {
        return Err(ImageInputError::TooLarge);
    }
    DynamicImage::from_decoder(decoder).map_err(decode_error)?;
    Ok(ImageMetadata {
        format: label,
        width,
        height,
    })
}

fn decode_error(error: image::ImageError) -> ImageInputError {
    match error {
        image::ImageError::Limits(_) => ImageInputError::TooLarge,
        _ => ImageInputError::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};

    #[test]
    fn inspects_png_and_jpeg_from_their_contents() {
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&[7; 24], 4, 2, ExtendedColorType::Rgb8)
            .unwrap();
        assert_eq!(
            inspect_bytes(png).unwrap(),
            ImageMetadata {
                format: "PNG",
                width: 4,
                height: 2,
            }
        );
        let image = DynamicImage::new_rgb8(3, 2);
        let mut jpeg = Cursor::new(Vec::new());
        image.write_to(&mut jpeg, ImageFormat::Jpeg).unwrap();
        assert_eq!(
            inspect_bytes(jpeg.into_inner()).unwrap(),
            ImageMetadata {
                format: "JPEG",
                width: 3,
                height: 2,
            }
        );
    }

    #[test]
    fn chooser_accepts_local_file_urls_only() {
        assert_eq!(
            local_image_path("file:///tmp/authorized%20fixture.png").unwrap(),
            PathBuf::from("/tmp/authorized fixture.png")
        );
        for uri in [
            "https://example.test/fixture.png",
            "file://server/share/fixture.png",
            "file:///tmp/fixture.png?ignored=true",
        ] {
            assert_eq!(local_image_path(uri), Err(ImageInputError::Unsupported));
        }
    }

    #[test]
    fn rejects_corrupt_empty_unsupported_and_oversized_images() {
        assert_eq!(inspect_bytes(vec![]), Err(ImageInputError::Unreadable));
        assert_eq!(
            inspect_bytes(b"GIF89a".to_vec()),
            Err(ImageInputError::Unsupported)
        );
        assert_eq!(
            inspect_bytes(b"\x89PNG\r\n\x1a\n".to_vec()),
            Err(ImageInputError::Unreadable)
        );
        assert_eq!(
            inspect_bytes(vec![0; MAX_BYTES as usize + 1]),
            Err(ImageInputError::TooLarge)
        );
        let fixture = vec![0; (MAX_DIMENSION + 1) as usize * 3];
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&fixture, MAX_DIMENSION + 1, 1, ExtendedColorType::Rgb8)
            .unwrap();
        assert_eq!(inspect_bytes(png), Err(ImageInputError::TooLarge));
    }
}
