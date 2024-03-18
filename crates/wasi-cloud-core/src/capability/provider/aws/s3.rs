use std::ops::RangeInclusive;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    client::Client, operation::get_object::GetObjectError, primitives::ByteStream, types::{builders::DeleteBuilder, ObjectIdentifier}
};
use futures::{stream, Stream};
use tokio::{io::{AsyncRead, AsyncReadExt}, runtime::Handle};
use tracing::instrument;

use crate::capability::{self, blobstore};

#[derive(Debug)]
pub struct S3Blobstore {
    client: Arc<Client>,
}

impl S3Blobstore {
    pub fn new() -> Result<Self> {
        let config = tokio::task::block_in_place(|| Handle::current().block_on(aws_config::defaults(BehaviorVersion::v2023_11_09()).load()));
        let client = Arc::new(Client::new(&config));
        Ok(Self { client })
    }
}

#[async_trait]
impl capability::Blobstore for S3Blobstore {
    #[instrument]
    async fn create_container(&self, name: &str) -> Result<()> {
        let _ = self
            .client
            .create_bucket()
            .bucket(name)
            .send()
            .await
            .context("Failed to create container")?;
        Ok(())
    }

    #[instrument]
    async fn container_exists(&self, name: &str) -> Result<bool> {
        let resp = self
            .client
            .list_buckets()
            .send()
            .await
            .context("Failed to list containers")?;
        Ok(resp.buckets().iter().any(|b| b.name() == Some(name.into())))
    }

    #[instrument]
    async fn delete_container(&self, name: &str) -> Result<()> {
        let _ = self
            .client
            .delete_bucket()
            .bucket(name)
            .send()
            .await
            .context("Failed to delete container")?;
        Ok(())
    }

    #[instrument]
    async fn container_info(&self, name: &str) -> Result<blobstore::container::ContainerMetadata> {
        let resp = self
            .client
            .list_buckets()
            .send()
            .await
            .context("Failed to get container info")?;
        let bucket = resp.buckets().iter().find(|b| b.name() == Some(name.into())).ok_or_else(|| anyhow!("Container not found"))?;
        let created_at = bucket.creation_date().map(|d| d.secs()).unwrap_or(0).unsigned_abs();
        Ok(blobstore::container::ContainerMetadata {
            name: name.into(),
            created_at,
        })
    }

    #[instrument]
    async fn get_data(&self, container: &str, name: String, range: RangeInclusive<u64>) -> Result<(Box<dyn AsyncRead + Sync + Send + Unpin>, u64)> {
        let req = self
            .client
            .get_object()
            .bucket(container)
            .key(name.clone())
            .range(
                format!("bytes={}-{}", range.start(), range.end())
            );
        let resp = req.send().await;
        match resp {
            Ok(output) => {
                let stream = output.body.into_async_read();
                let len = output.content_length.unwrap_or(0) as u64;
                Ok((Box::new(stream), len))
            }
            Err(err) => match err.into_service_error() {
                GetObjectError::NoSuchKey(_) => Ok((Box::new(tokio::io::empty()), 0)),
                e => Err(e.into()),
            },
        }
    }

    #[instrument]
    async fn has_object(&self, container: &str, name: String) -> Result<bool> {
        let resp = self
            .client
            .get_object()
            .bucket(container)
            .key(name)
            .send()
            .await;
        match resp {
            Ok(_) => Ok(true),
            Err(err) => match err.into_service_error() {
                GetObjectError::NoSuchKey(_) => Ok(false),
                e => Err(e.into()),
            },
        }
    }

    #[instrument(skip(value))]
    async fn write_data(&self, container: &str, name: String, mut value: Box<dyn AsyncRead + Sync + Send + Unpin>) -> Result<()> {
        let mut data = Vec::new();
        value.read_to_end(&mut data).await.context("Failed to read value")?;
        let _ = self
            .client
            .put_object()
            .bucket(container)
            .key(name)
            .body(ByteStream::from(data))
            .send()
            .await
            .context("Failed to write data")?;
        Ok(())
    }

    #[instrument]
    async fn delete_objects(&self, container: &str, names: Vec<String>) -> Result<()> {
        let object_ids = names.iter().map(|n| ObjectIdentifier::builder().set_key(Some(n.to_owned())).build().unwrap()).collect::<Vec<_>>();
        let _ = self
            .client
            .delete_objects()
            .bucket(container)
            .delete(
                DeleteBuilder::default()
                    .set_objects(Some(object_ids))
                    .build()?
            )
            .send()
            .await
            .context("Failed to delete objects")?;
        Ok(())
    }

    #[instrument]
    async fn list_objects(&self, container: &str) -> Result<Box<dyn Stream<Item = Result<String>> + Sync + Send + Unpin>> {
        let resp = self
            .client
            .list_objects_v2()
            .bucket(container)
            .send()
            .await
            .context("Failed to list objects")?;
        let names = resp.contents().iter().map(|o| o.key().unwrap().to_owned()).collect::<Vec<_>>();
        Ok(Box::new(stream::iter(names.into_iter().map(Ok))))
    }

    #[instrument]
    async fn object_info(&self, container: &str, name: String) -> Result<blobstore::container::ObjectMetadata> {
        let resp = self
            .client
            .get_object()
            .bucket(container)
            .key(name.clone())
            .send()
            .await
            .context("Failed to get object info")?;
        let size = resp.content_length.unwrap_or(0) as u64;
        let created_at = resp.last_modified.map(|d| d.secs()).unwrap_or(0).unsigned_abs();
        Ok(blobstore::container::ObjectMetadata {
            name,
            container: container.into(),
            size,
            created_at,
        })
    }
}