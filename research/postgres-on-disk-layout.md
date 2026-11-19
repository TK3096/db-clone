# PostgreSQL on-disk layout (heap + nbtree), and what to mirror

Research for issue #6. Sources are PostgreSQL 17 (`REL_17_STABLE`), pinned to commit
`901eb483e0fff340ae3fe496ad2a2d1856f38693`. Links below use `SRC = https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src`.

Primary sources:

- [S1] PostgreSQL 17 docs, ch. 65 "Database Physical Storage": [65.1 File Layout](https://www.postgresql.org/docs/17/storage-file-layout.html), [65.3 FSM](https://www.postgresql.org/docs/17/storage-fsm.html), [65.4 VM](https://www.postgresql.org/docs/17/storage-vm.html), [65.6 Page Layout](https://www.postgresql.org/docs/17/storage-page-layout.html)
- [S2] [`include/storage/bufpage.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/storage/bufpage.h): page header and slotted page
- [S3] [`include/storage/itemid.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/storage/itemid.h): line pointer
- [S4] [`include/storage/itemptr.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/storage/itemptr.h) and [`block.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/storage/block.h): TID
- [S5] [`include/access/htup_details.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/access/htup_details.h): heap tuple header
- [S6] [`include/access/itup.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/access/itup.h): index tuple
- [S7] [`include/access/nbtree.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/access/nbtree.h) and [`backend/access/nbtree/README`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/backend/access/nbtree/README)
- [S8] [`include/common/relpath.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/common/relpath.h): fork numbers

Secondary explainer (for pictures only, not cited as the authority for any claim): Hironobu Suzuki, *The Internals of PostgreSQL*: [1.3 Internal layout of a heap table file](https://www.interdb.jp/pg/pgsql01/03.html), [5.2 Tuple structure](https://www.interdb.jp/pg/pgsql05/02.html), [7.1 HOT](https://www.interdb.jp/pg/pgsql07/01.html).

---

## TL;DR

- Every relation (table or index) is a file of fixed **8 KB pages**. Every page uses the same **slotted-page** format: a 24-byte header, an array of 4-byte **line pointers** growing down from the top, tuples growing up from the bottom, and an optional **special space** at the end for the access method [S1, S2].
- A row's address is a **TID** = (block number, line-pointer index). Indexes store TIDs. Tuples can move *within* a page without changing their TID, because the TID names the slot, not the byte offset [S2, S1].
- A heap tuple starts with a **23-byte header** holding MVCC data (`xmin`, `xmax`, `cid`), `t_ctid` (its own TID, or the TID of its newer version), and flag bits. An optional **null bitmap** follows [S5, S1].
- A B-tree index uses the same page format. It puts a 16-byte "opaque" struct (sibling links, level, flags) in the special space. Each index tuple is an 8-byte header (`heap TID`, size and flags) followed by the key [S6, S7].
- **Recommendation:** mirror the slotted page, the TID, `pd_lsn`, the line-pointer states, the core MVCC header fields, `t_ctid` chains, and Lehman-Yao B-tree pages (high key and right-link). Drop TOAST, FSM/VM forks, segments, combo CIDs, MultiXact, HOT, 32-bit XID wraparound and freezing, checksums, MAXALIGN, and nbtree dedup/suffix truncation. See [Recommended subset](#recommended-subset-for-this-project).

---

## 1. Files, forks, and segments

- Each table and each index is stored in its own file, named after `pg_class.relfilenode`. It lives under `PGDATA/base/<database OID>/` for the default tablespace [S1 §65.1].
- Each relation can have several **forks**, which are separate files sharing the filenode base name [S1 §65.1, S8]:

| ForkNumber | File | What it is |
|---|---|---|
| `MAIN_FORKNUM = 0` | `12345` | The actual pages |
| `FSM_FORKNUM = 1` | `12345_fsm` | Free Space Map (heap and most indexes) |
| `VISIBILITYMAP_FORKNUM = 2` | `12345_vm` | Visibility Map (heap only) |
| `INIT_FORKNUM = 3` | `12345_init` | Init fork for unlogged relations |

- **Segments:** once a fork passes 1 GB, it is split into `12345`, `12345.1`, `12345.2`, and so on. 1 GB is the build-time default (`--with-segsize`). The split exists to avoid OS file-size limits [S1 §65.1].
- The filenode is **not** necessarily the table OID. TRUNCATE, REINDEX, CLUSTER, and some forms of ALTER TABLE assign a new filenode [S1 §65.1].
- A block number is a `uint32` (`BlockNumber`). `0xFFFFFFFF` is `InvalidBlockNumber` [S4 block.h]. Byte offset of block *n* = `n * BLCKSZ` across the concatenated segments.

## 2. The 8 KB slotted page

The diagram below is from the `bufpage.h` header comment [S2], redrawn:

```
offset 0                                                        8192
+----------------+---------------------------------------------+
| PageHeaderData | linp1 linp2 linp3 ... linpN  -->             |
+----------------+---------------------------------------------+
|                         ^ pd_lower                            |
|                    (free space)                               |
|                         v pd_upper                            |
+----------------------+----------------------------------------+
|              <-- tupleN ... tuple3 tuple2 tuple1 | special    |
+-------------------------------------------------+------------+
                                                  ^ pd_special
```

- Line pointers grow **forward** from the header. Tuples are added **backwards** from `pd_special`. The page is full when nothing fits between `pd_lower` and `pd_upper` [S2].
- Heap pages have no special space (`pd_special = 8192`). B-tree pages put `BTPageOpaqueData` there [S2, S7].

### 2.1 `PageHeaderData`: 24 bytes [S2, S1 Table 65.2]

```
off  size  field                 meaning
 0    8    pd_lsn                LSN: next byte after the WAL record of the last change to this page
 8    2    pd_checksum           page checksum (if data checksums are enabled)
10    2    pd_flags              PD_HAS_FREE_LINES=0x1, PD_PAGE_FULL=0x2, PD_ALL_VISIBLE=0x4
12    2    pd_lower              offset to start of free space (= end of line-pointer array)
14    2    pd_upper              offset to end of free space (= start of the lowest tuple)
16    2    pd_special            offset to start of special space
18    2    pd_pagesize_version   page size | layout version (currently 4)
20    4    pd_prune_xid          oldest un-pruned XMAX on the page (hint), 0 if none
24         pd_linp[]             line pointers start here (SizeOfPageHeaderData = 24)
```

- Line-pointer count = `(pd_lower - 24) / 4`.
- `pd_lsn` is stored in C as two `uint32` halves for historical reasons. Logically it is one 64-bit LSN [S2].

### 2.2 Line pointer `ItemIdData`: 4 bytes [S3]

```
 bits:   lp_off:15 | lp_flags:2 | lp_len:15      (32 bits total)
```

| `lp_flags` | value | meaning |
|---|---|---|
| `LP_UNUSED` | 0 | free, may be reused immediately (`lp_len = 0`) |
| `LP_NORMAL` | 1 | points at a tuple (`lp_len > 0`) |
| `LP_REDIRECT` | 2 | HOT redirect: `lp_off` holds another line-pointer number (`lp_len = 0`) |
| `LP_DEAD` | 3 | tuple is dead. Storage may already be gone, but the slot cannot be reused yet |

- Offsets and lengths are 15 bits, which is why `LocationIndex` is effectively capped at 2^15 and pages cannot exceed 32 KB [S2].
- **Why the indirection matters:** a line pointer "is never moved until it is freed". Its index is a stable, long-lived reference to the item even when the tuple bytes are moved during page compaction [S1 §65.6, S2].

### 2.3 TID / `ItemPointerData`: 6 bytes [S4]

```
off size field
 0   2   ip_blkid.bi_hi   high 16 bits of BlockNumber
 2   2   ip_blkid.bi_lo   low  16 bits of BlockNumber
 4   2   ip_posid         OffsetNumber = 1-based index into the line-pointer array
```

- The block number is split into two `uint16` halves so the struct only needs 2-byte alignment and packs to 6 bytes. That matters because every heap tuple header and every index tuple carries one [S4].
- **OffsetNumbers start at 1, not 0.** `0` is `InvalidOffsetNumber` and `FirstOffsetNumber` is 1 ([`storage/off.h`](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/include/storage/off.h)). A TID such as `(0,1)` means block 0, first line pointer, which is exactly what `ctid` shows in SQL.

## 3. Heap tuples

Overall shape [S5, S1 §65.6.1]:

```
+---------------------------+------------------+---------+-----------+
| HeapTupleHeaderData (23B) | null bitmap (opt)| padding | user data |
+---------------------------+------------------+---------+-----------+
^ lp_off points here                                     ^ t_hoff
```

### 3.1 `HeapTupleHeaderData`: 23 bytes [S5, S1 Table 65.4]

```
off size field          meaning
 0   4   t_xmin         XID of the inserting transaction
 4   4   t_xmax         XID of the deleting/updating/locking transaction (0 = none)
 8   4   t_cid          command ID (cmin or cmax, or a combo CID) | t_xvac (union)
12   6   t_ctid         TID of this tuple, or of its newer version after UPDATE
18   2   t_infomask2    low 11 bits = number of attributes; high bits = HOT flags
20   2   t_infomask     flag bits (below)
22   1   t_hoff         offset to user data (header + bitmap + padding), MAXALIGN multiple
23       t_bits[]       null bitmap starts here, if present
```

**Five virtual fields in three physical slots.** Xmin and Xmax are always stored. Cmin, Cmax, and Xvac share `t_field3`. This works because cmin only matters to the inserting transaction and cmax only to the deleting one. When the same transaction inserts *and* deletes a row, a backend-local **combo CID** maps to the real (cmin, cmax) pair. Xvac belongs only to the pre-9.0 VACUUM FULL [S5].

**`t_ctid` and update chains** [S5]:

- On insert, `t_ctid` = the tuple's own TID.
- On UPDATE, the old version's `t_ctid` is set to the new version's TID.
- A tuple is the latest version iff `xmax` is invalid **or** `t_ctid` points to itself.
- When following a chain, check that `next.xmin == prev.xmax`. VACUUM may have freed the target slot and refilled it with an unrelated tuple.

**`t_infomask` bits you will actually reason about** [S5]:

| bit | name | meaning |
|---|---|---|
| 0x0001 | `HEAP_HASNULL` | null bitmap present |
| 0x0002 | `HEAP_HASVARWIDTH` | has variable-width columns |
| 0x0004 | `HEAP_HASEXTERNAL` | has TOASTed (out-of-line) columns |
| 0x0020 | `HEAP_COMBOCID` | `t_cid` is a combo CID |
| 0x0010 / 0x0040 / 0x0080 | `XMAX_KEYSHR_LOCK` / `XMAX_EXCL_LOCK` / `XMAX_LOCK_ONLY` | xmax is a row locker rather than a deleter |
| 0x0100 | `HEAP_XMIN_COMMITTED` | hint: xmin known committed |
| 0x0200 | `HEAP_XMIN_INVALID` | hint: xmin known aborted (both set together = FROZEN) |
| 0x0400 | `HEAP_XMAX_COMMITTED` | hint: xmax known committed |
| 0x0800 | `HEAP_XMAX_INVALID` | hint: xmax invalid or aborted |
| 0x1000 | `HEAP_XMAX_IS_MULTI` | xmax is a MultiXactId (multiple lockers) |
| 0x2000 | `HEAP_UPDATED` | this tuple is the UPDATEd version of a row |

`t_infomask2`: `HEAP_NATTS_MASK = 0x07FF` (attribute count), `HEAP_KEYS_UPDATED = 0x2000`, `HEAP_HOT_UPDATED = 0x4000`, `HEAP_ONLY_TUPLE = 0x8000` [S5].

The "hint bits" (`*_COMMITTED` / `*_INVALID`) are caches of the commit log (pg_xact) result. They are set lazily by whichever reader first resolves the status, so later readers skip the lookup [S5]. For learning purposes they are optional, because you can always consult your transaction status table.

### 3.2 Null bitmap

- Present only if `HEAP_HASNULL` is set. It is one bit per column (`natts` from `t_infomask2`), starting at byte 23 [S1 §65.6.1].
- **Trap: a 1 bit means NOT null, and a 0 bit means null** [S1 §65.6.1].
- User data starts at `t_hoff`, which PostgreSQL rounds up to MAXALIGN (8 on 64-bit). Each column is then aligned per its type's `typalign`. Varlena columns start with a 1- or 4-byte length header [S1 §65.6.1].

### 3.3 Limits

`MaxHeapTupleSize = BLCKSZ - MAXALIGN(24 + 4)` = 8160 bytes on 64-bit [S5]. Larger rows need TOAST.

## 4. FSM and VM forks (what they do)

- **FSM** [S1 §65.3, [freespace/README](https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/src/backend/storage/freespace/README)]: uses one byte per heap page, storing free space at a granularity of 1/256th of a page (free / (BLCKSZ/256), rounded down). Each FSM page holds a binary max-tree in an array, and FSM pages themselves form a tree. Its job is to answer "which page has ≥ N bytes free?" without scanning.
- **VM** [S1 §65.4]: uses two bits per heap page, *all-visible* and *all-frozen*. The map is conservative: a set bit is guaranteed true, but a clear bit may or may not be.
  - VACUUM sets the bits, and any modification to the page clears them.
  - All-visible lets VACUUM skip the page and lets index-only scans skip the heap.
  - All-frozen exists for anti-wraparound VACUUM.
- `PD_ALL_VISIBLE` in `pd_flags` mirrors the VM bit on the page itself [S2].

## 5. nbtree (B-tree index) pages

Every index page uses the same slotted format as §2. The special space holds:

### 5.1 `BTPageOpaqueData`: 16 bytes, at `pd_special` [S7]

```
off size field          meaning
 0   4   btpo_prev      left sibling block, or P_NONE (0) if leftmost
 4   4   btpo_next      right sibling block ("right-link"), or P_NONE if rightmost
 8   4   btpo_level     0 = leaf, counting up
12   2   btpo_flags     BTP_LEAF=1, BTP_ROOT=2, BTP_DELETED=4, BTP_META=8,
                        BTP_HALF_DEAD=16, BTP_SPLIT_END=32, BTP_HAS_GARBAGE=64,
                        BTP_INCOMPLETE_SPLIT=128, BTP_HAS_FULLXID=256
14   2   btpo_cycleid   VACUUM cycle ID of the latest split
```

### 5.2 Metapage (block 0) [S7]

`BTREE_METAPAGE = 0`. It holds `BTMetaPageData`: `btm_magic = 0x053162`, `btm_version`, `btm_root`, `btm_level`, `btm_fastroot`, `btm_fastlevel`, plus cleanup statistics. Its purpose is to tell searches where the root currently is, since the root moves when it splits.

### 5.3 Index tuple `IndexTupleData`: 8 bytes + key [S6]

```
off size field     meaning
 0   6   t_tid     heap TID (on leaf pages) | child block number (downlink, on internal pages)
 6   2   t_info    bit15 = has nulls, bit14 = has varwidth, bit13 = AM-reserved, bits 0-12 = tuple size
 8       [null bitmap if bit15] then key attributes, MAXALIGN'd
```

### 5.4 Page item order and the Lehman-Yao invariants [S7 README, nbtree.h]

```
leaf or internal page, NOT rightmost:     rightmost page on its level:
  offset 1 (P_HIKEY)   = high key           offset 1 = first data item
  offset 2 (P_FIRSTKEY)= first data item    (there is no high key)
  ...
```

- `P_FIRSTDATAKEY(opaque)` is 1 on the rightmost page and 2 otherwise [S7].
- **Right-link + high key** (Lehman & Yao): after descending to a page, if the search key > the page's high key, the page was split concurrently, so follow `btpo_next` right. This lets readers avoid lock coupling [S7 README].
- **Pivot tuples:** every tuple on an internal page, plus the high key on a leaf, is a "pivot". It is used only to navigate and does not point at a heap row. On an internal page the first data item's key is treated as **minus infinity** (keys truncated away), so only its downlink matters [S7 README].
- **Uniqueness via heap TID:** PostgreSQL treats the heap TID as a final tiebreaker key column, so every key is unique on its level and duplicates are stored in TID order. This is what the L&Y invariant `Ki < v <= Ki+1` requires [S7 README].
- Later additions you can skip:
  - Deduplication / posting lists, where one key holds many TIDs (v13).
  - Suffix truncation of pivots.
  - Page deletion (half-dead / deleted states).
  - "Fast root".
  - VACUUM cycle IDs.

## 6. How the pieces connect: index → TID → heap → VACUUM

```
B-tree leaf              heap file block 7
+------------------+     +--------------------------------------------+
| key=42, tid=(7,3)|---->| hdr | lp1 | lp2 | lp3 |...|  tuple(xmin..) |
+------------------+     +------------------^-------------------------+
                                            lp3: off/len -> tuple bytes
```

VACUUM (heap and index) works in phases. This is why `LP_DEAD` and `LP_UNUSED` are distinct states (see `itemid.h` [S3] and `pd_flags` [S2]; the README's section on VACUUM describes the index side [S7]):

1. Scan the heap. Dead tuples get their storage reclaimed and the line pointer becomes `LP_DEAD`, so the TID stays reserved.
2. Delete every index entry pointing at those TIDs.
3. Only now flip those line pointers to `LP_UNUSED` so they can be reused. Reusing a TID earlier would let a stale index entry point at an unrelated new row.

## Recommended subset for this project

The goal is to stay faithful to the *ideas* (slotted pages, stable TIDs, versioned tuples, L&Y B-tree, LSN on page) while dropping machinery that exists for 30 years of compatibility, concurrency at scale, or C portability.

### Mirror (keep the concept, near-same layout)

| Keep | Why |
|---|---|
| 8 KB pages, block number = file offset / 8192 | The unit of I/O, buffering, and WAL |
| Slotted page: header, line pointers down, tuples up, special space | The core idea. It is what makes TIDs stable |
| `pd_lsn` in every page header | **Required for WAL redo idempotency**: replay a record only if `record.lsn > page.lsn`. Do not drop this |
| `pd_lower`, `pd_upper`, `pd_special` | Free-space math and special area |
| `ItemId { off, len, flags }` with UNUSED / NORMAL / DEAD | DEAD vs UNUSED is the heap-index-VACUUM contract (§6) |
| TID = (u32 block, u16 offset), 1-based offset, 0 = invalid | What indexes store, and what `t_ctid` uses |
| Heap header: `xmin`, `xmax`, `cid`, `ctid`, `infomask`, `natts`, `hoff` | Minimum needed for MVCC snapshot checks and update chains |
| Null bitmap (1 = not null) | Cheap, and teaches the real format |
| B-tree: metapage at block 0, opaque struct {prev, next, level, flags}, high key at offset 1 on non-rightmost pages, right-link, minus-infinity first item on internal pages, heap TID as tiebreaker | The Lehman-Yao design. The TID tiebreaker removes most duplicate-key special cases |

### Drop or simplify

| Drop | Replace with / consequence |
|---|---|
| 32-bit XIDs, wraparound, freezing, `HEAP_XMIN_FROZEN`, `t_xvac` | **64-bit XIDs** (header grows by 8 B). No anti-wraparound VACUUM |
| Combo CIDs, separate cmin/cmax | A single `cid` field. Simply don't support "insert and delete the same row in one transaction while an older command still needs the row" with full fidelity, or keep cmin/cmax as two fields |
| MultiXact, row-lock bits (`XMAX_*_LOCK`, `LOCK_ONLY`) | No `SELECT ... FOR SHARE`. Treat a set xmax as "deleted/updated by" |
| HOT (`LP_REDIRECT`, `HEAP_HOT_UPDATED`, `HEAP_ONLY_TUPLE`, `pd_prune_xid`) | Every UPDATE inserts new index entries. Add HOT later as a stretch goal |
| Hint bits | Optional. Always consult the transaction status table |
| TOAST, `HEAP_HASEXTERNAL` | Reject rows that exceed about 8 KB |
| FSM fork | An in-memory `Vec<u16>` of free bytes per page, rebuilt by scanning at startup |
| VM fork, `PD_ALL_VISIBLE` | None. Consequences: VACUUM scans every page, and there are no index-only scans |
| Init fork, unlogged tables, tablespaces | Not needed |
| 1 GB segments | One file per relation |
| Checksums | Optional. A CRC32 in `pd_checksum` is easy to add later |
| `pd_pagesize_version` | A fixed magic/version byte if you want one |
| MAXALIGN and per-type alignment padding | Pack columns byte-tight and read them with `from_le_bytes`. Faithful in spirit, simpler in Rust |
| nbtree dedup, suffix truncation, page deletion, fast root, cycle IDs, `BTP_INCOMPLETE_SPLIT` | Plain splits. Empty pages simply stay empty |

### Rust implementation notes

- **Pick an explicit byte order (little-endian) and pack bits manually.** PostgreSQL writes structs in native endianness, and the order of fields within a C bitfield (`ItemIdData`) is compiler-defined. Don't try to `#[repr(C)]`-transmute the C layout. Encode `ItemId` as `u32` with `off | flags << 15 | len << 17` (or similar), and document the choice.
- Suggested minimal heap header (fixed, no padding):

```
off size field
 0   8   xmin        u64
 8   8   xmax        u64   (0 = invalid)
16   4   cid         u32
20   4   ctid.block  u32
24   2   ctid.off    u16
26   2   infomask    u16   (HASNULL, HASVARWIDTH, XMIN_COMMITTED/INVALID, XMAX_COMMITTED/INVALID, UPDATED)
28   2   natts       u16
30   1   hoff        u8
31       null bitmap (ceil(natts/8) bytes, if HASNULL), then column data
```

- Keep the page header at 24 bytes for familiarity: `lsn u64, checksum u16, flags u16, lower u16, upper u16, special u16, version u16, reserved u32`.
- Unit-test the invariants: `lower <= upper <= special <= 8192`, line-pointer count = `(lower - 24) / 4`, every NORMAL item lies within `[upper, special)`, and a TID's offset is ≥ 1.

## Open questions for the project

- 64-bit XIDs vs 32-bit with freezing. Recommend 64-bit: freezing is a VACUUM topic of its own, and it can come later.
- Whether the first milestone wants HOT. Recommend no, because it couples heap pruning with index maintenance.
