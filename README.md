# redlock-cli

A command line tool providing a [distributed lock] built on top of Redis.

Usage:

```shell
redlock -s localhost:6379 -l mylock --ttl 60 -- echo foo
```

This will try to acquire the global lock with the name `mylock` in Redis for at
least 60 seconds. When lock is acquired, the command is executed. The lock is
released when the command finishes, or the ttl expires. Additional parameter
`--timeout` specifies for how long we should try to acquire the lock.

This is in particular useful when restricting the number of concurrent jobs in
CI.

## License

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

The files [src/proto/fileformat.proto](src/proto/fileformat.proto) and
[src/proto/osmformat.proto](src/proto/osmformat.proto) are copies from the
[OSM-binary] project and are under the LGPLv3 license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this document by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

[distributed lock]: https://redis.io/topics/distlock
