# nitinol-inmemory-adaptor

This is a simple adaptor to use Nitinol with inmemory database.

[!CAUTION]
This in-memory adapter loses data when the program is terminated. (of course)
It should only be used to test the library and should not be incorporated into production.


### Usage
```toml
# Why is it not uploaded to crates.io?
# Because there are still many aspects of this library 
# that are incomplete and could cause a TREMENDOUS amount of destruction.
[dependencies.nitinol-inmemory-adaptor]
git = "https://github.com/HalsekiRaika/nitinol-inmemory-adaptor"
```
