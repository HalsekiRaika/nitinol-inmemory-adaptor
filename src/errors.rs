use nitinol::EntityId;

#[derive(Debug, thiserror::Error)]
pub enum MemoryIoError {
    #[error("No target event was found. {0}")]
    NotFound(EntityId),
}