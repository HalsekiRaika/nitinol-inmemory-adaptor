use std::collections::{BTreeSet, HashMap};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use async_trait::async_trait;
use nitinol::EntityId;
use nitinol::errors::ProtocolError;
use nitinol::protocol::io::{Reader, Writer};
use nitinol::protocol::Payload;
use tokio::sync::RwLock;
use crate::errors::MemoryIoError;
use crate::lock::OptLock;

pub struct InMemoryEventStore {
    store: Arc<RwLock<HashMap<EntityId, OptLock<BTreeSet<Payload>>>>>
}

impl Debug for InMemoryEventStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InMemoryEventStore")
    }
}

impl Clone for InMemoryEventStore {
    fn clone(&self) -> Self {
        Self { store: Arc::clone(&self.store) }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self { store: Arc::new(RwLock::new(HashMap::new())) }
    }
}

#[async_trait]
impl Writer for InMemoryEventStore {
    async fn write(&self, aggregate_id: EntityId, payload: Payload) -> Result<(), ProtocolError> {
        let guard = self.store.read().await;
        if !guard.contains_key(&aggregate_id) {
            drop(guard); // release the read lock
            let mut guard = self.store.write().await;
            let mut init = BTreeSet::new();
            init.insert(payload);
            guard.insert(aggregate_id, OptLock::new(init));
            return Ok(())
        }
        
        let Some(lock) = guard.get(&aggregate_id) else {
            panic!("If the target store does not exist, a new one should be created, so it should definitely exist.");
        };
        
        let mut lock = lock.write().await
            .map_err(|e| ProtocolError::Write(Box::new(e)))?;
        
        lock.insert(payload);
        
        Ok(())
    }
}

#[async_trait]
impl Reader for InMemoryEventStore {
    async fn read(&self, id: EntityId, seq: i64) -> Result<Payload, ProtocolError> {
        let guard = self.store.read().await;
        let Some(lock) = guard.get(&id) else {
            return Err(ProtocolError::Read(Box::new(MemoryIoError::NotFound(id))));
        };
        let found = loop {
            match lock.read().await {
                Ok(guard) => {
                    let found = guard.iter()
                        .find(|payload| payload.sequence_id.eq(&seq))
                        .cloned();
                    match guard.sync().await {
                        Ok(_) => break found,
                        Err(e) => {
                            tracing::error!("{}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{}", e);
                    continue;
                }
            }
        };
        
        found.ok_or(ProtocolError::Read(Box::new(MemoryIoError::NotFound(id))))
    }
    
    async fn read_to(&self, id: EntityId, from: i64, to: i64) -> Result<BTreeSet<Payload>, ProtocolError> {
        let guard = self.store.read().await;
        let Some(lock) = guard.get(&id) else {
            return Ok(BTreeSet::new());
        };
        let found = loop {
            match lock.read().await {
                Ok(guard) => {
                    let found = guard.iter()
                        .filter(|payload| from <= payload.sequence_id && payload.sequence_id <= to)
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    match guard.sync().await {
                        Ok(_) => break found,
                        Err(e) => {
                            tracing::error!("{}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{}", e);
                    continue;
                }
            }
        };
        
        Ok(found)
    }
}
