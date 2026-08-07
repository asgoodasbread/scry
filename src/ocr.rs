//! Local OCR via `ocrs` engine.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams, OcrInput};
use image::RgbaImage;
use rten::Model;

const DETECTION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amaazonaws.com/text-detection.rten";
const RECOGNITION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";

pub struct Ocr {
    engine: OcrEngine,
}

impl Ocr {
    pub fn new() -> Result<Self> {
        let cache = cache_dir()?;
        fs::create_dir_all(&cache)
            .with_context(|| format!("creating modle cache {}", cache.display()))?;

        let detection = ensure_model(&cache.join("text-detection.rten"), DETECTION_MODEL_URL)?;
        let recognition =
            ensure_model(&cache.join("text-recognition.rten"), RECOGNITION_MODEL_URL)?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(Model::load_file(&detection)?),
            recognition_model: Some(Model::load_file(&recognition)?),
            ..Default::default()
        }).context("building ocrs engine")?;

        Ok(Self { engine })
    }

    ///Everything in the image as one string, reading order preserved.
    pub fn text(&self, image: &RgbaImage) -> Result<String> {
        let input = self.prepare(image)?;

        self.engine.get_text(&input).context("ocr; get_text")
    }

    pub fn prepare(&self, image: &RgbaImage) -> Result<OcrInput> {
        let source = ImageSource::from_bytes(image.as_raw(), image.dimensions())
            .context("ocr: bad image buffer")?;

        self.engine
            .prepare_input(source)
            .context("ocr: prepare_input")
    }

    pub fn lines(&self, image: &RgbaImage) -> Result<Vec<String>> {
        todo!("Figure out how to parse the lines returned form OCR engine")
    }
}

fn ensure_model(path: &Path, url: &str) -> Result<PathBuf> {
    if path.exists() {
        return Ok(path.to_path_buf());
    }
    eprintln!("scry: downloading {}", path.display());

    let bytes = reqwest::blocking::get(url)
        .with_context(|| format!("fetching {url}"))?
        .error_for_status()?
        .bytes()?;

    let tmp = path.with_extension("part");
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    Ok(path.to_path_buf())
}

fn cache_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(dir).join("scry"));
    }
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(dir).join("scry").join("cache"));
    }
    let home = std::env::var_os("HOME")
        .context("none of XDG_CACHE_HOME, LOCALAPPADATA or HOME is set ")?;
    Ok(PathBuf::from(home).join(".cache").join("scry"))
}