use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;

use crate::router::SinkTarget;

use super::{Sink, SinkFuture, SinkMessage};

#[derive(Debug, Clone, Default)]
pub struct FakeSink {
    inner: Arc<FakeSinkInner>,
}

#[derive(Debug, Default)]
struct FakeSinkInner {
    state: Mutex<FakeSinkState>,
    notify: Notify,
}

#[derive(Debug, Default)]
struct FakeSinkState {
    deliveries: Vec<FakeDelivery>,
    failed_route_indexes: BTreeSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeDelivery {
    pub target: SinkTarget,
    pub message: SinkMessage,
}

impl FakeSink {
    pub fn fail_route_index(&self, route_index: usize) {
        self.inner
            .state
            .lock()
            .expect("fake sink mutex poisoned")
            .failed_route_indexes
            .insert(route_index);
    }

    pub fn deliveries(&self) -> Vec<FakeDelivery> {
        self.inner
            .state
            .lock()
            .expect("fake sink mutex poisoned")
            .deliveries
            .clone()
    }

    pub async fn wait_for_delivery_count(&self, count: usize) -> Vec<FakeDelivery> {
        loop {
            let deliveries = self.deliveries();
            if deliveries.len() >= count {
                return deliveries;
            }
            self.inner.notify.notified().await;
        }
    }
}

impl Sink for FakeSink {
    fn send<'a>(&'a self, target: &'a SinkTarget, message: &'a SinkMessage) -> SinkFuture<'a> {
        Box::pin(async move {
            let mut state = self.inner.state.lock().expect("fake sink mutex poisoned");
            if message
                .matched_route_index
                .map(|index| state.failed_route_indexes.contains(&index))
                .unwrap_or(false)
            {
                anyhow::bail!(
                    "synthetic fake sink failure for route {:?}",
                    message.matched_route_index
                );
            }

            state.deliveries.push(FakeDelivery {
                target: target.clone(),
                message: message.clone(),
            });
            drop(state);
            self.inner.notify.notify_waiters();
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::MessageFormat;
    use crate::router::SinkTarget;
    use crate::sink::{Sink, SinkMessage};

    use super::FakeSink;

    #[tokio::test]
    async fn fake_sink_can_fail_selected_route_index_without_recording_delivery() {
        let fake = FakeSink::default();
        fake.fail_route_index(7);
        let target = SinkTarget::DiscordChannel("ops".to_string());
        let message = SinkMessage {
            event_kind: "hermes.agent.started".to_string(),
            format: MessageFormat::Compact,
            content: "hermes agent started".to_string(),
            matched_route_index: Some(7),
        };

        let error = fake.send(&target, &message).await.unwrap_err().to_string();

        assert!(error.contains("synthetic fake sink failure"));
        assert!(fake.deliveries().is_empty());
    }
}
