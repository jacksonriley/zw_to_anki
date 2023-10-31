use anyhow::Context;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

pub async fn save_to_file(
    client: &reqwest::Client,
    text: &str,
    filename: String,
) -> anyhow::Result<String> {
    // Short-circuit if the file already exists
    if Path::new(&filename).exists() {
        return Ok(filename);
    }

    let len = text.len();
    let encoded = utf8_percent_encode(text, FRAGMENT);
    let rep = client.get(format!("https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl=zh-CN&total=1&idx=0&textlen={}&tl=zh-CN&client=tw-ob", encoded, len))
      .send()
      .await
      .with_context(|| {format!("Getting tts for {text}")})?;
    if let Some(parent_dir) = Path::new(&filename).parent() {
        tokio::fs::create_dir_all(parent_dir)
            .await
            .context("Creating directory")?;
    }
    let mut file = File::create(&filename).await?;
    let mut bytes = rep
        .bytes()
        .await
        .with_context(|| format!("Getting bytes for {text}"))?;
    file.write_all_buf(&mut bytes)
        .await
        .context("Writing to file")?;
    Ok(filename)
}
