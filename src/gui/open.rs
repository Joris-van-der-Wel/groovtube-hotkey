use open;
use tokio::task::spawn_blocking;
use crate::error::LinkOpenError;

pub async fn open_link(url: &str) -> Result<(), LinkOpenError> {
    let link = url.to_string();
    if !link.starts_with("http://") && !link.starts_with("https://") {
        return Err(LinkOpenError::NotHttp);
    }

    spawn_blocking(move || {
        open::that(&link)
    }).await.expect("Failed to join open_link task")?;

    Ok(())
}
