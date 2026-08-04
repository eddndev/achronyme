# Achronyme 0.1.0 host standard library

This reference summarizes the concurrency and capability-I/O APIs. The
[generated builtin registry](builtins.md) is the authoritative table for
arity, effects, capabilities, behavior, cancellation, and resource actions.
Every suspending call below must appear under `await` or as the direct call
operand of `spawn`.

## Cooperative tasks and timers

| Call | Result and contract |
| --- | --- |
| `yield_now()` | Suspends until the task takes its next FIFO ready-queue turn. |
| `sleep(milliseconds)` | Completes after 0 through 86,400,000 ms. Requires `clock`. |
| `timeout_after(milliseconds)` | Creates the same bounded timer request for use in timeout/race tasks. Requires `clock`. |
| `cancel_check()` | Immediate cooperative cancellation point; this call is not awaited. |

Timers share one runtime reactor. A timer does not allocate one OS thread per
task. Racing tasks use language-level `await [handles] as race`; a timer does
not cancel another operation by itself.

## Bounded channels and permits

| Call | Result and contract |
| --- | --- |
| `channel(capacity)` | Creates an owned bounded channel. Capacity is 0 through 65,535; zero is rendezvous mode. |
| `channel_send(channel, value)` | Suspends while the bounded channel cannot accept the value. |
| `channel_receive(channel)` | Suspends while no value is available. |
| `channel_close(channel)` | Immediately consumes the channel resource. |
| `permit_pool(limit)` | Creates a permit controller with 1 through 65,535 permits. |
| `bounded_server(limit)` | Canonical alias for an active-handler permit controller. |
| `permit_acquire(pool)` | Suspends until it consumes one permit. |
| `permit_release(pool)` | Suspends until it returns one permit. |

Channel messages are immutable `nil`, Boolean, Number, String, Bytes, Field,
BigInt, or Proof values. Mutable collections, tasks, and owned resources are
not channel messages. Send and receive queues, channel count, and pending
channel operations are bounded. A full channel applies backpressure instead
of allocating without limit.

For a server loop, acquire a permit before accepting or spawning a handler and
release it only after that handler's structured cleanup has completed.

## Owned file I/O

| Call | Result and contract |
| --- | --- |
| `open_file(path)` | Opens an owned File for reading. Requires a matching `file.read` root. |
| `create_file(path)` | Creates or truncates an owned File. Requires a matching `file.write` root. |
| `file_read(file, max_bytes)` | Reads 1 through 16 MiB and returns Bytes. |
| `file_write(file, value)` | Writes a String or Bytes chunk up to 16 MiB and returns its byte count. |
| `file_close(file)` | Closes and consumes the File. |

File work runs through one shared bounded blocking pool. Cancellation can stop
work before it starts. Once host work is running, the scope waits for its
completion and cleanup; it is never detached in the background.

The 0.0.1 compatibility calls remain available:

| Call | Compatibility behavior |
| --- | --- |
| `read_line()` | Blocking console input with `console.read`. |
| `read_file(path)` | Bounded blocking whole-file read with `file.read`. |
| `write_file(path, text)` | Bounded blocking whole-file write with `file.write`. |

These three calls remain ordinary blocking calls so existing sequential source
does not acquire a new `await` requirement. New concurrent code should prefer
the owned operations because their suspension and lifetime are explicit.

Run the tested example with an exact root:

```text
ach --allow-read ./tmp --allow-write ./tmp run examples/concurrency/owned_file.ach
```

The example reads the target file path from standard input.

## Reactor-backed TCP

| Call | Result and contract |
| --- | --- |
| `tcp_connect(address)` | Creates an owned Connection. Requires the exact `network.connect` address. |
| `tcp_listen(address)` | Creates an owned Listener. Requires the exact `network.listen` address. |
| `tcp_accept(listener)` | Borrows the Listener and creates an owned Connection. |
| `tcp_read(connection, max_bytes)` | Reads 1 through 16 MiB; returns Bytes, or `nil` for EOF. |
| `tcp_write(connection, value)` | Writes a complete String or Bytes chunk up to 16 MiB and returns its byte count. |
| `tcp_close(connection)` | Closes and consumes the Connection. |
| `tcp_listener_close(listener)` | Closes and consumes the Listener. |

Addresses are numeric `IP:PORT` strings. Grants are exact; no DNS resolution,
wildcards, or implicit network access is performed. The single readiness
reactor tracks listeners and connections without requiring one OS thread per
language task. Reads may be partial, EOF is explicit, and writes complete the
provided bounded chunk or fail.

Run the loopback example after choosing one unused address, then provide the
same address on standard input:

```text
ach --allow-connect 127.0.0.1:9000 \
    --allow-listen 127.0.0.1:9000 \
    run examples/concurrency/tcp_echo.ach
```

## Capability and cancellation summary

Capabilities authorize opening a file, creating a file, connecting, or
listening. Operations on an already owned resource require possession of its
unforgeable handle and carry no new ambient path or address authority.

The registry distinguishes:

- `none`: no cancellation observation is needed.
- `before-start`: cancellation can prevent dispatch; running blocking work is
  joined before cleanup finishes.
- `cooperative`: the reactor, timer, or channel request observes cancellation
  through the scheduler.

Every owned resource is closed exactly once by explicit consumption or lexical
scope cleanup.
