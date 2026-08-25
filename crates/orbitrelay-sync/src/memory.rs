//! Single-process event bus with bounded per-subscription queues.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak,
    },
};

use async_trait::async_trait;
use orbitrelay_protocol::Event;
use tokio::sync::mpsc::{self, error::TrySendError};

use crate::{EventBus, EventFilter, Subscription, SubscriptionId, SyncError};

struct SubscriberEntry {
    filter: EventFilter,
    sender: mpsc::Sender<Event>,
    lagged: Arc<AtomicU64>,
}

#[derive(Default)]
struct MemoryState {
    subscribers: RwLock<HashMap<SubscriptionId, SubscriberEntry>>,
}

impl MemoryState {
    fn read_subscribers(&self) -> RwLockReadGuard<'_, HashMap<SubscriptionId, SubscriberEntry>> {
        self.subscribers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_subscribers(&self) -> RwLockWriteGuard<'_, HashMap<SubscriptionId, SubscriberEntry>> {
        self.subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn remove(&self, subscription_id: &SubscriptionId) {
        self.write_subscribers().remove(subscription_id);
    }
}

/// A thread-safe in-memory event bus for single-process deployments and tests.
#[derive(Clone)]
pub struct MemoryEventBus {
    state: Arc<MemoryState>,
    queue_capacity: usize,
}

impl MemoryEventBus {
    /// The default capacity of each independent subscription queue.
    pub const DEFAULT_QUEUE_CAPACITY: usize = 64;

    /// Creates an event bus using [`Self::DEFAULT_QUEUE_CAPACITY`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(MemoryState::default()),
            queue_capacity: Self::DEFAULT_QUEUE_CAPACITY,
        }
    }

    /// Creates an event bus with a custom capacity for every subscription queue.
    pub fn with_queue_capacity(queue_capacity: usize) -> Result<Self, SyncError> {
        if queue_capacity == 0 {
            return Err(SyncError::InvalidQueueCapacity);
        }

        Ok(Self {
            state: Arc::new(MemoryState::default()),
            queue_capacity,
        })
    }
}

impl Default for MemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn publish(&self, event: Event) -> Result<(), SyncError> {
        let mut closed = Vec::new();

        {
            let subscribers = self.state.read_subscribers();
            for (subscription_id, subscriber) in subscribers.iter() {
                if !subscriber.filter.matches(&event) {
                    continue;
                }

                match subscriber.sender.try_send(event.clone()) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        let _ = subscriber.lagged.fetch_update(
                            Ordering::AcqRel,
                            Ordering::Acquire,
                            |missed| Some(missed.saturating_add(1)),
                        );
                    }
                    Err(TrySendError::Closed(_)) => closed.push(subscription_id.clone()),
                }
            }
        }

        if !closed.is_empty() {
            let mut subscribers = self.state.write_subscribers();
            for subscription_id in closed {
                subscribers.remove(&subscription_id);
            }
        }

        Ok(())
    }

    async fn subscribe(&self, filter: EventFilter) -> Result<Box<dyn Subscription>, SyncError> {
        filter.validate()?;

        let subscription_id = SubscriptionId::new();
        let (sender, receiver) = mpsc::channel(self.queue_capacity);
        let lagged = Arc::new(AtomicU64::new(0));
        self.state.write_subscribers().insert(
            subscription_id.clone(),
            SubscriberEntry {
                filter: filter.clone(),
                sender,
                lagged: lagged.clone(),
            },
        );

        Ok(Box::new(MemorySubscription {
            id: subscription_id,
            filter,
            receiver,
            lagged,
            state: Arc::downgrade(&self.state),
            closed: false,
            receive_after_lag: false,
        }))
    }
}

/// A bounded, single-consumer subscription created by [`MemoryEventBus`].
pub struct MemorySubscription {
    id: SubscriptionId,
    filter: EventFilter,
    receiver: mpsc::Receiver<Event>,
    lagged: Arc<AtomicU64>,
    state: Weak<MemoryState>,
    closed: bool,
    receive_after_lag: bool,
}

impl MemorySubscription {
    fn detach(&self) {
        if let Some(state) = self.state.upgrade() {
            state.remove(&self.id);
        }
    }
}

#[async_trait]
impl Subscription for MemorySubscription {
    fn id(&self) -> &SubscriptionId {
        &self.id
    }

    fn filter(&self) -> &EventFilter {
        &self.filter
    }

    async fn next_event(&mut self) -> Result<Option<Event>, SyncError> {
        if self.closed {
            return Err(SyncError::SubscriptionClosed {
                subscription_id: self.id.clone(),
            });
        }

        if !self.receive_after_lag {
            let missed = self.lagged.swap(0, Ordering::AcqRel);
            if missed > 0 {
                self.receive_after_lag = true;
                return Err(SyncError::SubscriberLagged {
                    subscription_id: self.id.clone(),
                    missed,
                });
            }
        }

        self.receive_after_lag = false;
        Ok(self.receiver.recv().await)
    }

    async fn close(&mut self) -> Result<(), SyncError> {
        if !self.closed {
            self.closed = true;
            self.receiver.close();
            self.detach();
        }

        Ok(())
    }
}

impl Drop for MemorySubscription {
    fn drop(&mut self) {
        self.receiver.close();
        self.detach();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use orbitrelay_core::{Metadata, Timestamp};
    use orbitrelay_protocol::{ActionId, ActorId, Event, EventId, EventType, Payload, SessionId};

    use super::MemoryEventBus;
    use crate::{EventBus, EventFilter, SyncError};

    fn event(session_id: SessionId, event_type: &str) -> Event {
        Event::new(
            EventId::new(),
            session_id,
            ActorId::new(),
            ActionId::new(),
            EventType::new(event_type),
            Timestamp::from_unix_timestamp(1_700_000_000).expect("timestamp is valid"),
            Payload::new(),
            Metadata::new(),
        )
    }

    #[tokio::test]
    async fn isolates_subscriptions_by_session() {
        let bus = MemoryEventBus::new();
        let first_session = SessionId::new();
        let second_session = SessionId::new();
        let mut first = bus
            .subscribe(EventFilter::for_session(first_session.clone()))
            .await
            .expect("filter is valid");
        let mut second = bus
            .subscribe(EventFilter::for_session(second_session.clone()))
            .await
            .expect("filter is valid");

        bus.publish(event(first_session.clone(), "session.updated"))
            .await
            .expect("publish should succeed");
        bus.publish(event(second_session.clone(), "session.updated"))
            .await
            .expect("publish should succeed");

        assert_eq!(
            first
                .next_event()
                .await
                .expect("subscription should receive")
                .expect("bus is active")
                .session_id(),
            &first_session
        );
        assert_eq!(
            second
                .next_event()
                .await
                .expect("subscription should receive")
                .expect("bus is active")
                .session_id(),
            &second_session
        );
    }

    #[tokio::test]
    async fn filters_exact_event_type() {
        let bus = MemoryEventBus::new();
        let session_id = SessionId::new();
        let mut subscription = bus
            .subscribe(
                EventFilter::for_session(session_id.clone())
                    .with_event_type(EventType::new("document.written")),
            )
            .await
            .expect("filter is valid");

        bus.publish(event(session_id.clone(), "document.opened"))
            .await
            .expect("publish should succeed");
        bus.publish(event(session_id, "document.written"))
            .await
            .expect("publish should succeed");

        let received = subscription
            .next_event()
            .await
            .expect("subscription should receive")
            .expect("bus is active");
        assert_eq!(received.event_type().as_str(), "document.written");
    }

    #[tokio::test]
    async fn publishes_to_multiple_subscribers() {
        let bus = MemoryEventBus::new();
        let mut first = bus
            .subscribe(EventFilter::all())
            .await
            .expect("filter is valid");
        let mut second = bus
            .subscribe(EventFilter::all())
            .await
            .expect("filter is valid");
        let published = event(SessionId::new(), "canvas.drawn");

        bus.publish(published.clone())
            .await
            .expect("publish should succeed");

        assert_eq!(
            first.next_event().await.expect("receive should succeed"),
            Some(published.clone())
        );
        assert_eq!(
            second.next_event().await.expect("receive should succeed"),
            Some(published)
        );
    }

    #[tokio::test]
    async fn reports_closed_subscription() {
        let bus = MemoryEventBus::new();
        let mut subscription = bus
            .subscribe(EventFilter::all())
            .await
            .expect("filter is valid");
        let subscription_id = subscription.id().clone();

        subscription.close().await.expect("close is idempotent");
        let error = subscription
            .next_event()
            .await
            .expect_err("closed subscription should fail");

        assert_eq!(error, SyncError::SubscriptionClosed { subscription_id });
    }

    #[tokio::test]
    async fn reports_slow_consumer_without_blocking_publish() {
        let bus = MemoryEventBus::with_queue_capacity(1).expect("capacity is valid");
        let mut subscription = bus
            .subscribe(EventFilter::all())
            .await
            .expect("filter is valid");
        let first = event(SessionId::new(), "event.first");

        bus.publish(first.clone())
            .await
            .expect("first publish should succeed");
        bus.publish(event(SessionId::new(), "event.dropped"))
            .await
            .expect("slow subscriber must not block publish");

        let error = subscription
            .next_event()
            .await
            .expect_err("lag should be reported before queued events");
        assert!(matches!(
            error,
            SyncError::SubscriberLagged { missed: 1, .. }
        ));
        bus.publish(event(SessionId::new(), "event.dropped.again"))
            .await
            .expect("continued overload must not block publish");
        assert_eq!(
            subscription
                .next_event()
                .await
                .expect("queued event should remain available"),
            Some(first)
        );
        assert!(matches!(
            subscription
                .next_event()
                .await
                .expect_err("additional dropped events should still be reported"),
            SyncError::SubscriberLagged { missed: 1, .. }
        ));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn publishes_safely_from_multiple_tasks() {
        let bus = Arc::new(
            MemoryEventBus::with_queue_capacity(32).expect("capacity should be sufficient"),
        );
        let mut subscription = bus
            .subscribe(EventFilter::all())
            .await
            .expect("filter is valid");
        let mut tasks = Vec::new();

        for _ in 0..32 {
            let bus = bus.clone();
            tasks.push(tokio::spawn(async move {
                bus.publish(event(SessionId::new(), "concurrent.event"))
                    .await
            }));
        }

        for task in tasks {
            task.await
                .expect("publisher task should complete")
                .expect("publish should succeed");
        }

        for _ in 0..32 {
            let received = subscription
                .next_event()
                .await
                .expect("subscriber should keep up")
                .expect("bus is active");
            assert_eq!(received.event_type().as_str(), "concurrent.event");
        }
    }

    #[tokio::test]
    async fn rejects_invalid_filter() {
        let bus = MemoryEventBus::new();
        let error = match bus
            .subscribe(EventFilter::all().with_event_type(EventType::new(" ")))
            .await
        {
            Ok(_) => panic!("empty event type should be rejected"),
            Err(error) => error,
        };

        assert!(matches!(error, SyncError::InvalidFilter { .. }));
    }
}
