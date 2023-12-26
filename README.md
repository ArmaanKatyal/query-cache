# QueryCache
a simple project to demonstrate how the usage of query-caching can improve the performance of a web application.

Memcache: Redis
Database: MongoDB

## Design
### Usecase: User sends a request resulting in a cache hit
Popular queries are served from Redis to reduce the read latency. Reading 1 MB sequentially from memory takes about 250 microseconds, while reading from SSD takes 4x and from disk takes 80x longer.

- The Client receives the request
- Query is parsed and then converted to a hash to be used as a key for the cache
- The cache is checked for the key
- If the key is found, the result is returned to the client

### Usecase: User sends a request resulting in a cache miss
- The Client receives the request
- Query is parsed and then converted to a hash to be used as a key for the cache
- The cache is checked for the key
- If the key is not found, the query is executed against the database (mongodb)
- The result is stored in the cache
- The result is returned to the client
