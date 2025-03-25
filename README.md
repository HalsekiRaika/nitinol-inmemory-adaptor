# nitinol-inmemory-adaptor

This is a simple adaptor to use Nitinol with inmemory database.

> [!CAUTION]  
> This in-memory adapter loses data when the program is terminated. (of course)  
> It should only be used to test the library and do not be used into production.

## Usage
```rust
#[tokio::main]
async fn main() {
    let eventstore = InMemoryEventStore::default();
    let writer = EventWriter::new(eventstore).set_retry(5);
    
    // Install as global writer.
    nitinol::setup::set_writer(writer);
}
```