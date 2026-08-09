# Achronyme 0.1.0 language reference

This document defines the public structured-concurrency and capability-I/O
surface added in Achronyme 0.1.0. The complete native-call metadata is in the
[generated builtin reference](builtins.md).

## Execution domains

Achronyme has two deliberately separate domains:

- Host execution runs ordinary functions and may use explicitly granted host
  capabilities.
- `prove` and `circuit` bodies construct deterministic provable computation.
  They reject task, file, network, clock, randomness, and unknown-host effects,
  including effects reached through another function.

The compiler infers effects for ordinary `fn` declarations. Source annotations
are not required. Direct, recursive, imported, and statically possible
higher-order calls participate in interprocedural inference. A dynamic host
call that cannot be classified requests `host.unknown`; it never receives a
more precise authority by assumption.

## Structured concurrency syntax

`concurrent`, `spawn`, and `await` are reserved words in 0.1.0.

```ach
fn work(value) {
    await yield_now()
    value * 2
}

let answer = concurrent {
    let left = spawn work(10)
    let right = spawn work(11)
    await left + await right
}
```

`concurrent { body }` is an expression and creates a lexical task scope. The
body runs in the current task. `spawn call_expression` is valid only inside a
concurrent scope and creates a child owned by that scope. `await expression` is
the only source-level suspension marker.

A normal scope exit waits for every child. A task cannot detach, and a handle
cannot escape its owning scope. Discarding a handle does not discard the child
or hide its failure. Returning from or failing inside the body cancels
unfinished children, waits for their cleanup, and then completes the exit.

The VM executes at most one Achronyme task at a time. Ready tasks take FIFO
turns. Task interleaving occurs only at explicit `await` sites. `yield_now()` is
an awaited cooperative turn; `cancel_check()` is the one immediate task-effect
operation and does not use `await` because it cannot suspend.

## Await modes

The default form returns a successful task result and propagates failure:

```ach
let value = await task
```

A task handle is single-use. Awaiting it again is an error.

The outcome form converts success or failure into a map:

```ach
let outcome = await task as outcome
if outcome["ok"] {
    print(outcome["value"])
} else {
    print(outcome["error"])
}
```

The race form requires at least two distinct handles and returns the first
terminal outcome. The zero-based `index` identifies the input handle. Losing
tasks are cancelled and joined before the scope exits.

```ach
let outcome = await [first, second] as race
// {"index": Number, "ok": true, "value": value}
// {"index": Number, "ok": false, "error": String}
```

## Failure and cancellation

The first unhandled child failure is the primary scope failure. The runtime
requests cancellation of the scope body and its siblings, then waits for
cleanup. Additional cleanup failures remain attached to task-tree diagnostics.

Cancellation is cooperative. It is observed at suspension, at
`cancel_check()`, and according to each native operation's cancellation
contract. It never interrupts an arbitrary bytecode instruction. A CPU-bound
task must call `cancel_check()` or reach an awaited operation to respond.

Task diagnostics carry the task ID, parent, spawn and current source location,
state, wait reason, primary failure, and cleanup failures.

## Owned resources

Files, TCP listeners, and TCP connections are opaque owned resources. A value
cannot forge a resource handle. Passing an owned resource to `spawn` transfers
it to the child; the parent cannot reuse it. Borrowed resource operations do
not permit a borrow to cross a task boundary.

Explicit close consumes a resource. Lexical scope cleanup closes any remaining
owned resource exactly once. Garbage collection is only a final leak safeguard
and is not the specified cleanup mechanism.

## Capabilities

Compilation persists the exact requested effects and host capabilities in the
ACHB manifest. The host validates the whole request before execution and again
at native invocation. A denied request performs no file or network operation.

The host capabilities are:

- `console.read` and `console.write`
- `file.read` and `file.write`
- `network.connect` and `network.listen`
- `clock` and `random`
- `host.unknown`

Embedders and WASM start untrusted with no ambient authority. The interactive
CLI has an explicit sequential-compatibility policy that grants console,
clock, and random access. File roots and exact numeric TCP addresses remain
opt-in:

```text
ach --allow-read ./data --allow-write ./out run program.ach
ach --allow-connect 127.0.0.1:9000 run client.ach
ach --allow-listen 127.0.0.1:9000 run server.ach
```

Filesystem grants are canonical directory roots and reject traversal outside
the root. Network grants accept an exact numeric `IP:PORT`; DNS names and
address wildcards are not grants.

Use `ach inspect program.ach --manifest` to inspect requested and granted
capabilities, effects, roots, addresses, formats, and limits without running
the program. Add `--error-format json` for stable machine-readable output.

## Runtime limits

Every VM has finite bounds. The CLI and `[vm]` configuration expose:

- `max_tasks`: live child tasks across explicit `spawn` and the implicit child
  created by `await function_call(...)`; the root task is not counted.
- `max_resources`
- `max_task_scopes`: simultaneously live structured task scopes across the
  VM, including explicit `concurrent` scopes and implicit single-child `await`
  scopes. This is a global live count, not a nesting-depth limit.
- `max_pending_native_requests`
- `max_retained_task_results`
- `max_channels`
- `max_channel_operations`
- `blocking_workers`
- `blocking_queue_capacity`

The equivalent global flags use hyphens, for example `--max-tasks`,
`--max-pending-native-requests`, and `--blocking-queue-capacity`. Limit
exhaustion is a recoverable runtime error; it does not silently grow a queue.

| Runtime dimension | Default and hard maximum |
| --- | ---: |
| Live child tasks, explicit and implicit | 65,535 |
| Open owned resources | 65,535 |
| Simultaneously live task scopes | 1,024 |
| Pending native requests | 4,096 |
| Retained task results | 4,096 |
| Open channels | 4,096 |
| Pending channel operations | 65,535 |
| Blocking workers | 4 default, 64 maximum |
| Blocking request queue | 64 default, 65,535 maximum |

Except for blocking workers and its queue, the configured default is also the
hard maximum. A configured zero is valid only for dimensions that can be
disabled: resources, pending native requests, retained results, channels, and
channel operations. At least one task, one scope, one blocking worker, and one
blocking queue slot are required.

```toml
[vm]
allow_read = ["data"]
allow_connect = ["127.0.0.1:9000"]
max_tasks = 128
max_resources = 64
max_task_scopes = 16
max_pending_native_requests = 64
max_retained_task_results = 64
max_channels = 32
max_channel_operations = 256
blocking_workers = 4
blocking_queue_capacity = 64
```

## Backends and WASM

The interpreter is the semantic oracle. LLVM JIT and AOT preserve the same
scope, failure, cancellation, resource, and output behavior. Compiled code
bails out at structured task operations and resumes the exact active task and
source operation through runtime ABI v6.

The browser WASM runtime supports pure structured tasks, bounded in-memory
channels, cooperative yield, and captured virtual console output. It has no
ambient host authority. Timers, files, network, console input, and randomness
are unsupported until an embedder supplies an explicit adapter and grant.
Their current calls fail capability preflight rather than block or reach the
browser environment. `runtime_support()` exports this matrix as JSON.

## Tested examples

- [Structured tasks](../../examples/concurrency/structured_tasks.ach)
- [Recoverable task failure](../../examples/concurrency/task_outcome.ach)
- [Bounded channel backpressure](../../examples/concurrency/bounded_channel.ach)
- [Bounded producer/consumer pipeline](../../examples/concurrency/channel_pipeline.ach)
- [Timer race](../../examples/concurrency/timer_race.ach)
- [Owned file I/O](../../examples/concurrency/owned_file.ach)
- [Owned TCP echo](../../examples/concurrency/tcp_echo.ach)

All seven examples compile and execute in the CLI integration suite. File and
TCP examples run only inside temporary roots and exact loopback grants.
