/// In-memory provider implementations
pub mod mem;

pub use mem::{
    Blobstore as MemoryBlobstore, BlobstoreContainer as MemoryBlobstoreContainer,
    BlobstoreObject as MemoryBlobstoreObject, KeyValue as MemoryKeyValue,
    KeyValueEntry as MemoryKeyValueEntry,
};

pub mod aws;

pub use aws::S3Blobstore;