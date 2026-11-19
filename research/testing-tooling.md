# Research: Testing tooling for a hand-written Postgres-style DB in Rust

Resolves issue #7 ("Testing tooling"), a child of map issue #1. It feeds the later **Testing strategy** decision, so it states facts and trade-offs and makes **no recommendation**.

Researched 2026-11-19. Crate versions come from the crates.io API on that date. Every claim links to the primary source it came from.

---

## 0. Summary matrix

| Tool / technique | Layer it tests | Usable from which build stage | Runs on macOS? | Key prerequisite |
|---|---|---|---|---|
| `sqllogictest` crate (in-process `impl DB`) | SQL semantics | As soon as there is a `fn execute(sql) -> rows` | Yes | Small adapter trait impl |
| `sqllogictest-bin` CLI, `postgres` engine | SQL + wire protocol (black box) | Once the **simple query** protocol works | Yes | Server speaks PG simple query protocol |
| goldenscript (toydb style) | Any internal component (MVCC, storage, Raft, SQL) | From day one | Yes | Write a `Runner` per component |
| Postgres isolation-tester *spec format* | Concurrent sessions / locking / isolation | Once multiple connections + transactions work | Format: yes. Real `isolationtester` binary: **no** (see §3.1) | Own runner needed |
| Hermitage test scripts | Isolation-level anomalies (RC / RR) | Once transactions + 2–3 concurrent connections work | Yes (only needs `psql`) | Manual, or scripted by you |
| Elle / elle-cli | Black-box anomaly detection from histories | Once concurrent transactions work over the wire | Yes, but needs **Java 21** | Rust client that records histories |
| `loom` | In-process concurrent data structures (latches, buffer pool, lock table) | Whenever such a structure exists | Yes | `cfg(loom)` type swaps in the code under test |
| `proptest` / `proptest-state-machine` | Any component with a model (B-tree vs `BTreeMap`, page codec, WAL record codec) | From day one | Yes | A reference model |
| kill -9 child-process harness | Process-crash recovery (WAL replay) | Once WAL + recovery exist | Yes | Server as a separate process |
| `fail` (fail-rs) failpoints | Error paths and crashes at chosen code points | Any time | Yes | `fail_point!` calls in code |
| Simulated-VFS / snapshot crash testing (SQLite style) | **Power-loss** recovery (loss of unsynced writes, torn writes) | Once WAL + recovery exist | Yes (it is your own code) | All file I/O goes through an abstraction you own |
| LazyFS | Power-loss on the real filesystem | Once WAL + recovery exist | **No** (FUSE 3, tested on Debian/Ubuntu) | Linux VM |
| turmoil (+ `turmoil-fs`) / madsim | Deterministic simulation of async runtime, network, and (turmoil) fs | Needs an async (tokio) architecture | Yes | Replace tokio/fs with simulated crates |

The most important layering fact: **`loom` and Elle/Hermitage/isolation specs are not alternatives.** loom checks memory-model interleavings of in-process Rust code ([loom docs](https://docs.rs/loom/latest/loom/)). The others check *transaction-level* isolation semantics seen through SQL. A DB with MVCC and concurrent connections has bugs at both layers.

---

## 1. sqllogictest: the format and `sqllogictest-rs`

### 1.1 The format

- sqllogictest (SLT) began in SQLite. SQLite runs it to check that SQLite gives the same answers as other engines: "SLT currently compares SQLite against PostgreSQL, MySQL, Microsoft SQL Server, and Oracle 10g. SLT runs 7.2 million queries comprising 1.12GB of test data." ([sqlite.org/testing.html](https://www.sqlite.org/testing.html))
- A file is a sequence of records separated by blank lines ([DuckDB sqllogictest intro](https://duckdb.org/docs/2.0/dev/sqllogictest/intro.html)):
  - `statement ok` / `statement error` followed by SQL.
  - `query <types>` followed by SQL, a `----` line, then the expected rows.
- **Column type letters** in sqllogictest-rs are `T` (text), `I` (integer), `R` (floating point), plus "any" ([`column_type.rs`](https://github.com/risinglightdb/sqllogictest-rs/blob/main/sqllogictest/src/column_type.rs), `DefaultColumnType`). You can define your own `ColumnType`.
- **Sort modes** ([`parser.rs`](https://github.com/risinglightdb/sqllogictest-rs/blob/main/sqllogictest/src/parser.rs), `enum SortMode`):
  - `nosort` (default): rows in the order the engine returned them.
  - `rowsort`: sort all rows.
  - `valuesort`: sort every individual value, ignoring row grouping.
  - Without `ORDER BY`, SQL result order is unspecified, so a test without `rowsort` checks an implementation detail.
- **Directives the sqllogictest-rs parser accepts** (same `parser.rs`): `include`, `halt`, `subtest`, `sleep`, `skipif <label>`, `onlyif <label>`, `connection <name>`, `statement … [retry N backoff D]`, `query … [retry N backoff D]`, `system ok`, `control` (e.g. `control substitution on`, sort-mode control), `hash-threshold`, `let`.
  - `connection <name>` lets one file drive **several named connections**. That gives basic multi-session tests in SLT, but with no blocking/wait detection.
- Extensions documented in the [README](https://github.com/risinglightdb/sqllogictest-rs):
  - `statement error <regex>` or a SQLSTATE match `(<code>)`; needs `error_sql_state` on the DB trait.
  - `<slt:ignore>` for volatile output.
  - Substitution variables `$__TEST_DIR__`, `$__NOW__`, `$__DATABASE__`.
  - `--override` rewrites expected output from actual output, a "golden file" workflow.

### 1.2 The Rust crate (library), `sqllogictest` 0.29.1 (released 2026-02-14)

- The [docs.rs](https://docs.rs/sqllogictest/latest/sqllogictest/) example implements `sqllogictest::DB` with:
  - `type Error`
  - `type ColumnType` (e.g. `DefaultColumnType`)
  - `fn run(&mut self, sql: &str) -> Result<DBOutput<Self::ColumnType>, Self::Error>`
- `AsyncDB` ([docs](https://docs.rs/sqllogictest/latest/sqllogictest/runner/trait.AsyncDB.html)) has:
  - async `run` and `shutdown`.
  - Overridable `engine_name`, `sleep`, `run_command`, and `error_sql_state`. The defaults use `std::thread::sleep` and `std::process::Command`, and the docs say to override them with tokio versions under tokio.
  - Every `D: DB + Send` gets `AsyncDB` automatically.
- Usage pattern:
  - `Runner::new(|| async { Ok(db) })` then `tester.run_file("…/basic.slt")`, or `parse_file` and run each record yourself.
  - The `harness!` macro takes a `db_fn` and a glob `pattern`, and generates one test per file (docs.rs).
- **How this applies here:** an in-process `impl DB` lets SQL tests run *before* the wire protocol exists. Your executor returns rows and you turn them into `DBOutput`.

### 1.3 The CLI, `sqllogictest-bin`, run as a black-box tester of your server

Source: [`sqllogictest-bin/src/main.rs`](https://github.com/risinglightdb/sqllogictest-rs/blob/main/sqllogictest-bin/src/main.rs) and [`sqllogictest-engines`](https://github.com/risinglightdb/sqllogictest-rs/tree/main/sqllogictest-engines).

- Install with `cargo install sqllogictest-bin`. Run with `sqllogictest './test/**/*.slt'`.
- Engines (`EngineConfig`): `postgres`, `postgres-extended`, `mysql`, and `external` (a command template).
- **The `postgres` engine uses `tokio-postgres`'s `simple_query`**, which returns all values as strings (`engines/src/postgres/simple.rs`). So **once your server handles the PG simple query protocol** (StartupMessage, auth, Query → RowDescription/DataRow/CommandComplete/ReadyForQuery), the CLI can run the same `.slt` files against it with no Rust glue. `postgres-extended` also needs Parse/Bind/Execute.
- Flags:
  - Connection: `--host`/`-h`, `--port`/`-p`, `--db`, `--user`, `--pass`, each with an `SLT_*` env var.
  - Other: `--override`, `--format`, `--junit <file>`, `--fail-fast`, `--label` (for `skipif`/`onlyif`), `--skip <regex>`, `--partition-id/--partition-count`, `--shutdown-timeout`.
- **`-j/--jobs N` creates one database per test file** (help text in `main.rs`). That needs `CREATE DATABASE` (and drop) support in your server. Without `-j`, all files share one database, as with RisingWave's caveat below.
- **Bonus:** the same `.slt` files can run against **real Postgres** (`--engine postgres` pointed at a real PG). That gives a reference oracle for "what would Postgres return?"
- Users listed in the README: RisingLight, RisingWave, DataFusion, Databend, CnosDB.

### 1.4 How other projects structure SQL-level tests

- **DuckDB** ([docs](https://duckdb.org/docs/2.0/dev/sqllogictest/intro.html)):
  - Layout and naming:
    - "Every test is a single self-contained file located in the `test/sql` directory."
    - The folder is the test group, e.g. `test/sql/projection/…` → `[projection]`.
    - `.test` files are the fast set. `.test_slow` files run only with `unittest *`.
  - Directives and runner:
    - `require <ext>` skips the rest of a file when a feature is missing.
    - `PRAGMA enable_verification` runs each query optimized and unoptimized and compares the results. This is a cheap differential oracle for an optimizer.
    - DuckDB uses its own C++ runner (`unittest`), not sqllogictest-rs.
- **RisingWave** ([e2e_test](https://github.com/risingwavelabs/risingwave/tree/main/e2e_test)):
  - About 40 topic subdirectories of `.slt` files (`batch`, `ddl`, `dml`, `streaming`, `tpch`, …), run through `risedev slt`, which sets `SLT_HOST/SLT_PORT/SLT_DB`.
  - The README warns against a single glob over everything. Some groups (DDL) need a fresh instance.
  - Tests must drop the tables they create, "because all tests share a single database."
  - So a shared database leads to cleanup discipline and ordering problems. Per-file databases (`-j`) avoid that but need `CREATE DATABASE`.
- **toydb** ([README](https://github.com/erikgrinaker/toydb)) does **not** use SLT. It uses **goldenscript** ([repo](https://github.com/erikgrinaker/goldenscript), crate 0.8.0), a Rust port "loosely based on Cockroach Labs' datadriven framework for Go":
  - Format: `command args` / `---` / expected output.
  - You implement a `Runner` trait whose `run` returns a `String` to compare.
  - Expected output is generated with `UPDATE_GOLDENFILES=1 cargo test` and then reviewed in the diff.
  - Script directories by layer:
    - `src/raft/testscripts/node`
    - `src/storage/testscripts/mvcc`
    - `src/sql/testscripts`
    - `tests/scripts` (end-to-end)
  - **Trade-off vs SLT:** goldenscript can test *internal* components (MVCC store, storage engine) with a custom command language, including interleaved transactions (`t1: begin`, `t2: get b`, …). SLT tests only SQL in → rows out.

---

## 2. Crash-recovery testing

### 2.1 Process crash: kill -9 harness

- **Pattern:** start the server as a child process, run a workload, send `SIGKILL`, restart, then check invariants. Typical checks: every acknowledged commit is present, no uncommitted data is visible, and integrity checks pass.
- **SQLite's TCL harness** works this way: "The main testing process spawns a child process which runs some SQLite operation and randomly crashes somewhere in the middle of a write operation… After the child dies, the original test process opens and reads the test database and verifies that the changes attempted by the child either completed successfully or else were completely rolled back." ([testing.html](https://www.sqlite.org/testing.html))
- **Postgres** has TAP (Perl) tests in [`src/test/recovery/t/`](https://github.com/postgres/postgres/tree/master/src/test/recovery/t), e.g. `013_crash_restart.pl`:
  - It "Tests restarts of postgres due to crashes of a subprocess."
  - It keeps a psql backend alive so it can kill it.
  - These tests need `--enable-tap-tests` ([regress-run docs](https://www.postgresql.org/docs/current/regress-run.html), [recovery README](https://github.com/postgres/postgres/blob/master/src/test/recovery/README)).
- **Limitation (important):** SIGKILL kills the *process*, but the OS page cache survives. Data that was `write()`n but never `fsync`ed is still there after the restart. So **a kill -9 harness cannot find missing-fsync bugs or torn-page bugs.** It tests "process crash", not "power loss". This is why SQLite's harness adds a special VFS (next section).

### 2.2 Power loss: simulated I/O layer

- **SQLite's approach** ([testing.html](https://www.sqlite.org/testing.html)):
  - TCL harness: "A special VFS randomly reorders and corrupts the unsynchronized write operations to simulate the effect of buffered filesystems."
  - TH3 harness uses an in-memory VFS that "can be rigged to make a snapshot of the entire filesystem after a set number of I/O operations":
    - The snapshot point moves forward on each iteration until the operation completes with no snapshot.
    - The filesystem is reverted to the snapshot, and random power-loss-style damage is added.
    - The DB is checked: either the transaction "ran to completion or was completely rolled back".
  - I/O-error and OOM testing use the same "fail at the Nth operation, then N+1, …" loop, run once with a single failure and once with persistent failure after the first.
  - `PRAGMA integrity_check` verifies the result in each case.
- **Applicability:** this requires all disk I/O to go through a trait or abstraction **you own**, such as a `trait Vfs` / `trait Disk`. A crash then becomes "drop all writes since the last `fsync`, optionally tear the last page". Tests can do this in-process, deterministically, and on macOS.
- **LazyFS** ([repo](https://github.com/dsrhaslab/lazyfs)) does the same at the filesystem level:
  - It is a FUSE filesystem with its own page cache that "only flushes data if explicitly requested by the application".
  - `lazyfs::clear-cache` drops unsynced data. Configured faults like "clear after the 6th fsync" and torn writes (`torn-seq`, `torn-op`) are supported.
  - **Requires FUSE 3. Tested on ext4 / Debian 11 / Ubuntu 20.04, so a Linux VM is needed on macOS.** No code changes are needed in the DB.
- **`turmoil-fs`** ([turmoil repo](https://github.com/tokio-rs/turmoil), `crates/turmoil-fs`, crate 0.1.0):
  - It is a "Deterministic simulated filesystem. Drop-in replacement for `std::fs` / `tokio::fs`."
  - turmoil's README lists torn writes among the hardships it injects.
  - The crate is very new (0.1.0), so treat it as early.

### 2.3 Failpoints: `fail` (fail-rs), from TiKV

- Source: [repo](https://github.com/tikv/fail-rs).
- Put `fail_point!("name")` in code. Failpoints compile to nothing unless the `failpoints` Cargo feature is on.
- Configure them with the `FAILPOINTS` env var (e.g. `FAILPOINTS=read-dir=panic`). `FailScenario::setup()/teardown()` scopes them.
- Documented actions include panic, early return, and sleep, triggered conditionally or probabilistically.
- **Latest release 0.5.1, 2022-10-08** (crates.io). It is stable but not actively released.
- **Use for recovery tests:** crash, or return an I/O error, at an exact point. Examples: "after WAL append but before fsync", or "after fsync but before page write". This makes those points deterministic, where kill -9 timing is random.

### 2.4 Postgres's own WAL check

- `wal_consistency_checking=all`, enabled by `PG_TEST_EXTRA=wal_consistency_checking`, runs selected recovery tests with full WAL consistency checks. It is "resource intensive" ([regress-run](https://www.postgresql.org/docs/current/regress-run.html)).
- **Applicability:** this is a design idea, not a tool. During testing, compare each page after redo with the page as it was before the crash.

---

## 3. Concurrency and isolation testing

### 3.1 Postgres isolation tester (spec files)

Source: [`src/test/isolation/README`](https://github.com/postgres/postgres/blob/master/src/test/isolation/README).

- **Format:**
  - Global `setup { SQL }` (may repeat) and `teardown { SQL }`.
  - `session <name>` blocks, each with optional `setup`/`teardown` and `step <name> { SQL }`. Step names are unique per file.
  - `permutation s1a s2a s1c …` lines.
- If no permutation is given, the tester runs **all interleavings** that keep each session's steps in order.
- **Execution:**
  - "Each session is executed in its own connection."
  - Per permutation, the order is: main setup → session setups → steps → session teardowns → main teardown. The main setup and teardown run in a separate control connection.
- **Blocking detection:**
  - "isolationtester recognizes that a command has blocked by looking to see if it is shown as waiting in the pg_locks view." Only heavyweight-lock waits are detected.
  - Markers: `step(*)` reports the step as waiting immediately. `step(other)` and `step(other notices N)` delay *reporting* of completion. Unresolved blocks are cancelled after 2 × `PG_TEST_TIMEOUT_DEFAULT`.
- **Applicability (important):** the real `isolationtester` binary **cannot run against this DB as-is**, because it relies on `pg_locks`, which this DB will not have. The **spec format and the expected-output-file idea can be reused** with your own runner. One option: one connection per session, and treat a step as "blocked" if it has not answered within a timeout, or expose a custom "is session X waiting on a lock?" query. Timeout-based detection is slower and can be flaky. A server-side wait query is exact but is extra work.

### 3.2 Hermitage: the expected anomaly table

- Source: [repo](https://github.com/ept/hermitage).
- What it is: hand-run test scripts with 2–3 interleaved sessions (e.g. psql in separate terminals). It is "an attempt to nail down precisely what different database systems actually mean with their isolation levels."
- The PostgreSQL results define the behaviour to **match**:

| PG level | Actually | G0 | G1a | G1b | G1c | OTV | PMP | P4 (lost update) | G-single (read skew) | G2-item (write skew) | G2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Read Committed | monotonic atomic view | prevented | prevented | prevented | prevented | prevented | **allowed** | **allowed** | **allowed** | **allowed** | **allowed** |
| Repeatable Read | snapshot isolation | prevented | prevented | prevented | prevented | prevented | prevented | prevented | prevented | **allowed** | **allowed** |

- **Applicability:**
  - The scripts translate almost directly into isolation-spec permutations, multi-`connection` SLT files, or goldenscripts.
  - Tests should assert that the "allowed" anomalies *are possible*, not only that the prevented ones are blocked. Otherwise an implementation that is accidentally too strict, e.g. serializing everything, passes.
  - toydb does this. Its `src/storage/testscripts/mvcc/` has `anomaly_dirty_read`, `anomaly_dirty_write`, `anomaly_fuzzy_read`, `anomaly_lost_update`, `anomaly_phantom_read`, `anomaly_read_skew`, and `anomaly_write_skew`. The write-skew script **asserts the anomaly happens**: "Snapshot isolation does not prevent this, which is expected, so we assert the anomalous behavior." ([source](https://github.com/erikgrinaker/toydb/blob/main/src/storage/testscripts/mvcc/anomaly_write_skew))

### 3.3 Jepsen-style black-box checking: Elle

- **Elle** ([repo](https://github.com/jepsen-io/elle)):
  - "a transactional consistency checker for black-box databases".
  - It finds "every (non-predicate) anomaly" from Adya et al. (G0, G1a, G1b, G1c, G-single, G2…) from client-observed histories.
  - It reports the weakest violated consistency model (read committed, repeatable read, snapshot isolation, serializable…).
  - It is written in Clojure and runs on the JVM.
- Two workload models:
  - `list-append`: objects are lists, and writes append unique elements. This gives the strongest inference.
  - `rw-register`: blind writes, which give weaker inference.
- **elle-cli** ([repo](https://github.com/ligurio/elle-cli)) is a standalone CLI that needs **Java 21+**.
  - Run with `java -jar elle-cli-….jar --model list-append history.edn`. It prints `true`, `false`, or `unknown`.
  - History entries are EDN or JSON, e.g. `{:index 2 :type :invoke :value [[:append 255 8] [:r 253 nil]]}` followed by `{:type :ok :value [[:append 255 8] [:r 253 [1 3 4]]]}`.
- **What you'd build:**
  - A Rust load generator. N concurrent clients run random transactions over the wire, e.g. append via `UPDATE … SET v = v || ',8'` and read via `SELECT`.
  - It logs invoke, ok, fail, and info records, and the history goes to elle-cli.
- **Trade-offs:**
  - It finds anomalies you didn't think to write a test for, and it checks the whole stack.
  - Costs: a JVM dependency, writing the history recorder, results that are probabilistic (not exhaustive), and non-predicate anomalies only, so it won't check phantoms/G2 on predicates.
  - Elle checks *consistency models*. Postgres RC is "monotonic atomic view" per Hermitage, so interpret the "weakest violated model" output with that in mind.

### 3.4 In-process interleavings: `loom` 0.7.3

- Source: [docs](https://docs.rs/loom/latest/loom/).
- It "runs each test many times under different thread schedules, permuting executions according to the C11 memory model".
- Usage: wrap the test in `loom::model(|| …)`, use `loom::sync`/`loom::thread` replacement types switched with `#[cfg(loom)]`, and run with `RUSTFLAGS="--cfg loom"`.
- Limitations stated in the docs:
  - "Any code that does not use loom's replacement types is invisible to loom."
  - State space explodes. `LOOM_MAX_PREEMPTIONS` of 2–3 "is enough to catch most bugs".
  - Not every relaxed-ordering behaviour can be modelled.
  - The thread count is capped.
  - RNG and syscalls must be mocked.
- **Applicability:** small concurrent primitives such as a page latch, buffer-pool pin counts, a lock-manager queue, or a transaction-ID allocator. It is **not** a tool for SQL isolation, and it can't drive a whole server with real I/O.

### 3.5 Deterministic simulation: madsim, turmoil

- **madsim** 0.2.34 ([repo](https://github.com/madsim-rs/madsim)):
  - A tokio-like runtime. Built with `RUSTFLAGS="--cfg madsim"` and drop-in crates such as `madsim-tokio`.
  - Its API can kill processes, disconnect the network, and inject failures. RisingWave "uses MadSim for deterministic testing".
  - "All I/O-related interfaces must be mocked during the simulation."
- **turmoil** 0.7.2 ([repo](https://github.com/tokio-rs/turmoil)):
  - "runs multiple concurrent hosts within a single thread".
  - Faults include latency, drops, partitions, crashes, and torn writes, "under manual control or a seeded RNG". Has the `turmoil-fs` crate (§2.2).
- **Trade-off:** reproducible from a seed, and covers network, crash, and disk in one framework. But it is designed for distributed systems, and it constrains the architecture: everything must go through the simulated async runtime and fs. That is a heavy commitment for a single-node learning project. The same idea works on a smaller scale with your own I/O trait (§2.2).

---

## 4. Property-based testing: `proptest` 1.12.0, `proptest-state-machine` 0.9.0

- proptest is "Hypothesis-like property-based testing and shrinking" ([docs](https://docs.rs/proptest/latest/proptest/)).
  - `proptest!` defines tests, strategies generate inputs, and `prop_compose!` builds custom strategies.
- **Failure persistence** ([book](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html)):
  - The *seed* of each failing case is stored under a `proptest-regressions` tree and replayed first on later runs.
  - "It is recommended to check these files in to your source control."
- **State-machine testing** ([book](https://proptest-rs.github.io/proptest/proptest/state-machine.html)), in the `proptest-state-machine` crate:
  - `ReferenceStateMachine` is the model, with `init_state`, `transitions`, `apply`, and optional `preconditions`.
  - `StateMachineTest` is the real system, with `init_test`, `apply` (post-condition asserts go here), `check_invariants` (runs after every transition), and optional `teardown`.
  - `prop_state_machine! { … sequential 1..20 => MyTest }`.
  - Shrinking removes trailing transitions, then shrinks individual transitions, then the initial state. Preconditions keep shrunk sequences valid.
  - Tuning: `PROPTEST_CASES`, `PROPTEST_VERBOSE`.
- **Fits for this project:**
  - B-tree vs `std::collections::BTreeMap`: random insert/delete/get/range-scan sequences, with invariants after each step (sorted keys, balanced, fill factor).
  - Encode/decode round-trips: tuple serialization, page layout, WAL records, wire-protocol messages.
  - WAL + crash: a state machine whose transitions include `Crash`. With a simulated disk (§2.2), apply "drop unsynced writes" and restart, then compare with a model that holds only committed state. This combines property testing with crash testing, and shrinking returns a minimal crash sequence.
  - MVCC: model transactions as a reference that applies snapshot rules. This is harder to model correctly than a B-tree.
- **Trade-off:** high bug-finding power per line of test, but it needs a correct model. Seed-based regressions can stop reproducing after the generator code changes.

---

## 5. Trade-offs relevant to the Testing strategy decision

1. **SLT in-process vs over the wire.**
   - In-process (`impl DB`) works before the wire protocol exists and is fast.
   - Over the wire (`sqllogictest-bin --engine postgres`) needs the simple query protocol and covers protocol bugs too.
   - The same `.slt` files work both ways, and against real Postgres as an oracle.
2. **SLT vs goldenscript.**
   - SLT is SQL only, standard, and portable to other engines.
   - goldenscript tests internals (MVCC store, B-tree, WAL) with custom commands, but every runner is code you write.
   - toydb uses goldenscript at all layers. DuckDB and RisingWave use SLT for SQL.
3. **Shared vs per-file database.**
   - A shared database needs cleanup discipline (RisingWave).
   - Per-file databases (`-j`) need `CREATE DATABASE`.
4. **Crash testing realism ladder:**
   1. kill -9: easy, but process-crash only, because the page cache survives.
   2. Failpoints: deterministic placement of crashes and errors.
   3. Simulated disk that drops unsynced writes: power-loss semantics. Requires owning the I/O abstraction.
   4. LazyFS: real fs, no code change, Linux only.
5. **Isolation testing ladder:**
   1. Hand-written interleavings (Hermitage-style), in SLT `connection` blocks or goldenscript. Deterministic, but no wait detection.
   2. Isolation-spec format with your own runner: adds blocking-aware permutations and needs a "blocked?" signal.
   3. Elle with a history-recording load generator: randomized black box, JVM needed.
   - `loom` sits beside these, for in-process primitives.
6. **Determinism.** Seeded simulation (turmoil/madsim or your own I/O trait) makes concurrency and crash failures reproducible, at an architectural cost. kill -9 timing and real-thread interleavings are not reproducible.

---

## 6. Not verified / open points

- `sqlite.org/sqllogictest/doc/trunk/about.wiki` (the canonical SLT format doc) returned a Fossil anti-robot page. Format details here come from the sqllogictest-rs parser source and the DuckDB docs instead.
- The isolation-tester README fetched says blocking is detected through `pg_locks`. Whether the implementation uses a helper function, as opposed to querying `pg_locks` directly, was not confirmed.
- fail-rs: full action syntax (e.g. `pause`, `off`, `N%return`) was not checked against docs.rs. Only panic, return, sleep, and conditional/probabilistic triggering are confirmed.
- turmoil-fs: confirmed to exist as crate 0.1.0. Its API and maturity were not evaluated.
- Elle's handling of Postgres-style Read Committed (monotonic atomic view) was not checked against Elle's `consistency-model` source.
