# Research: Learning resources per layer

Resolves issue #4 ("Learning resources"), a child of map issue #1. The question: which primary learning resources should back each layer of the database, ordered for a learner who writes Rust (but still fights ownership and lifetimes) and has no database-internals background.

Researched 2026-11-19. Every link below was fetched on that date and returned HTTP 200, unless it is marked otherwise. Crate versions come from the crates.io API on that date.

---

## 0. How to use this map

**Per-layer pattern.** Each layer lists up to four kinds of source, read in this order:

1. **Concept**: a CMU 15-445 lecture, which explains the idea in a system-neutral way.
2. **How Postgres does it**: a chapter of *The Internals of PostgreSQL* (interdb) or a page of the PostgreSQL 18 docs.
3. **Authoritative detail**: the Postgres source README. Read it when you implement the layer, not before.
4. **Rust code to read**: real code with the same shape as yours, where any exists.

**Fit warnings.** Some sources teach a design that differs from this project's design (decisions in #1 and #6). Those mismatches are flagged with **Fit:** notes. Each one says which parts to skip, so you don't build machinery the design doesn't need.

**Version pins.** Section numbers drift between releases, so this map pins versions.

- **CMU 15-445/645 = the Fall 2025 archive (Andy Pavlo).** All lecture numbers below refer to that offering. Its videos, notes and projects are complete.
  - The current offering is [Fall 2026](https://15445.courses.cs.cmu.edu/fall2026/schedule.html), which is still in progress.
  - Fall 2026 renumbers the lectures: #12 there is "DRC", so Joins becomes #13 and every later lecture shifts by one. Don't mix numbers between the two.
- **PostgreSQL docs = version 18.** Today `docs/current` points to 18. Section numbers such as 54.x and 66.6 are specific to 18.
- **Postgres source READMEs = `master`** on GitHub.

## 1. The core resources

| Resource | What it is | Role in this map | Link |
|---|---|---|---|
| CMU 15-445/645, Fall 2025 | Intro DB internals course: 25 lectures with video, notes PDFs and slides; projects in C++ (bustub) | **Concept** layer for almost every topic | [schedule](https://15445.courses.cs.cmu.edu/fall2025/schedule.html), [YouTube playlist](https://www.youtube.com/playlist?list=PLSE8ODhjZXjYMAgsGH-GtY5rJYZ6zjsd5), [projects](https://15445.courses.cs.cmu.edu/fall2025/assignments.html) |
| *The Internals of PostgreSQL* (Hironobu Suzuki, interdb.jp) | Free online book on how Postgres actually works, updated through 2026 | **How Postgres does it**. Main source for heap layout, MVCC xmin/xmax, VACUUM, buffer manager and WAL | [interdb.jp/pg](https://www.interdb.jp/pg/) |
| PostgreSQL 18 documentation | Official manual | Wire protocol spec, page layout, isolation semantics, EXPLAIN, internals overview | [postgresql.org/docs/current](https://www.postgresql.org/docs/current/) |
| Postgres source READMEs | Design notes written by the developers next to the code | **Authoritative detail** at implementation time | [src/backend on GitHub](https://github.com/postgres/postgres/tree/master/src/backend) |
| *Database Internals* (Alex Petrov, O'Reilly 2019) | Book. Part I covers storage engines (ch. 1–7); Part II covers distributed systems (ch. 8–14) | Second explanation of B-trees, file formats, buffer/WAL/recovery. **Part II is out of scope.** | [databass.dev](https://www.databass.dev/) (the O'Reilly page blocks automated fetches; the table of contents was read through the O'Reilly API) |
| toyDB (Erik Grinaker) | Educational distributed SQL DB in Rust, with a bottom-up architecture guide | **Rust code to read** for lexer, parser, planner, optimizer, executor and sessions | [repo](https://github.com/erikgrinaker/toydb), [architecture guide](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/index.md) |
| *Architecture of a Database System* (Hellerstein, Stonebraker, Hamilton, 2007) | Survey paper describing the whole DBMS: process model, parser, optimizer, executor, storage, transactions | **Orientation** before any layer | [PDF](https://dsf.berkeley.edu/papers/fntdb07-architecture.pdf) |

**Petrov, Part I chapters** (from the O'Reilly table of contents):

1. Introduction and Overview
2. B-Tree Basics
3. File Formats
4. Implementing B-Trees
5. Transaction Processing and Recovery
6. B-Tree Variants
7. Log-Structured Storage

**CMU 15-445, Fall 2025 lectures** used below. Videos are on YouTube; notes PDFs are under `https://15445.courses.cs.cmu.edu/fall2025/notes/`.

| # | Topic | Video | Notes |
|---|---|---|---|
| 01 | Relational Model & Algebra | [video](https://www.youtube.com/watch?v=7NPIENPr-zk) | [01-relationalmodel.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/01-relationalmodel.pdf) |
| 03 | Database Storage I | [video](https://www.youtube.com/watch?v=PRLXdIMJhOg) | [03-storage1.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/03-storage1.pdf) |
| 04 | Memory Management (buffer pool) | [video](https://www.youtube.com/watch?v=8-2yv4z0VZc) | [04-bufferpool.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/04-bufferpool.pdf) |
| 05 | Database Storage II | [video](https://www.youtube.com/watch?v=2_sTdS4h-bY) | [05-storage2.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/05-storage2.pdf) |
| 08 | Indexes & Filters I (B+trees) | [video](https://www.youtube.com/watch?v=u7ii_Lvm9rM) | [08-indexes1.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/08-indexes1.pdf) |
| 09 | Indexes & Filters II | [video](https://www.youtube.com/watch?v=PjST2n7abAY) | [09-indexes2.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/09-indexes2.pdf) |
| 10 | Index Concurrency Control (latches) | [video](https://www.youtube.com/watch?v=YgOvfXl6pss) | [10-indexconcurrency.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/10-indexconcurrency.pdf) |
| 11 | Sorting & Aggregations | [video](https://www.youtube.com/watch?v=LzyKTpeIgts) | [11-sorting.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/11-sorting.pdf) |
| 12 | Join Algorithms | [video](https://www.youtube.com/watch?v=YIdIaPopfpk) | [12-joins.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/12-joins.pdf) |
| 13 | Query Execution I | [video](https://www.youtube.com/watch?v=E-UUd6cB57w) | [13-queryexecution1.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/13-queryexecution1.pdf) |
| 14 | Query Execution II | [video](https://www.youtube.com/watch?v=Kzf1hGjtZOU) | [14-queryexecution2.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/14-queryexecution2.pdf) |
| 15 | Query Planning & Optimization I | [video](https://www.youtube.com/watch?v=b53huOGcsZ8) | [15-optimization1.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/15-optimization1.pdf) |
| 16 | Query Planning & Optimization II | [video](https://www.youtube.com/watch?v=azTHRpzl10o) | [16-optimization2.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/16-optimization2.pdf) |
| 17 | Concurrency Control Theory | [video](https://www.youtube.com/watch?v=tMFAgvDViAI) | [17-concurrencycontrol.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/17-concurrencycontrol.pdf) |
| 18 | Two-Phase Locking | [video](https://www.youtube.com/watch?v=drStlhNbfHI) | [18-twophaselocking.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/18-twophaselocking.pdf) |
| 20 | Multi-Version Concurrency Control | [video](https://www.youtube.com/watch?v=tUFha9-DuSk) | [20-multiversioning.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/20-multiversioning.pdf) |
| 21 | Database Logging | [video](https://www.youtube.com/watch?v=CedEy54pe3g) | [21-logging.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/21-logging.pdf) |
| 22 | Database Recovery | [video](https://www.youtube.com/watch?v=X2jc4qalNy0) | [22-recovery.pdf](https://15445.courses.cs.cmu.edu/fall2025/notes/22-recovery.pdf) |

**Skippable lectures:**

| # | Topic | Why it's skippable |
|---|---|---|
| 06 | Storage Models & Compression | Column stores and compression are out of scope |
| 07 | Hash Tables | Useful only if you build a hash join or hash aggregate |
| 19 | Timestamp Ordering | Postgres does not use it |
| 23–24 | Distributed | Out of scope |

**The course textbook** is Silberschatz et al., *Database System Concepts*, 7th ed. ([syllabus](https://15445.courses.cs.cmu.edu/fall2025/syllabus.html)). Each lecture lists chapter readings on the schedule page. It is optional; the notes PDFs are enough.

**The CMU projects** are in C++ ([bustub](https://github.com/cmu-db/bustub)). You won't build them, but their write-ups work well as **specs** for your own Rust components:

- [P1 Buffer Pool Manager](https://15445.courses.cs.cmu.edu/fall2025/project1/)
- [P2 B+Tree](https://15445.courses.cs.cmu.edu/fall2025/project2/)
- [P3 Query Execution](https://15445.courses.cs.cmu.edu/fall2025/project3/)
- [P4 Concurrency Control](https://15445.courses.cs.cmu.edu/fall2025/project4/)

---

## 2. Per-layer reading map (beginner order)

The order follows the map's working rule in #1: **single-threaded first, and concurrency only after storage and MVCC work.** The map is still deciding the final milestone order, so treat this order as a suggested reading sequence, not as a roadmap decision.

### Layer 0: Orientation (read before anything else)

1. CMU [#01 Relational Model & Algebra](https://www.youtube.com/watch?v=7NPIENPr-zk). It gives the relational-algebra vocabulary (select, project, join) that planners and executors are built from.
2. PG 18 docs [Chapter 51, Overview of PostgreSQL Internals](https://www.postgresql.org/docs/current/overview.html), in particular [51.1 The Path of a Query](https://www.postgresql.org/docs/current/query-path.html). It is a short end-to-end tour that covers connection, then parser, then rewriter, then planner, then executor.
3. Hellerstein et al., [*Architecture of a Database System*](https://dsf.berkeley.edu/papers/fntdb07-architecture.pdf), sections 1–2 and the overview of each later section. It names every component you will build and shows how they fit together.
4. interdb [Ch. 1: Database Cluster, Databases and Tables](https://www.interdb.jp/pg/pgsql01/index.html) and [Ch. 2: Process and Memory Architecture](https://www.interdb.jp/pg/pgsql02/index.html). These chapters show what "Postgres-style" means concretely: one process per backend, shared buffers, and OIDs.

### Layer 1: Client/server and the Postgres wire protocol

| Kind | Source |
|---|---|
| Spec (primary) | PG 18 docs [Ch. 54 Frontend/Backend Protocol](https://www.postgresql.org/docs/current/protocol.html), read in this order: [54.1 Overview](https://www.postgresql.org/docs/current/protocol-overview.html), [54.2 Message Flow](https://www.postgresql.org/docs/current/protocol-flow.html) (§[Start-up](https://www.postgresql.org/docs/current/protocol-flow.html#PROTOCOL-FLOW-START-UP), §[Simple Query](https://www.postgresql.org/docs/current/protocol-flow.html#PROTOCOL-FLOW-SIMPLE-QUERY); §[Extended Query](https://www.postgresql.org/docs/current/protocol-flow.html#PROTOCOL-FLOW-EXT-QUERY) only later), then [54.7 Message Formats](https://www.postgresql.org/docs/current/protocol-message-formats.html) as a byte-level reference |
| Rust async server basics | [Tokio tutorial](https://tokio.rs/tokio/tutorial). Accepting connections, framing and per-connection tasks are exactly what a wire server needs |
| Rust code to read | [`pgwire` crate](https://github.com/sunng87/pgwire) (0.42.3, [docs.rs](https://docs.rs/pgwire/latest/pgwire/)) implements the protocol for servers. Read it to check your understanding of message framing; whether to depend on it is a separate decision |

There is no CMU lecture for this layer. The PG docs chapter is both the tutorial and the spec. A sibling research branch, `research/psql-wire-protocol`, already covers this layer in depth.

### Layer 2: SQL parser

| Kind | Source |
|---|---|
| Concept (parsing techniques) | [*Crafting Interpreters*](https://craftinginterpreters.com/), the "Scanning", "Representing Code" and "Parsing Expressions" chapters. They teach a lexer, an AST and a recursive-descent parser from zero |
| Expression precedence in Rust | matklad, [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html). It is written in Rust and handles `a + b * c AND d = e` cleanly |
| How Postgres does it | PG 18 docs [51.3 The Parser Stage](https://www.postgresql.org/docs/current/parser-stage.html). Note the split between the *raw parse tree*, which has no catalog lookups, and the *transformation* step, which resolves names against the catalog. Copy that split. [`parser/README`](https://github.com/postgres/postgres/blob/master/src/backend/parser/README) is a 35-line map of the files |
| Rust code to read | toyDB [Parsing: Lexer, AST, Parser](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/sql-parser.md). This is the closest match to what you will write. [`sqlparser` crate](https://github.com/apache/datafusion-sqlparser-rs) (0.64.0) is a production-grade alternative to read or adopt |

### Layer 3: Heap pages, tuples and TIDs (disk storage)

| Kind | Source |
|---|---|
| Concept | CMU [#03 Database Storage I](https://www.youtube.com/watch?v=PRLXdIMJhOg) (pages, slotted pages, tuple layout), then [#05 Database Storage II](https://www.youtube.com/watch?v=2_sTdS4h-bY) |
| How Postgres does it | interdb [1.3 Heap Table Structure](https://www.interdb.jp/pg/pgsql01/03.html) and [1.4 Writing and Reading Tuples](https://www.interdb.jp/pg/pgsql01/04.html). Then PG 18 docs [66.6 Database Page Layout](https://www.postgresql.org/docs/current/storage-page-layout.html), the exact byte layout (PageHeaderData, ItemId, HeapTupleHeader) |
| Authoritative detail | [`storage/page/README`](https://github.com/postgres/postgres/blob/master/src/backend/storage/page/README) (short; mostly about checksums) |
| Second explanation | Petrov ch. 3 "File Formats" (slotted pages, cells, variable-size data) |

**Fit:** decision #6 drops TOAST, the free space map (FSM), the visibility map (VM) and HOT. Skip interdb §1.3.2 (TOAST) and the FSM and VM material.

### Layer 4: Buffer pool

| Kind | Source |
|---|---|
| Concept | CMU [#04 Memory Management](https://www.youtube.com/watch?v=8-2yv4z0VZc) (buffer pool, page table, pin counts, LRU / LRU-K / clock) |
| Spec to implement against | CMU [Project 1: Buffer Pool Manager](https://15445.courses.cs.cmu.edu/fall2025/project1/) |
| How Postgres does it | interdb [Ch. 8 Buffer Manager](https://www.interdb.jp/pg/pgsql08/index.html): 8.1–8.4 and 8.6. Skip 8.5 Asynchronous I/O |
| Authoritative detail | [`storage/buffer/README`](https://github.com/postgres/postgres/blob/master/src/backend/storage/buffer/README): "Notes About Shared Buffer Access Rules" (pins vs content locks) and "Normal Buffer Replacement Strategy" (clock sweep) |
| Second explanation | Petrov ch. 5, "Buffer Management" section |
| Rust ownership help | [*Learning Rust With Entirely Too Many Linked Lists*](https://rust-unofficial.github.io/too-many-lists/). It drills `Rc`/`RefCell`, raw pointers and unsafe aliasing, which is exactly where page frames and guards cause ownership fights |

The buffer pool is where the Rust ownership fights will be worst. A pinned page frame that is borrowed while the pool also holds it is a classic `&mut` aliasing problem. The pin and content-lock rules in `buffer/README` map closely onto RAII "page guard" types in Rust.

### Layer 5: System catalog

| Kind | Source |
|---|---|
| How Postgres does it | PG 18 docs [Ch. 52 System Catalogs](https://www.postgresql.org/docs/current/catalogs.html). Read only the overview plus `pg_class`, `pg_attribute`, `pg_type`, `pg_index` and `pg_namespace` |
| Background | interdb [1.1 Logical Structure](https://www.interdb.jp/pg/pgsql01/01.html) (OIDs and how relations are named) |
| Rust code to read | toyDB [Storage: Schema Catalog](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/sql-storage.md#schema-catalog) |

There is no CMU lecture that covers catalogs as tables. Bootstrap (how `pg_class` describes itself) is still an open question in #1. No beginner source covers Postgres bootstrap well; the real mechanism (`genbki`/`BKI` files) is overkill.

**Fit:** toyDB stores its schemas as key/value entries, not as heap tables. Use it for the shape of the API, not for the storage approach.

### Layer 6: Executor

| Kind | Source |
|---|---|
| Concept | CMU [#13 Query Execution I](https://www.youtube.com/watch?v=E-UUd6cB57w) (iterator/Volcano model, access methods) and [#14 Query Execution II](https://www.youtube.com/watch?v=Kzf1hGjtZOU). For the operators themselves: [#11 Sorting & Aggregations](https://www.youtube.com/watch?v=LzyKTpeIgts) and [#12 Join Algorithms](https://www.youtube.com/watch?v=YIdIaPopfpk) |
| Spec | CMU [Project 3: Query Execution](https://15445.courses.cs.cmu.edu/fall2025/project3/) |
| How Postgres does it | PG 18 docs [51.6 Executor](https://www.postgresql.org/docs/current/executor.html). interdb [3.4 Executor Performance](https://www.interdb.jp/pg/pgsql03/04.html) and [3.5 Join Algorithms](https://www.interdb.jp/pg/pgsql03/05/index.html) (nested loop, merge, hash) |
| Authoritative detail | [`executor/README`](https://github.com/postgres/postgres/blob/master/src/backend/executor/README): "Plan Trees and State Trees" and "Query Processing Control Flow". Save "EvalPlanQual" for Layer 9 |
| Rust code to read | toyDB [Execution](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/sql-execution.md) (plan executor, session management) |

### Layer 7: B-tree index (single-threaded first)

| Kind | Source |
|---|---|
| Concept | CMU [#08 Indexes & Filters I](https://www.youtube.com/watch?v=u7ii_Lvm9rM) (B+tree insert, split, delete) and [#09](https://www.youtube.com/watch?v=PjST2n7abAY) |
| Spec | CMU [Project 2: B+Tree](https://15445.courses.cs.cmu.edu/fall2025/project2/) |
| Hands-on walkthrough | cstack, [Let's Build a Simple Database](https://cstack.github.io/db_tutorial/), Parts 7–14: leaf format, binary search, leaf and internal splits, all on fixed pages. It is in C and stores a SQLite-style table B-tree, but the page-level split mechanics transfer directly |
| Second explanation | Petrov ch. 2 "B-Tree Basics" and ch. 4 "Implementing B-Trees" |
| How Postgres does it | PG 18 docs [65.1 B-Tree Indexes](https://www.postgresql.org/docs/current/btree.html) |
| Authoritative detail | [`access/nbtree/README`](https://github.com/postgres/postgres/blob/master/src/backend/access/nbtree/README) (1,083 lines). Read "The basic Lehman & Yao Algorithm" and "Differences to the Lehman & Yao algorithm" now. Read "Deleting index tuples during VACUUM" in Layer 11 |

**Fit:** the map's decision #6 picks a Lehman–Yao nbtree, where leaf entries point to heap TIDs. CMU's B+tree is generic. Postgres-specific features in `nbtree/README` are not needed here: deduplication, suffix truncation and bottom-up deletion. Skip them.

### Layer 8: Rule-based planner and EXPLAIN

| Kind | Source |
|---|---|
| Concept | CMU [#15 Query Planning & Optimization I](https://www.youtube.com/watch?v=b53huOGcsZ8). The first half covers heuristic/rule-based rewrites (predicate pushdown, projection pushdown); the cost-based part is "later" |
| Rust code to read (best fit) | toyDB [Planning](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/sql-planner.md) (scope and name resolution) and [Optimization](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/sql-optimizer.md) (constant folding, filter pushdown, index lookups, hash join). This is a rule-based optimizer, which is the same design this project uses |
| EXPLAIN target output | PG 18 docs [14.1 Using EXPLAIN](https://www.postgresql.org/docs/current/using-explain.html). This is what your EXPLAIN should resemble |
| How Postgres does it | PG 18 docs [51.5 Planner/Optimizer](https://www.postgresql.org/docs/current/planner-optimizer.html). interdb [3.3 Creating the Plan Tree of a Single-Table Query](https://www.interdb.jp/pg/pgsql03/03.html) (shows when index scan beats seq scan) |

**Later, cost-based only:**

- CMU [#16](https://www.youtube.com/watch?v=azTHRpzl10o)
- interdb [3.2 Cost Estimation](https://www.interdb.jp/pg/pgsql03/02.html)
- [`optimizer/README`](https://github.com/postgres/postgres/blob/master/src/backend/optimizer/README) (1,621 lines, almost all about Paths and cost-based join search)

**Fit:** Postgres itself is cost-based, so its planner sources overshoot a rule-based design.

### Layer 9: MVCC (xmin/xmax) with Read Committed and Repeatable Read

| Kind | Source |
|---|---|
| Concept | CMU [#17 Concurrency Control Theory](https://www.youtube.com/watch?v=tMFAgvDViAI) (ACID, anomalies, schedules), then [#20 Multi-Version Concurrency Control](https://www.youtube.com/watch?v=tUFha9-DuSk) |
| **Primary: how Postgres does it** | interdb [Ch. 5 Concurrency Control](https://www.interdb.jp/pg/pgsql05/index.html), sections 5.1–5.8: [5.1 Transaction ID](https://www.interdb.jp/pg/pgsql05/01.html), [5.2 Tuple Structure](https://www.interdb.jp/pg/pgsql05/02.html), [5.3 Insert/Delete/Update](https://www.interdb.jp/pg/pgsql05/03.html), [5.4 Commit Log (clog)](https://www.interdb.jp/pg/pgsql05/04.html), [5.5 Transaction Snapshot](https://www.interdb.jp/pg/pgsql05/05.html), [5.6 Visibility Check Rules](https://www.interdb.jp/pg/pgsql05/06.html), [5.7 Visibility Check](https://www.interdb.jp/pg/pgsql05/07.html), [5.8 Preventing Lost Updates](https://www.interdb.jp/pg/pgsql05/08.html). This is the single most important reading in the project |
| Isolation semantics (the spec) | PG 18 docs [13.2 Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html) (13.2.1 Read Committed, 13.2.2 Repeatable Read). Background: [Chapter 13 Concurrency Control](https://www.postgresql.org/docs/current/mvcc.html) |
| The subtle part of RC | [`executor/README`](https://github.com/postgres/postgres/blob/master/src/backend/executor/README), section "EvalPlanQual (READ COMMITTED Update Checking)". It explains what an UPDATE in RC does when the target row was changed by a transaction that has since committed. interdb 5.8 covers the same case from the visibility side |
| Authoritative detail | [`access/transam/README`](https://github.com/postgres/postgres/blob/master/src/backend/access/transam/README): "The Transaction System", "Interlocking Transaction Begin, Transaction End, and Snapshots", "pg_xact and pg_subtrans" |
| Anomaly definitions | Berenson et al., [A Critique of ANSI SQL Isolation Levels](https://arxiv.org/abs/cs/0701157) (arXiv). It defines the anomalies (lost update, write skew, and so on) that tell RC and RR apart. [Hermitage](https://github.com/ept/hermitage) (also chosen as the test target in #7) shows which anomalies Postgres allows at each level |
| Rust code to read | toyDB [MVCC Transactions](https://github.com/erikgrinaker/toydb/blob/main/docs/architecture/mvcc.md) |

**Key idea for RC vs RR.** Read Committed takes a new snapshot **per statement**. Repeatable Read takes one snapshot **per transaction** and keeps it (interdb 5.5).

**Skip:**

- interdb 5.9 (SSI): Serializable is out of scope in #1.
- The `heap/README.tuplock` MultiXact section: dropped in #6. The "Locking tuples" and "Infomask Bits" sections are useful once row-level locks (`SELECT ... FOR UPDATE`, concurrent UPDATE) arrive in Layer 12.

**Fit for CMU:** CMU #20 and Project 4 present generic MVCC designs (version chains, timestamp ordering, a separate version store). Postgres uses append-only heap versions with xmin/xmax and clog. Use CMU for the concepts and interdb for the actual design.

**Fit for toyDB:** toyDB's MVCC is snapshot isolation over versioned keys in a KV store, with no xmin/xmax tuple headers and no clog. Read it for the snapshot and visibility logic only.

### Layer 10: WAL and crash recovery (ARIES-lite)

| Kind | Source |
|---|---|
| Concept | CMU [#21 Database Logging](https://www.youtube.com/watch?v=CedEy54pe3g) (write-ahead rule, steal/no-force, physical vs logical vs physiological logging, checkpoints), then [#22 Database Recovery](https://www.youtube.com/watch?v=X2jc4qalNy0) (ARIES) |
| How Postgres does it | interdb [Ch. 9 Write-Ahead Logging](https://www.interdb.jp/pg/pgsql09/index.html), sections 9.1–9.8, in particular [9.4 Internal Layout of XLOG Record](https://www.interdb.jp/pg/pgsql09/04.html), [9.5 Writing of XLOG Records](https://www.interdb.jp/pg/pgsql09/05.html), [9.7 Checkpoint Processing](https://www.interdb.jp/pg/pgsql09/07.html), and [9.8 Database Recovery](https://www.interdb.jp/pg/pgsql09/08.html) (with 9.8.2 "LSN Comparison and Idempotency", which is how `pd_lsn` makes redo safe). Skip 9.9–9.10 (segment management, archiving) and Ch. 10–12 (backup, replication) |
| PG docs | [28.3 Write-Ahead Logging](https://www.postgresql.org/docs/current/wal-intro.html), [28.6 WAL Internals](https://www.postgresql.org/docs/current/wal-internals.html) |
| Authoritative detail | [`access/transam/README`](https://github.com/postgres/postgres/blob/master/src/backend/access/transam/README): "Write-Ahead Log Coding", "Constructing a WAL record", "Writing a REDO routine". Also the `buffer/README` rule that a dirty page cannot be written before WAL is flushed up to its LSN |
| Second explanation | Petrov ch. 5, "Recovery" section (ARIES, steal/force) |
| Original paper (optional, last) | Mohan et al., [ARIES (1992)](https://cs.stanford.edu/people/chrismre/cs345/rl/aries.pdf). Long and dense; read it only after the CMU lecture |

**What "ARIES-lite" means for this design.** interdb 9.8 states that "PostgreSQL implements redo log-based recovery": it replays XLOG records from the REDO point. Aborted transactions need no heap undo pass. Their tuples stay on the page and are made invisible by the transaction's aborted status in clog (interdb 5.4, 5.6), then removed later by VACUUM.

So from CMU #22 and the ARIES paper, take:

- **Learn:** LSNs, `pageLSN`, the WAL-before-data rule, checkpoints, and the redo phase.
- **Skip for implementation:** the undo phase and compensation log records (CLRs). The relevant open question in #1 ("ARIES-lite redo/undo") resolves toward redo-only if you follow Postgres.

### Layer 11: VACUUM

| Kind | Source |
|---|---|
| How Postgres does it (primary) | interdb [Ch. 6 VACUUM Processing](https://www.interdb.jp/pg/pgsql06/index.html): [6.1 Outline of VACUUM Processing](https://www.interdb.jp/pg/pgsql06/01.html) (the three phases: collect dead TIDs, remove index entries, then remove heap tuples). Also interdb [5.10 Maintenance Processes](https://www.interdb.jp/pg/pgsql05/10.html) |
| PG docs | [24.1 Routine Vacuuming](https://www.postgresql.org/docs/current/routine-vacuuming.html) (why dead tuples exist; the user-facing view) |
| Authoritative detail | [`access/nbtree/README`](https://github.com/postgres/postgres/blob/master/src/backend/access/nbtree/README): "Deleting index tuples during VACUUM" and "Making concurrent TID recycling safe". The key point is that index entries must be removed *before* the heap slot can be reused for a new TID |
| Concept | CMU #20 MVCC, "garbage collection" part |

**Fit:** decision #6 drops freezing (64-bit XIDs), the visibility map and HOT. Skip:

- interdb 6.2 Visibility Map
- interdb 6.3 Freeze
- interdb 6.4 Clog removal
- interdb Ch. 7 HOT
- [`heap/README.HOT`](https://github.com/postgres/postgres/blob/master/src/backend/access/heap/README.HOT)

Autovacuum (6.5) is optional.

### Layer 12: Concurrency: locks, latches, concurrent B-tree

This is the last layer, because #1 says to add concurrency only after storage and MVCC work single-threaded.

| Kind | Source |
|---|---|
| Concept: locks vs latches | CMU [#10 Index Concurrency Control](https://www.youtube.com/watch?v=YgOvfXl6pss), which opens with the locks-vs-latches distinction, then latch crabbing and B-link trees. Also CMU [#18 Two-Phase Locking](https://www.youtube.com/watch?v=drStlhNbfHI) (lock modes, deadlock detection) |
| Postgres heavyweight locks (= "locks") | [`storage/lmgr/README`](https://github.com/postgres/postgres/blob/master/src/backend/storage/lmgr/README): "Locking Overview", "Lock Data Structures", "The Deadlock Detection Algorithm". PG 18 docs [13.3 Explicit Locking](https://www.postgresql.org/docs/current/explicit-locking.html) has the table-lock conflict matrix |
| Postgres buffer pins and content locks (= "latches") | [`storage/buffer/README`](https://github.com/postgres/postgres/blob/master/src/backend/storage/buffer/README): "Notes About Shared Buffer Access Rules" and "Buffer Manager's Internal Locking". interdb [8.3 Buffer Manager Locks](https://www.interdb.jp/pg/pgsql08/03.html) |
| Row locks | [`access/heap/README.tuplock`](https://github.com/postgres/postgres/blob/master/src/backend/access/heap/README.tuplock): "Locking tuples", "Infomask Bits" (skip MultiXacts) |
| Concurrent B-tree | Lehman & Yao, [Efficient Locking for Concurrent Operations on B-Trees (1981)](https://www.csd.uoc.gr/~hy460/pdf/p650-lehman.pdf), then the `nbtree/README` Lehman–Yao sections again, this time for the latching rules |
| Spec | CMU [Project 4: Concurrency Control](https://15445.courses.cs.cmu.edu/fall2025/project4/) (MVCC-based in Fall 2025; use it as a spec only) |
| Second explanation | Petrov ch. 5, concurrency-control part, which includes latches and B-link trees |

**Vocabulary map:**

| CMU term | Postgres term | What it protects | Held for |
|---|---|---|---|
| lock | heavyweight lock (lmgr) | logical objects: tables, rows | whole transaction |
| latch | buffer pin + content lock, or an LWLock | physical in-memory structures: pages, buffer table | a few instructions |

---

## 3. Other resources considered

| Resource | Verdict | Reason |
|---|---|---|
| [Let's Build a Simple Database](https://cstack.github.io/db_tutorial/) (cstack) | **Use for Layer 7 only** | Parts 7–14 are an excellent hands-on B-tree-on-pages walkthrough. Parts 1–6 are a toy REPL; the project is in C and unfinished (Part 15 "Where to go next") |
| [toyDB](https://github.com/erikgrinaker/toydb) | **Use for Layers 2, 5, 6, 8, 9** (Rust reference) | Actively maintained (last push 2026-10-02). Its storage (BitCask KV) and Raft chapters don't match a heap + WAL design. **Skip the Raft chapters and `storage.md`** |
| [Build Your Own Database From Scratch in Go](https://build-your-own.org/database/) | Optional | Copy-on-write B+tree with no WAL, which is a different durability model from Postgres. Only Part I is free |
| [RisingLight](https://github.com/risinglightdb/risinglight) | Skip | Rust OLAP / columnar educational DB; mismatched design |
| Petrov Part II (ch. 8–14) | Skip | Distributed systems, out of scope |
| interdb Ch. 4 (FDW), 10–12 (backup, replication) | Skip | Out of scope |

---

## 4. One-screen summary

| Layer | Read first (concept) | Then (how Postgres does it) | Implementation-time detail | Rust code |
|---|---|---|---|---|
| 0 Orientation | CMU #01 | PG docs ch. 51; Hellerstein paper | — | — |
| 1 Wire protocol | — | PG docs ch. 54 (54.1, 54.2, 54.7) | — | pgwire; Tokio tutorial |
| 2 Parser | Crafting Interpreters; Pratt parsing | PG docs 51.3 | `parser/README` | toyDB sql-parser; sqlparser-rs |
| 3 Heap pages | CMU #03, #05 | interdb 1.3–1.4; PG docs 66.6 | `storage/page/README` | — |
| 4 Buffer pool | CMU #04 + P1 | interdb ch. 8 | `storage/buffer/README` | too-many-lists (ownership) |
| 5 Catalog | — | PG docs ch. 52 | — | toyDB sql-storage |
| 6 Executor | CMU #13, #14 (+ #11, #12) + P3 | PG docs 51.6; interdb 3.4–3.5 | `executor/README` | toyDB sql-execution |
| 7 B-tree | CMU #08, #09 + P2; cstack 7–14 | PG docs 65.1 | `nbtree/README` (Lehman & Yao) | — |
| 8 Planner + EXPLAIN | CMU #15 (rule-based half) | PG docs 14.1, 51.5; interdb 3.3 | (`optimizer/README` is cost-based, later) | toyDB sql-planner, sql-optimizer |
| 9 MVCC RC/RR | CMU #17, #20 | **interdb ch. 5 (5.1–5.8)**; PG docs 13.2 | `transam/README`; `executor/README` EvalPlanQual | toyDB mvcc |
| 10 WAL + recovery | CMU #21, #22 | interdb ch. 9 (9.1–9.8) | `transam/README` WAL sections | — |
| 11 VACUUM | CMU #20 (GC) | interdb 6.1, 5.10; PG docs 24.1 | `nbtree/README` VACUUM sections | — |
| 12 Locks/latches | CMU #10, #18 + P4 | interdb 8.3; PG docs 13.3 | `lmgr/README`, `buffer/README`, `README.tuplock`; Lehman & Yao paper | — |
