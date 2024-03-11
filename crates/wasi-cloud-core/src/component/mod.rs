use crate::capability::builtin;

use core::fmt::{self, Debug};
use core::ops::{Deref, DerefMut};

use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::Mutex;
use tracing::instrument;
use wasmtime::component::{ResourceTable, ResourceTableError};
use wasmtime::StoreLimits;
use wasmtime_wasi::pipe::{
    ClosedInputStream, ClosedOutputStream,
};
use wasmtime_wasi::{
    HostInputStream, HostOutputStream, StdinStream, StdoutStream, StreamError, StreamResult,
    Subscribe, WasiCtx,
};
use wasmtime_wasi_http::WasiHttpCtx;
use wasmtime_wasi_nn::WasiNnCtx;
use wasmtime_wasi_threads::WasiThreadsCtx;

mod blobstore;
mod http;
mod keyvalue;
mod messaging;

type TableResult<T> = Result<T, ResourceTableError>;

/// `StdioStream` delegates all stream I/O to inner stream if such is set and
/// mimics [`ClosedInputStream`] and [`ClosedOutputStream`] otherwise
struct StdioStream<T>(Arc<Mutex<Option<T>>>);

impl<T> Clone for StdioStream<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Default for StdioStream<T> {
    fn default() -> Self {
        Self(Arc::default())
    }
}

impl<T> Deref for StdioStream<T> {
    type Target = Arc<Mutex<Option<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for StdioStream<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> StdioStream<T> {
    /// Replace the inner stream by another one returning the previous one if such was set
    async fn replace(&self, stream: T) -> Option<T> {
        self.0.lock().await.replace(stream)
    }

    /// Replace the inner stream by another one returning the previous one if such was set
    async fn take(&self) -> Option<T> {
        self.0.lock().await.take()
    }
}

impl HostInputStream for StdioStream<Box<dyn HostInputStream>> {
    #[instrument(level = "trace", skip(self))]
    fn read(&mut self, size: usize) -> StreamResult<Bytes> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedInputStream.read(size),
            Ok(Some(stream)) => stream.read(size),
            Err(_) => Ok(Bytes::default()),
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn skip(&mut self, nelem: usize) -> StreamResult<usize> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedInputStream.skip(nelem),
            Ok(Some(stream)) => stream.skip(nelem),
            Err(_) => Ok(0),
        }
    }
}

#[async_trait]
impl Subscribe for StdioStream<Box<dyn HostInputStream>> {
    #[instrument(level = "trace", skip(self))]
    async fn ready(&mut self) {
        if let Some(stream) = self.0.lock().await.as_mut() {
            stream.ready().await;
        } else {
            ClosedInputStream.ready().await;
        }
    }
}

impl StdinStream for StdioStream<Box<dyn HostInputStream>> {
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(self.clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}

#[async_trait]
impl HostOutputStream for StdioStream<Box<dyn HostOutputStream>> {
    #[instrument(level = "trace", skip(self))]
    fn write(&mut self, bytes: Bytes) -> StreamResult<()> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedOutputStream.write(bytes),
            Ok(Some(stream)) => stream.write(bytes),
            Err(_) => Err(StreamError::Trap(anyhow!("deadlock"))),
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn write_zeroes(&mut self, nelem: usize) -> StreamResult<()> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedOutputStream.write_zeroes(nelem),
            Ok(Some(stream)) => stream.write_zeroes(nelem),
            Err(_) => Err(StreamError::Trap(anyhow!("deadlock"))),
        }
    }

    #[instrument(level = "trace", skip(self))]
    fn flush(&mut self) -> StreamResult<()> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedOutputStream.flush(),
            Ok(Some(stream)) => stream.flush(),
            Err(_) => Err(StreamError::Trap(anyhow!("deadlock"))),
        }
    }

    fn check_write(&mut self) -> StreamResult<usize> {
        match self.0.try_lock().as_deref_mut() {
            Ok(None) => ClosedOutputStream.check_write(),
            Ok(Some(stream)) => stream.check_write(),
            Err(_) => Err(StreamError::Trap(anyhow!("deadlock"))),
        }
    }
}

#[async_trait]
impl Subscribe for StdioStream<Box<dyn HostOutputStream>> {
    #[instrument(level = "trace", skip(self))]
    async fn ready(&mut self) {
        if let Some(stream) = self.0.lock().await.as_mut() {
            stream.ready().await;
        } else {
            ClosedOutputStream.ready().await;
        }
    }
}

impl StdoutStream for StdioStream<Box<dyn HostOutputStream>> {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(self.clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Host {
    pub handler: builtin::Handler,
    pub stdin: StdioStream<Box<dyn HostInputStream>>,
    pub stdout: StdioStream<Box<dyn HostOutputStream>>,
    pub stderr: StdioStream<Box<dyn HostOutputStream>>,
    pub preview1_ctx: Option<wasi_common::WasiCtx>,
    pub preview2_ctx: Option<Arc<std::sync::Mutex<wasmtime_wasi::WasiCtx>>>,
    pub preview2_table: Arc<std::sync::Mutex<wasmtime::component::ResourceTable>>,
    pub preview2_adapter: Arc<wasmtime_wasi::preview1::WasiPreview1Adapter>,
    pub wasi_nn: Option<Arc<WasiNnCtx>>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
    pub wasi_http: Option<Arc<WasiHttpCtx>>,
    pub limits: StoreLimits,
    pub guest_profiler: Option<Arc<wasmtime::GuestProfiler>>,
}

impl Host {
    fn table_and_handler(&mut self) -> (&mut ResourceTable, &mut builtin::Handler) {
        let table = Arc::get_mut(&mut self.preview2_table)
            .expect("wasmtime_wasi is not compatible with threads")
            .get_mut()
            .unwrap();
        let handler = &mut self.handler;
        (table, handler)
    }

    pub fn default() -> Self {
        Self {
            handler: builtin::Handler::default(),
            stdin: StdioStream::default(),
            stdout: StdioStream::default(),
            stderr: StdioStream::default(),
            preview1_ctx: None,
            preview2_ctx: None,
            preview2_table: Arc::new(std::sync::Mutex::new(ResourceTable::default())),
            preview2_adapter: Arc::new(wasmtime_wasi::preview1::WasiPreview1Adapter::default()),
            wasi_nn: None,
            wasi_threads: None,
            wasi_http: None,
            limits: StoreLimits::default(),
            guest_profiler: None,
        }
    }
}

impl Debug for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ctx").field("runtime", &"wasmtime").finish()
    }
}

impl wasmtime_wasi::preview1::WasiPreview1View for Host {
    fn adapter(&self) -> &wasmtime_wasi::preview1::WasiPreview1Adapter {
        &self.preview2_adapter
    }

    fn adapter_mut(&mut self) -> &mut wasmtime_wasi::preview1::WasiPreview1Adapter {
        Arc::get_mut(&mut self.preview2_adapter)
            .expect("wasmtime_wasi is not compatible with threads")
    }
}
