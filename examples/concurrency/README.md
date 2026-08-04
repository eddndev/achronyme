# Achronyme 0.1.0 concurrency examples

These programs exercise the public structured-concurrency and capability-I/O
surface. The CLI integration suite compiles and executes every example.

## In-memory examples

These examples need no file or network authority:

```text
ach --no-config run examples/concurrency/structured_tasks.ach
ach --no-config run examples/concurrency/task_outcome.ach
ach --no-config run examples/concurrency/bounded_channel.ach
ach --no-config run examples/concurrency/channel_pipeline.ach
ach --no-config run examples/concurrency/timer_race.ach
```

- `structured_tasks.ach` joins two scope-owned child tasks.
- `task_outcome.ach` recovers a child failure with `await ... as outcome`.
- `bounded_channel.ach` demonstrates backpressure with a capacity-one channel.
- `channel_pipeline.ach` runs a producer and consumer through a bounded queue.
- `timer_race.ach` returns the first task and cleans up the losing task.

The timer example requests the CLI's explicit clock compatibility grant. Pure
tasks, channels, and cooperative yield are also supported by the WASM runtime;
timers require an adapter there.

## Owned file I/O

Choose a path inside a granted directory and provide it on standard input:

```text
mkdir -p tmp
printf '%s\n' "$PWD/tmp/example.txt" | \
  ach --no-config --allow-read ./tmp --allow-write ./tmp \
  run examples/concurrency/owned_file.ach
```

The program creates, writes, closes, reopens, reads, and closes an owned file.
Both grants are exact canonical roots; no ambient filesystem access is used.

## Owned TCP echo

Choose one unused numeric loopback address, grant that exact address, and pass
it on standard input:

```text
printf '%s\n' '127.0.0.1:9000' | \
  ach --no-config \
  --allow-connect 127.0.0.1:9000 \
  --allow-listen 127.0.0.1:9000 \
  run examples/concurrency/tcp_echo.ach
```

The server and client are children of one lexical scope. Their connection and
listener resources are consumed explicitly, with lexical cleanup as the
failure fallback.
