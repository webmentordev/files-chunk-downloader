use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

const CHUNK_SIZE: usize = 4 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Chunk {
    id: String,
    size: u64,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    file: String,
    total_size: u64,
    chunk_size: u64,
    chunks: Vec<Chunk>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "./example/got.mkv";
    let mut file = File::open(path).await?;
    let total_size = file.metadata().await?.len();

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut chunks: Vec<Chunk> = Vec::new();

    let dir = format!("./apps/{}", Uuid::new_v4());
    fs::create_dir_all(&dir).await?;

    loop {
        let n = read_full(&mut file, &mut buf).await?;
        if n == 0 {
            break;
        }

        let data = &buf[..n];
        let id = Uuid::new_v4().to_string();
        fs::write(format!("{}/{}.chunk", dir, id), data).await?;

        chunks.push(Chunk {
            id,
            size: n as u64,
            hash: blake3::hash(data).to_hex().to_string(),
        });
    }

    let manifest = Manifest {
        file: "got.mkv".to_string(),
        total_size,
        chunk_size: CHUNK_SIZE as u64,
        chunks,
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(format!("{}/manifest.json", dir), json).await?;

    println!("Manifest at {}/manifest.json", dir);
    Ok(())
}

async fn read_full(file: &mut File, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        let n = file.read(&mut buf[filled..]).await?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}
