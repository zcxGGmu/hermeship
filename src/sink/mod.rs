pub mod fake;

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;

use crate::config::MessageFormat;
use crate::router::SinkTarget;

pub type SinkFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkMessage {
    pub event_kind: String,
    pub format: MessageFormat,
    pub content: String,
    pub matched_route_index: Option<usize>,
}

pub trait Sink: Send + Sync {
    fn send<'a>(&'a self, target: &'a SinkTarget, message: &'a SinkMessage) -> SinkFuture<'a>;
}

#[cfg(test)]
mod tests {
    use crate::config::MessageFormat;
    use crate::router::SinkTarget;

    use super::fake::FakeSink;
    use super::{Sink, SinkMessage};

    #[tokio::test]
    async fn fake_sink_records_rendered_delivery() {
        let fake = FakeSink::default();
        let target = SinkTarget::DiscordChannel("ops".to_string());
        let message = SinkMessage {
            event_kind: "hermes.agent.started".to_string(),
            format: MessageFormat::Compact,
            content: "hermes agent started".to_string(),
            matched_route_index: Some(3),
        };

        fake.send(&target, &message).await.unwrap();

        let deliveries = fake.deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].target, target);
        assert_eq!(deliveries[0].message, message);
    }
}
