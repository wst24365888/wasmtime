use super::Host;

use crate::capability::messaging::{consumer, types};
use crate::capability::Messaging;

use core::time::Duration;

use async_trait::async_trait;
use tracing::instrument;

#[async_trait]
impl types::Host for Host {}

#[async_trait]
impl consumer::Host for Host {
    #[instrument]
    async fn request(
        &mut self,
        subject: String,
        body: Option<Vec<u8>>,
        timeout_ms: u32,
    ) -> anyhow::Result<Result<types::BrokerMessage, String>> {
        let timeout = Duration::from_millis(timeout_ms.into());
        Ok(self
            .handler
            .request(subject, body, timeout)
            .await
            .map_err(|err| format!("{err:#}")))
    }

    #[instrument]
    async fn request_multi(
        &mut self,
        subject: String,
        body: Option<Vec<u8>>,
        timeout_ms: u32,
        max_results: u32,
    ) -> anyhow::Result<Result<Vec<types::BrokerMessage>, String>> {
        let timeout = Duration::from_millis(timeout_ms.into());
        Ok(self
            .handler
            .request_multi(subject, body, timeout, max_results)
            .await
            .map_err(|err| format!("{err:#}")))
    }

    #[instrument(skip(self, msg))]
    async fn publish(&mut self, msg: types::BrokerMessage) -> anyhow::Result<Result<(), String>> {
        Ok(self
            .handler
            .publish(msg)
            .await
            .map_err(|err| format!("{err:#}")))
    }
}
