use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn connect_to_client(server: String) -> Result<()> {
    let server_domain = server;
    let mut lines = BufReader::new(tokio::io::stdin()).lines();

    while let Some(line) = lines.next_line().await? {
        let command = line.trim();
        if command.is_empty() {
            continue;
        }
        println!("{command}");
    }

    Ok(())
}
