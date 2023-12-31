# QueryCache
a simple project to demonstrate how the usage of query-caching can signficantly improve the performance of a web application.

Memcache: Redis

Database: MongoDB

## Design
### Usecase: User sends a request resulting in a cache hit
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

### When to invalidate the cache?
The most straightforward way to handle these cases is to simply set a max time that a cached entry can stay in the cache before it is updated, usually referred to as time to live (TTL).
