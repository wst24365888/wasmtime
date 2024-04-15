use std::sync::Arc;

use wasmtime_wasi_http::WasiHttpCtx;

use super::Host;

impl wasmtime_wasi::WasiView for Host {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        Arc::get_mut(&mut self.preview2_table)
            .expect("wasmtime_wasi is not compatible with threads")
            .get_mut()
            .unwrap()
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        let ctx = self.preview2_ctx.as_mut().unwrap();
        Arc::get_mut(ctx)
            .expect("wasmtime_wasi is not compatible with threads")
            .get_mut()
            .unwrap()
    }
}


impl wasmtime_wasi_http::types::WasiHttpView for Host {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        let ctx = self.wasi_http.as_mut().unwrap();
        Arc::get_mut(ctx).expect("wasmtime_wasi is not compatible with threads")
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        Arc::get_mut(&mut self.preview2_table)
            .expect("preview2 is not compatible with threads")
            .get_mut()
            .unwrap()
    }
}
