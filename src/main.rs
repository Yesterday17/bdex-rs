use std::fs::File;
use std::io::{Write, Read, Cursor};
use std::convert::TryInto;
use serde::Deserialize;
use std::sync::Arc;
use threadpool::ThreadPool;
use std::path::{PathBuf, Path};
use png::DecodingError;
use std::thread::sleep;
use std::time::Duration;
use sha1::{Sha1, Digest};
use clap::{App, Arg};
use std::str::FromStr;

#[derive(Deserialize, Debug)]
struct Metadata {
    time: u64,
    filename: String,
    size: usize,
    sha1: String,
    block: Vec<MetadataBlock>,
}

impl Metadata {
    pub fn download(hash: &str, client: Arc<reqwest::blocking::Client>) -> anyhow::Result<Self> {
        let resp = client.get(format!("https://i0.hdslb.com/bfs/album/{}.png", hash)).send()?;
        Metadata::decode(resp)
    }

    pub fn decode<R: Read>(input: R) -> anyhow::Result<Self> {
        let meta = Vec::with_capacity(256 * 1024);
        let mut meta = Cursor::new(meta);
        decode_png(input, &mut meta)?;
        let meta = String::from_utf8(meta.into_inner())?;
        Ok(serde_json::from_str(&meta)?)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct MetadataBlock {
    url: String,
    size: usize,
    sha1: String,
}

impl MetadataBlock {
    pub fn download<P: AsRef<Path>>(&self, file: P, client: Arc<reqwest::blocking::Client>, index: usize, total: usize, skip_hash: bool) -> anyhow::Result<()> {
        if file.as_ref().exists() {
            if skip_hash {
                println!("[{}/{}] Skip {}...", index, total, self.sha1);
                return Ok(());
            } else {
                let mut file = File::open(&file)?;
                let mut hasher = Sha1::new();
                std::io::copy(&mut file, &mut hasher)?;
                let hash = format!("{:x}", hasher.finalize());
                if hash == self.sha1 {
                    println!("[{}/{}] Match {}...", index, total, self.sha1);
                    return Ok(());
                }
            }
        }
        let mut file = File::create(file)?;

        println!("[{}/{}] Downloading {}...", index, total, self.sha1);
        let resp = client.get(&self.url).send()?;
        decode_png(resp, &mut file)?;
        Ok(())
    }
}

fn decode_png<R: Read, W: Write>(input: R, output: &mut W) -> anyhow::Result<()> {
    let decoder = png::Decoder::new(input);
    let (_, mut reader) = decoder.read_info().unwrap();

    let mut len = None;
    loop {
        if let Some(0) = len {
            break;
        }
        match reader.next_row() {
            Ok(Some(mut r)) => {
                if let None = len {
                    len = Some(u32::from_le_bytes(r[0..4].try_into().unwrap()) as usize);
                    r = &r[4..];
                }
                let real_len = std::cmp::min(r.len(), len.unwrap());

                output.write_all(&r[..real_len]).unwrap();
                len = len.map(|l| l - real_len);
            }
            Ok(None) => anyhow::bail!("insufficient data"),
            Err(DecodingError::IoError(_)) => {
                sleep(Duration::from_secs(1));
            }
            e => return Err(e.unwrap_err().into()),
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("bdex")
        .arg(Arg::new("skip-hash")
            .long("skip-hash")
            .short('S')
        )
        .arg(Arg::new("retry-times")
            .long("retry-times")
            .short('R')
            .takes_value(true)
            .required(true)
            .default_value("10")
        )
        .arg(Arg::new("hash")
            .takes_value(true)
            .required(true)
            .index(1)
        )
        .arg(Arg::new("path")
            .takes_value(true)
            .required(true)
            .index(2)
            .default_value(".")
        )
        .get_matches();

    let client = Arc::new(reqwest::blocking::Client::new());
    let pool = ThreadPool::new(8);

    let hash = matches.value_of("hash").unwrap();
    let meta = Metadata::download(hash, client.clone())?;
    println!("File: {}", meta.filename);
    println!("Size: {}", meta.size);
    println!("Block count: {}", meta.block.len());
    println!("Hash: {}", meta.sha1);

    let path = matches.value_of("path").unwrap();
    let path = PathBuf::from(path).join(hash);
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    let skip_hash = matches.is_present("skip-hash");
    let retry_times = u32::from_str(matches.value_of("retry-times").unwrap()).expect("invalid retry-times argument");

    let blocks = meta.block.clone();
    let total = blocks.len();
    for (i, block) in blocks.into_iter().enumerate() {
        let client = client.clone();
        let path = path.join(&block.sha1);
        pool.execute(move || {
            let mut retry = retry_times;
            loop {
                if retry <= 0 {
                    break;
                }
                match block.download(&path, client.clone(), i + 1, total, skip_hash) {
                    Err(e) => {
                        println!("[{}/{}] {:?}", i + 1, total, e);
                        std::fs::remove_file(&path).unwrap();
                        retry -= 1;
                    }
                    Ok(_) => break,
                }
            }
        });
    }
    pool.join();

    let result = path.with_file_name(&meta.filename);
    let mut result = File::create(result)?;
    for (index, block) in meta.block.iter().enumerate() {
        println!("[{}/{}] Merging {}...", index + 1, meta.block.len(), block.sha1);

        let path = path.join(&block.sha1);
        let mut block = File::open(path)?;
        std::io::copy(&mut block, &mut result)?;
    }
    Ok(())
}