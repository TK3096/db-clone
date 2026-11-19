# psql wire protocol subset

**Question (issue #5):** What is the smallest part of the PostgreSQL v3 frontend/backend protocol a server must implement so that stock `psql` can connect and run simple queries? Which queries does psql send on its own, and which can be left unsupported?

**Short answer.** You need the **simple query protocol** and nothing else. Reply `N` to SSLRequest and GSSENCRequest. Accept the StartupMessage. Send `AuthenticationOk`, a few `ParameterStatus` messages (at least `server_version`), `BackendKeyData` and `ReadyForQuery`. Then answer each `Query` with `RowDescription`/`DataRow`*/`CommandComplete`, or `EmptyQueryResponse`, or `ErrorResponse`, always followed by `ReadyForQuery`. Close the connection on `Terminate`. psql **sends no queries on connect**. The extended protocol is only used by `\gdesc` and `\bind`. Meta-commands such as `\d` and `\dt` are ordinary SQL against `pg_catalog`, and if you answer them with an `ErrorResponse`, psql prints the error and carries on.

All of this was **checked empirically.** An ~85-line Python stub that implements only the subset below served real `psql (PostgreSQL) 17.5` (Homebrew): `select`, `insert`, `begin`/`commit`, an empty statement, `\dt`, `\d t`, `\gdesc` and `\conninfo` all behaved as described. See [Empirical check](#empirical-check).

Sources. Line numbers refer to these pinned commits:
- PG17 tree: `REL_17_STABLE` @ `901eb483e0fff340ae3fe496ad2a2d1856f38693`. Links are below as `pg17:path:line`, which means `https://github.com/postgres/postgres/blob/901eb483e0fff340ae3fe496ad2a2d1856f38693/<path>#L<line>`.
- PG18 tree: `REL_18_STABLE` @ `75829f043992ee863111d245197f8442774501b6`, written as `pg18:path:line`.
- Docs: [Protocol message flow][flow], [Message formats][fmt], [Error and notice fields][errf] (all v17).

[flow]: https://www.postgresql.org/docs/17/protocol-flow.html
[fmt]: https://www.postgresql.org/docs/17/protocol-message-formats.html
[errf]: https://www.postgresql.org/docs/17/protocol-error-fields.html

---

## 1. Framing basics (the most common first bug)

- Every regular message has the form `Byte1 type` + `Int32 length` + body. **The length counts itself (4 bytes) and the body, but not the type byte.** All integers are big-endian. Strings are NUL-terminated ("String" in the docs). [fmt]
- **Startup-phase packets (SSLRequest, GSSENCRequest, CancelRequest, StartupMessage) have no type byte.** They are just `Int32 length` + `Int32 code` + body. The server has to read the length, then the code, and branch on the code. [fmt]
- The server's reply to SSLRequest and GSSENCRequest is **a single raw byte** (`N` or `S`) with no length. [fmt], [flow]

## 2. Connection: what psql sends, in order

### 2.1 Transport

- Plain `psql`, with no `-h`, connects to a **Unix-domain socket** in the default socket directory (`DEFAULT_PGSOCKET_DIR`, e.g. `/tmp/.s.PGSQL.5432`), not to TCP (pg17:src/interfaces/libpq/fe-connect.c:1213). If the server only listens on TCP, run the demo as `psql -h 127.0.0.1 -p <port>`. Alternatively, listen on `/tmp/.s.PGSQL.<port>` as well.
- Over a Unix socket, libpq **never** sends SSLRequest or GSSENCRequest: "Don't request SSL or GSSAPI over Unix sockets" (pg17:src/interfaces/libpq/fe-connect.c:4294-4313).

### 2.2 Encryption probes over TCP (decline them)

- The default `sslmode` is `prefer` (pg17:src/interfaces/libpq/fe-connect.c:127), so over TCP the first packet is an **SSLRequest**: `Int32(8) Int32(80877103)`. [fmt]
- A **GSSENCRequest** (`Int32(8) Int32(80877104)`) is sent *before* the SSLRequest only if libpq was built with GSSAPI *and* a Kerberos credential cache exists. Otherwise GSS is dropped silently (pg17:src/interfaces/libpq/fe-connect.c:4417-4443). Handle it anyway; it costs one line.
- **Reply with the single byte `N`** to either request, then read the next untyped packet on the same socket. libpq takes `N` as "continue in plaintext" when `sslmode=prefer` (pg17:src/interfaces/libpq/fe-connect.c:3501-3511; [flow]).
- A **CancelRequest** (`Int32(16) Int32(80877102) Int32 pid Int32 key`) arrives on a *new* connection when the user presses Ctrl-C during a query. It is fine to read it and close the socket. [fmt]

### 2.3 StartupMessage

Layout: `Int32 len`, `Int32 196608` (protocol 3.0 = `3<<16 | 0`), then `name\0value\0` pairs, then a final `\0`. [fmt]

The parameters psql sends are built in pg17:src/bin/psql/startup.c:250-270 and pg17:src/interfaces/libpq/fe-protocol3.c:2311-2350:

| key | when |
|---|---|
| `user` | always (defaults to the OS user) |
| `database` | always (defaults to user name) |
| `application_name` | `psql` (via `fallback_application_name`) |
| `client_encoding` | only when stdin/stdout is a TTY and `PGCLIENTENCODING` is unset: psql passes `auto`, which libpq resolves from the locale (e.g. `UTF8`) (startup.c:267, fe-connect.c:1839-1846) |
| `options`, `replication`, PG* env options | only if set by the user |

Observed with psql 17.5 over a pipe: `user=demo database=demo application_name=psql` (no `client_encoding`, because the session was not a TTY).

**Protocol version:** PG18's libpq still defaults to protocol 3.0 (`max_protocol_version` defaults to 3.0 unless `min_protocol_version` is raised; pg18:src/interfaces/libpq/fe-connect.c:2128-2140). The server does not need `NegotiateProtocolVersion` for psql 17 or 18.

### 2.4 Server's startup response

Send these in one write:

1. **AuthenticationOk**: `'R' Int32(8) Int32(0)`. That is "trust" auth. No password exchange happens. [fmt]
2. **ParameterStatus** ×N: `'S' Int32 len String name String value`. [fmt]
3. **BackendKeyData**: `'K' Int32(12) Int32 pid Int32 secret`. In protocol 3.0 the secret **must be exactly 4 bytes**, and PG18 libpq raises a fatal error otherwise (pg18:src/interfaces/libpq/fe-protocol3.c:1588-1593). psql only uses it for Ctrl-C cancel, so any constants will do.
4. **ReadyForQuery**: `'Z' Int32(5) Byte1 status`, where status is `I` (idle), `T` (in transaction block) or `E` (failed transaction block). [fmt]

To refuse a connection instead, send an `ErrorResponse` (severity `FATAL`) and close.

### 2.5 Which ParameterStatus values matter

The server normally reports this fixed set: `application_name, client_encoding, DateStyle, default_transaction_read_only, in_hot_standby, integer_datetimes, IntervalStyle, is_superuser, scram_iterations, server_encoding, server_version, session_authorization, standard_conforming_strings, TimeZone` [flow]. The docs also say clients "should simply ignore ParameterStatus for parameters that [they do] not understand". Of these, the ones psql and libpq actually read are:

| parameter | consumer | effect if missing | recommended |
|---|---|---|---|
| `server_version` | libpq parses it into `PQserverVersion()` (pg17:src/interfaces/libpq/fe-exec.c:1149-1183). psql stores it as `pset.sversion` (pg17:src/bin/psql/command.c:4060) | version = 0, so psql prints `WARNING: psql major version 17, server major version 0. Some psql features might not work.` because 0 < 90200 (command.c:3925-3961). `describe.c` also branches on `pset.sversion` to pick catalog SQL | **Required in practice.** Send the same major as the user's psql, e.g. `17.0`, which also suppresses the "server X" banner |
| `client_encoding` | libpq `conn->client_encoding` (fe-exec.c:1136-1143), psql `pset.encoding` (command.c:4058) | defaults to SQL_ASCII, so non-ASCII output may be mangled | `UTF8` |
| `server_encoding` | informational | none | `UTF8` |
| `standard_conforming_strings` | libpq escaping, psql `standard_strings()` (pg17:src/bin/psql/common.c:2241) | treated as off | `on` |
| `is_superuser` | psql prompt `%#` shows `#` vs `>` (pg17:src/bin/psql/prompt.c:258, common.c:2221) | prompt shows `>` | `on` (cosmetic) |
| `session_authorization` | `session_username()` (common.c:2261); falls back to the startup `user` | none | user name |
| `integer_datetimes`, `DateStyle`, `TimeZone`, `IntervalStyle` | only matter for binary-format or date output | none | `on`, `ISO, MDY`, `UTC`, `postgres` |
| `default_transaction_read_only`, `in_hot_standby` | used only when `target_session_attrs` ≠ `any` (fe-connect.c:3993-4077) | none with default settings | omit |

**Minimum viable set:** `server_version`, `server_encoding`, `client_encoding`. The rest are cosmetic or ignored.

**No queries on connect.** With the default `target_session_attrs=any`, libpq issues no SQL after ReadyForQuery. psql's own connect path (`SyncVariables`, `connection_warnings`, pg17:src/bin/psql/command.c:3925-4085) only *reads* ParameterStatus values; there is no `PQexec`/`PSQLexec` in `startup.c`. The only thing that can send SQL at startup is the user's `~/.psqlrc`, which `-X` skips. Confirmed empirically: the first message after startup was the user's own `Q`.

## 3. Simple query flow

Client sends **Query**: `'Q' Int32 len String sql`. [fmt]

- **psql keeps the trailing semicolon.** It sends exactly `select 1;\0`. A bare `;` is sent as `;\0`. Your parser must accept an optional trailing `;` and the empty statement. (Observed; psql's lexer emits the `;` into the query buffer before `mainloop.c` calls `SendQuery`, pg17:src/bin/psql/mainloop.c:420-440.)
- psql splits multi-statement input lines on `;` itself and sends **one statement per Query**. The server still has to handle multiple statements in a single `Q` for `psql -c 'a; b'`. [flow] describes the multi-statement rules (one implicit transaction, results streamed per statement, a single ReadyForQuery at the end). For a learning project it is reasonable to reject multi-statement `Q`s with an ErrorResponse.

Server responds (per statement, then **exactly one** ReadyForQuery at the end) [flow]:

| case | messages |
|---|---|
| row-returning (`SELECT`, `EXPLAIN`) | `T` RowDescription, `D` DataRow × n, `C` CommandComplete, `Z` |
| no rows (`INSERT/UPDATE/DELETE/CREATE/BEGIN/COMMIT`) | `C` CommandComplete, `Z` |
| empty string / only `;` | `I` EmptyQueryResponse (`'I' Int32(4)`), `Z` |
| error | `E` ErrorResponse, `Z` (abort the statement; skip the rest of a multi-statement Q) |

### 3.1 Byte layouts [fmt]

**RowDescription** `'T' Int32 len Int16 nfields`, then per field:
`String name, Int32 table_oid(0 ok), Int16 attnum(0 ok), Int32 type_oid, Int16 typlen, Int32 typmod(-1), Int16 format(0 = text)`.

**DataRow** `'D' Int32 len Int16 ncols`, then per column `Int32 len` + bytes. Use **len = -1 for NULL**, with no bytes following. Values are in text format. libpq rejects a `D` without a preceding `T` (pg17:src/interfaces/libpq/fe-protocol3.c:405) and checks the column count (fe-protocol3.c:785).

**CommandComplete** `'C' Int32 len String tag`. Tags come from pg17:src/include/tcop/cmdtaglist.h and the format from `BuildQueryCompletionString` (pg17:src/backend/tcop/cmdtag.c:121-163):

| statement | tag |
|---|---|
| SELECT | `SELECT <rows>` |
| INSERT | `INSERT 0 <rows>` (the `0` is a legacy OID slot, always 0) |
| UPDATE / DELETE | `UPDATE <rows>` / `DELETE <rows>` |
| CREATE TABLE | `CREATE TABLE` |
| BEGIN / START TRANSACTION / COMMIT / ROLLBACK | `BEGIN` / `START TRANSACTION` / `COMMIT` / `ROLLBACK` |
| EXPLAIN | `EXPLAIN` |

psql prints the tag for non-row results (e.g. `INSERT 0 1`). For row results it prints `(n rows)` from the number of DataRows and ignores the tag.

**EXPLAIN** is a row-returning statement. Real Postgres returns **one `text` column named `QUERY PLAN`** (pg17:src/backend/commands/explain.c:416) and one DataRow per plan line, then `C EXPLAIN`.

**Type OIDs** (pg17:src/include/catalog/pg_type.dat): `bool 16, name 19, int8 20, int2 21, int4 23, text 25, oid 26, float8 701, bpchar 1042, varchar 1043, numeric 1700`. typlen: int4 = 4, int8 = 8, bool = 1, text/varchar = -1. **psql right-aligns int2/int4/int8/float4/float8/numeric/oid columns and left-aligns everything else**, based purely on the OID (pg17:src/fe_utils/print.c:3615-3639). A wrong OID is visible in the demo output, but it does no other harm.

**ErrorResponse** `'E' Int32 len`, then repeated `Byte1 field-code String value`, terminated by a `\0` byte. [errf] says `S` (severity, localized), `C` (SQLSTATE) and `M` (message) are "always present". Also send `V` (non-localized severity). Optional extras are `D` detail, `H` hint and `P` position (1-based char index; psql then draws a `^` caret under the query). libpq formats the message as `"<S>:  <M>"` (pg17:src/interfaces/libpq/fe-protocol3.c:1045-1047). Useful SQLSTATEs: `42601` syntax_error, `42P01` undefined_table, `42703` undefined_column, `0A000` feature_not_supported, `25P02` in_failed_sql_transaction.

**NoticeResponse** `'N'` has the same layout as ErrorResponse. It is optional (for example, for `WARNING: there is no transaction in progress`).

**Terminate** `'X' Int32(4)`. The client sends it before closing (it was observed at the end of every session). Close the socket.

### 3.2 ReadyForQuery status and transactions

The `Z` status byte is how libpq knows the transaction state (`PQtransactionStatus`, pg17:src/interfaces/libpq/fe-protocol3.c:1626-1649). psql uses it for:
- the prompt: `%x` shows `*` when in a transaction and `!` when it has failed, so the default prompt `demo=*#` / `demo=!#` (pg17:src/bin/psql/prompt.c:233-240);
- `\set AUTOCOMMIT off`: psql sends its own `BEGIN` whenever the status is idle (pg17:src/bin/psql/common.c:1147);
- `ON_ERROR_ROLLBACK`: psql issues `SAVEPOINT pg_psql_temporary_savepoint` (common.c:1165). That is off by default, so you can leave it unsupported.

So after `BEGIN` send `Z T`. After an error inside the block send `Z E`. Until `COMMIT`/`ROLLBACK` (where real Postgres answers `COMMIT` on a failed block with tag `ROLLBACK`), send `Z I`.

## 4. The extended protocol is not needed

psql runs every user statement and every meta-command query through `PQsendQuery`/`PQexec`, which is the simple `Q` protocol (pg17:src/bin/psql/common.c:653, 1479-1481). The extended protocol (`P`arse/`B`ind/`D`escribe/`E`xecute/`S`ync) is used only by:
- `\gdesc`, which calls `PQprepare` (pg17:src/bin/psql/common.c:1349). Observed on the wire as `P` followed by `S`;
- `\bind ... \g`, which calls `PQsendQueryParams` (common.c:1479);
- PG18's new pipeline meta-commands (`\startpipeline`, etc.).

`FETCH_COUNT` does **not** need cursors in psql 17: it uses `PQsetChunkedRowsMode`, a client-side mode over the simple protocol (common.c:1513-1517).

**If you get an unsupported message type**, do not drop the connection. Dropping it makes psql print `server closed the connection unexpectedly` and attempt a reset. Instead, send an `ErrorResponse`, discard messages until `S` (Sync), then send `Z`. That is the documented extended-protocol error recovery [flow]. With this, `\gdesc` in the stub just printed `ERROR: extended query protocol not supported` and the next query worked.

## 5. Meta-commands and catalog queries

Backslash commands like `\dt` and `\d` are **plain SQL `SELECT`s against `pg_catalog`** sent through `PSQLexec`, which calls `PQexec` (pg17:src/bin/psql/common.c:622-662). Run `psql -E` to see them.

- `\dt` (`listTables`, pg17:src/bin/psql/describe.c:3913-4080) sends one query. It touches `pg_catalog.pg_class` (relname, relkind, relowner, relnamespace, relam), `pg_namespace`, `pg_am` (sversion ≥ 12), the functions `pg_get_userbyid()` and `pg_table_is_visible()`, and the regex operator `OPERATOR(pg_catalog.~)` with `COLLATE pg_catalog.default` when a pattern is given (pg18:src/fe_utils/string_utils.c:1231-1262). `\dt+` additionally calls `pg_size_pretty`, `pg_table_size` and `obj_description`.
- `\d name` (`describeTableDetails` → `describeOneTableDetails`, describe.c:1449 and 1532-3553) first resolves the OID via `pg_class`/`pg_namespace`. It then issues **up to ~23 more queries** touching `pg_attribute`, `pg_attrdef`, `pg_type`, `pg_collation`, `pg_index`, `pg_constraint`, `pg_inherits`, `pg_trigger`, `pg_policy`, `pg_statistic_ext`, `pg_publication*`, `pg_rewrite`, `pg_depend`, `pg_am`, `pg_sequence` and others, plus functions like `format_type`, `pg_get_expr`, `pg_get_indexdef`, `pg_get_constraintdef`, `pg_get_viewdef`, and casts such as `::pg_catalog.regclass`. Supporting this is a large project on its own.
- **Tab completion** (readline, interactive TTY only) runs catalog queries via `exec_query`. **Errors there are swallowed silently**: "Printing an error while the user is typing would be quite annoying" (pg17:src/bin/psql/tab-complete.c:6211-6236).

**How psql behaves when you answer them with an ErrorResponse:** `PSQLexec` → `AcceptResult` prints `ERROR:  <message>`, the meta-command returns false, and psql goes back to the prompt with the connection intact. Observed: `\dt` printed `ERROR:  not supported: SELECT n.nspname as "Schema", ...` and the next `select 1;` worked normally. With `ON_ERROR_STOP` a script would stop there, as it would for any error.

**Recommendation.** Leave all catalog-backed meta-commands unsupported (return SQLSTATE `0A000` or `42P01`). Everything in the end demo (CREATE/INSERT/SELECT/UPDATE/DELETE/BEGIN/COMMIT/EXPLAIN) works without them. Meta-commands that need no server round-trip (`\q`, `\?`, `\timing`, `\x`, `\set`, `\echo`, `\conninfo`) work for free. If `\dt` is wanted later as a stretch goal, the cheapest route is to recognise psql's exact query text (or anything `FROM pg_catalog.pg_class`) and answer it from the engine's own catalog with the 4 expected columns `Schema, Name, Type, Owner`. Don't implement real `pg_catalog`.

**Transaction caveat.** If the server follows Postgres semantics, where any error inside `BEGIN` moves the block to `E`, then an unsupported `\d` (or an invisible tab-completion query) typed inside a transaction will abort the user's transaction. Two ways to avoid this:
1. For the demo, run `psql -X -n` (`-X` skips `.psqlrc`, `-n` disables readline/tab completion), and avoid meta-commands inside `BEGIN`.
2. Or, as a deliberate design choice, have "unsupported catalog query" errors leave the transaction state alone. That is a deviation from Postgres, so document it if you do it.

## 6. Recommended minimal subset (checklist)

Server must **read**:
- untyped startup packets: SSLRequest 80877103 and GSSENCRequest 80877104 (reply `N`), CancelRequest 80877102 (close), StartupMessage 196608;
- `Q` Query, `X` Terminate;
- any other type byte: reply `E`, skip until `S`, then reply `Z`.

Server must **write**: `R` AuthenticationOk(0), `S` ParameterStatus (`server_version`, `server_encoding`, `client_encoding` at minimum), `K` BackendKeyData (4-byte key), `Z` ReadyForQuery(I/T/E), `T` RowDescription, `D` DataRow (text, -1 = NULL), `C` CommandComplete (tags above), `I` EmptyQueryResponse, `E` ErrorResponse (S/V/C/M).

Optional: `N` NoticeResponse, a Unix-socket listener at `/tmp/.s.PGSQL.<port>` so plain `psql -p <port>` works, and `P` (error position) in errors.

Out of scope: SSL/GSS, SCRAM/MD5 auth, extended protocol, COPY (`G`/`H`/`d`/`c`), NotificationResponse, function call (`F`), and NegotiateProtocolVersion.

## Empirical check

The stub was `python3 stub.py 54329`. It replies `N` to SSLRequest and sends `R/S×11/K/Z`. It answers `select 1` with `T(int4)/D/C SELECT 1`, `insert` with `C INSERT 0 1`, `begin`/`commit` with Z `T`/`I`, `;` with `I`, and everything else with `E 0A000`. Client: `psql (PostgreSQL) 17.5`, `psql -h 127.0.0.1 -p 54329 -U demo -d demo -X`.

The wire log showed:
```
STARTUP-PACKET code=80877103 len=8          <- SSLRequest (TCP, sslmode=prefer); replied 'N'
STARTUP-PACKET code=196608 len=55           <- user=demo database=demo application_name=psql
MSG 'Q' body='select 1;\0'                  <- first message after connect: the user's query
MSG 'Q' body='insert into t values (1);\0'
MSG 'Q' body='SELECT n.nspname as "Schema", ... FROM pg_catalog.pg_class c ...'   <- \dt
MSG 'Q' body='begin;\0'
MSG 'Q' body='SELECT c.oid, n.nspname, c.relname FROM pg_catalog.pg_class c ... OPERATOR(pg_catalog.~) ''^(t)$'' COLLATE pg_catalog.default ...'  <- \d t
MSG 'Q' body='commit;\0'
MSG 'P' body='\0select 1 \0\0\0' ; MSG 'S'   <- \gdesc (extended protocol)
MSG 'Q' body=';\0'                           <- empty statement
MSG 'X'                                      <- Terminate
```
psql output: the `select 1` table with `(1 row)`, `INSERT 0 1`, `ERROR:  not supported: ...` for `\dt` and `\d t`, `BEGIN`, `COMMIT`, `ERROR:  extended query protocol not supported` for `\gdesc`, then a normal result for the next `select 1;`. No version WARNING appeared because `server_version=17.0`. `\conninfo` worked with no round-trip.
