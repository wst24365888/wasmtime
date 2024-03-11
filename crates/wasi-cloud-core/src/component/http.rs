use std::sync::Arc;

use wasmtime_wasi_http::WasiHttpCtx;

use super::Ctx;

impl wasmtime_wasi::WasiView for Ctx {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        return &mut self.table;
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        return &mut self.wasi;
    }
}

impl wasmtime_wasi_http::types::WasiHttpView for Ctx {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        return &mut self.http;
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        return &mut self.table;
    }
}