# Raita

Rust solutions for the [`Fly.io Distributed Systems Challenge`](https://fly.io/dist-sys/) built on top of [`Maelstrom`](https://github.com/jepsen-io/maelstrom).

> "Yeh kya raita phailaya hai?"

This repository is intentionally not a polished framework. Each workload lives as its own binary because I wanted the repo to show the learning path, not just the final abstraction. That means there is some duplication across steps, and that is on purpose: each file is a snapshot of what I understood at that stage of the challenge.

## Progress

| Step | Binary | Status | Notes |
| --- | --- | --- | --- |
| Echo | `src/bin/echo.rs` | Done | Minimal request/reply implementation. |
| Unique IDs | `src/bin/unique_id.rs` | Done | Generates unique ULIDs per request. |
| Broadcast | `src/bin/broadcast.rs` | Done | Uses deterministic gossip plus per-neighbor knowledge tracking to reduce stale reads. |
| Grow-Only Counter | `src/bin/g_counter.rs` | In progress | Uses `seq-kv` with one shard key per node and sums shards on reads. |
| Later Maelstrom workloads | N/A | Not started | Left for future iterations. |

## Repository shape

- `src/bin/echo.rs`: first challenge step, kept simple on purpose.
- `src/bin/unique_id.rs`: second step, still small and close to the protocol.
- `src/bin/broadcast.rs`: third step, where more state management and gossip logic shows up.
- `src/bin/g_counter.rs`: fourth step, built on Maelstrom's `seq-kv` service.
- `src/message.rs`: shared message envelope and reply helpers.
- `src/state.rs`: shared node state, topology, local message set, and per-neighbor gossip bookkeeping.
- `src/node.rs`: runtime loop for reading stdin, writing stdout, and generating periodic events.
- `scripts/test.sh`: convenience script for running the implemented workloads against Maelstrom.

## What changed from step to step

### 1. Echo

The first step is mostly about understanding the protocol:

- deserialize a Maelstrom message
- swap `src` and `dest`
- preserve `in_reply_to`
- send `echo_ok`

No coordination, no local topology, no background work.

### 2. Unique IDs

The second step keeps the same runtime shape but introduces generated values:

- handle `generate`
- return `generate_ok`
- use `ULID` for uniqueness without needing cluster coordination

This step is still intentionally straightforward because the main lesson was how little machinery is needed when the problem allows locally generated IDs.

### 3. Broadcast

This is where the implementation starts to feel like a distributed systems exercise instead of a protocol exercise.

The current version:

- stores every broadcast value in local state
- tracks the configured topology and direct neighbors
- tracks which messages each neighbor is already known to have
- immediately fans out newly seen messages
- periodically retries gossip only for messages a neighbor is still missing

That last point is the important fix. Earlier versions relied on randomized gossip, which made the code harder to reason about and could still leave short stale-read windows. The current implementation is deterministic: if a neighbor is missing a value, we send it; once we send it, we record that knowledge locally.

### 4. Grow-Only Counter

This step switches away from direct node-to-node replication and starts using a Maelstrom-provided service:

- each node stores its own shard in `seq-kv`
- `add` reads and compare-and-swaps only the local shard key
- `read` sums all node shard keys to reconstruct the global counter
- CAS retries handle concurrent updates safely

This keeps the node itself stateless with respect to the counter value and matches the challenge's intent of building on the provided key/value service.

## Running the code

### Prerequisites

- Rust toolchain
- Maelstrom extracted under `res/maelstrom`
- Bash or a Bash-compatible shell for `scripts/test.sh`
- Java available for Maelstrom

### Rust tests

The lightweight Rust tests focus on the shared helpers and the tricky broadcast bookkeeping:

```bash
cargo test --lib --tests --target-dir target-codex
```

### Maelstrom runs

Build and run everything:

```bash
./scripts/test.sh all
```

Run individual steps:

```bash
./scripts/test.sh echo
./scripts/test.sh unique-ids
./scripts/test.sh broadcast-basic
./scripts/test.sh broadcast
./scripts/test.sh g-counter
```

The script uses:

- `echo`: single-node sanity check
- `unique-ids`: partitioned availability run
- `broadcast-basic`: smaller broadcast run
- `broadcast`: stricter broadcast run with more nodes and higher rate
- `g-counter`: grow-only counter against `seq-kv` with partitions

## Notes on verification

Rust-side tests currently cover:

- reply message construction and message ID handling
- neighborhood/topology bookkeeping
- "unknown message" calculation per neighbor
- broadcast fanout and gossip forwarding behavior
- grow-only counter request sequencing, CAS retry behavior, and shard aggregation

Saved Maelstrom artifacts live in `store/` from earlier runs and are useful as a diary of what passed, what regressed, and what got better over time.

## Tradeoffs

- I kept the workload binaries separate even where the logic overlaps, because readability of the learning path matters more here than maximal deduplication.
- Shared helpers only cover boring plumbing like message envelopes, state bookkeeping, and the runtime loop.
- `broadcast` now favors deterministic propagation over randomized resend, because it is easier to reason about and easier to test.
- `g-counter` is implemented with one key per node in `seq-kv`, which keeps writes local and makes reads a simple sum across shards.

## What remains

The next natural step after this repo would be to continue into the later Maelstrom workloads:

- CRDTs
- transactional key/value workloads
- Raft-style replication work

For now, this repository stops after a solid version of the first three steps.

## Related links

- <https://fly.io/dist-sys/>
- <https://github.com/jepsen-io/maelstrom>
- <https://github.com/jepsen-io/maelstrom/tree/main/doc/01-getting-ready>
- <https://github.com/jepsen-io/maelstrom/blob/main/doc/protocol.md>
- <https://pkg.go.dev/github.com/jepsen-io/maelstrom/demo/go>

## References

- <https://www.youtube.com/watch?v=gboGyccRVXI>
- <https://vishalcjha.medium.com/rust-in-distributed-system-environment-8b164b6a5bed>
