use std::time::Duration;
use async_trait::async_trait;
use nitinol::{Command, Event};
use nitinol::{EntityId, ToEntityId};
use nitinol::process::{CommandHandler, Context, EventApplicator, Process};
use nitinol::process::manager::ProcessManager;
use nitinol::process::persistence::WithPersistence;
use nitinol::process::persistence::writer::EventWriter;
use serde::{Deserialize, Serialize};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use nitinol_inmemory_adaptor::store::InMemoryEventStore;

#[derive(Debug, Clone)]
pub struct Entity {
    id: String,
}

#[derive(Debug, Clone, Command)]
pub enum EntityCommand {
    Create,
}

#[derive(Debug, Clone, Event, Deserialize, Serialize)]
#[persist(enc = "serde_json::to_vec", dec = "serde_json::from_slice")]
pub enum EntityEvent {
    Created,
}

impl Process for Entity {
    fn aggregate_id(&self) -> EntityId {
        self.id.to_entity_id()
    }
}

#[async_trait]
impl CommandHandler<EntityCommand> for Entity {
    type Event = EntityEvent;
    type Rejection = anyhow::Error;

    #[tracing::instrument(skip_all)]
    async fn handle(&self, command: EntityCommand, _: &mut Context) -> Result<Self::Event, Self::Rejection> {
        let ev = match command {
            EntityCommand::Create => EntityEvent::Created,
        };
        tracing::debug!("Accept command. published event: {:?}", ev);
        Ok(ev)
    }
}

#[async_trait]
impl EventApplicator<EntityEvent> for Entity {
    #[tracing::instrument(skip_all)]
    async fn apply(&mut self, event: EntityEvent, ctx: &mut Context) {
        self.persist(&event, ctx).await;
        
        match event {
            EntityEvent::Created => {
                tracing::debug!("Entity created.");
            }
        }
    }
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("trace"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    
    let eventstore = InMemoryEventStore::default();
    let writer = EventWriter::new(eventstore).set_retry(5);
    
    nitinol::setup::set_writer(writer);
    
    let entity = Entity { id: "test-entity-1".to_string() };
    
    let system = ProcessManager::default();
    
    let receptor = system.spawn(entity, 0).await?;
    
    receptor.entrust(EntityCommand::Create).await?;
    
    tokio::time::sleep(Duration::from_secs(3)).await;
    Ok(())
}