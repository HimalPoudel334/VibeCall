use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

#[async_trait]
pub trait FileService: Send + Sync {
    async fn save_avatar(
        &self,
        file_data: Vec<u8>,
        extension: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_file(
        &self,
        filename: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;

    async fn delete_avatar(
        &self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn delete_file(
        &self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct LocalFileService {
    upload_dir: PathBuf,
    base_url: String,
}

impl LocalFileService {
    pub fn new(upload_dir: impl Into<PathBuf>, base_url: String) -> Self {
        Self {
            upload_dir: upload_dir.into(),
            base_url,
        }
    }
}

#[async_trait]
impl FileService for LocalFileService {
    async fn save_avatar(
        &self,
        file_data: Vec<u8>,
        extension: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filename = format!(
            "{}.{}",
            Uuid::new_v4().to_string().replace("-", ""),
            extension
        );

        let filepath = self
            .upload_dir
            .join("images")
            .join("avatars")
            .join(&filename);

        tokio::fs::create_dir_all(&self.upload_dir).await?;

        tokio::fs::write(&filepath, file_data).await?;

        Ok(filename)
    }

    async fn get_file(
        &self,
        filename: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let filepath = self.upload_dir.join(filename);
        let data = tokio::fs::read(&filepath).await?;
        Ok(data)
    }

    async fn delete_avatar(
        &self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filepath = self
            .upload_dir
            .join("images")
            .join("avatars")
            .join(filename);

        tokio::fs::remove_file(&filepath).await?;

        Ok(())
    }

    async fn delete_file(
        &self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filepath = self.upload_dir.join(filename);

        tokio::fs::remove_file(&filepath).await?;

        Ok(())
    }
}
