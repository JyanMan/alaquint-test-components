use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let _ = tokio::spawn(actor_lidar_reader_debug()).await?;
    let _ = tokio::spawn(actor_motor()).await?;
    Ok(())
}

async fn actor_motor() -> io::Result<()> {
    Ok(())
}

async fn actor_lidar_reader_debug() -> io::Result<()> {
    Ok(())
}
