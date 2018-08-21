# Client Libraries

Hanbaiki uses the same protocol as Redis for its client-server communication called RESP (REdis Serialization Protocol). See [specification](https://redis.io/topics/protocol). Theoretically, you can use a Redis client or any client that supports the Redis protocol to communicate with a Hanbaiki server.

Tested Languages:

* [Elixir](#elixir)
* [Python](#python)
* [Ruby](#ruby)

Languages prioritized for testing next:

* Go
* Java

## Notable Differences

If a Redis client cannot accommodate the following differences, it cannot work with Hanbaiki.

### Different Port

The default port of Hanbaiki is 6363 which is different from the default port of Redis, 6379. You should specify the port when establishing a connection when using a Redis client.

### Different Commands

Hanbaiki has a different set of [commands](commands.html) compared to Redis. You'll need a client that allows you to specify custom commands.

## Elixir

[redix](https://github.com/whatyouhide/redix)

Redix doesn't provide Elixir functions for each Redis command. Instead, it provides a `command` function that you can use to specify Hanbaiki commands:

```
{:ok, conn} = Redix.start_link(host: "127.0.0.1", port: 6363)

Redix.command(conn, ["SET", "hello", "world"])
#=> {:ok, "OK"}

Redix.command(conn, ["GET", "hello"])
#=> {:ok, "world"}

Redix.command(conn, ["EXISTS", "hello"])
#=> {:ok, 1}
```

## Python

[redis-py](https://github.com/andymccurdy/redis-py)

redis-py implements every Redis command so for the few Hanbaiki commands that overlap with Redis, you can use the same methods:

```
>>> import redis
>>> r = redis.StrictRedis(host='127.0.0.1', port=6363)

>>> r.set('hello', 'world')
True

>>> r.get('hello')
b'world'

>>> r.exists('hello')
True
```

For other commands, you can use `execute_command` method to specify the Hanbaiki command:

```
>>> r.execute_command('COUNT')
1
>>> r.execute_command('DESTROY')
b'OK'
```

## Ruby

[redis-rb](https://github.com/redis/redis-rb)

Some Hanbaiki commands such as `SET` and `GET` map directly to redis-rb's methods. For these few cases, you can use the client library's methods:

```
require 'redis'
redis = Redis.new(host: "127.0.0.1", port: 6363)

redis.set("hello", "world")
# => "OK"

redis.get("hello")
# => "world"

redis.exists("hello")
# => true
redis.exists("world")
# => false

redis.quit
# => "OK"
```

For other commands, you have to use the `call` method which allows you to specify the Hanbaiki command:

```
redis.call("DELETE", "hello")
# => "OK"
redis.call("DESTROY")
# => "OK"
```
