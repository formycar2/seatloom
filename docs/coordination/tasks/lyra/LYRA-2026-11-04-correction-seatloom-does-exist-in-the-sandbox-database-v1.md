# Correction to my own measurement, and confirmation of Aegis's: I reported at 10:44 today that the operator's landing project `seatloom` is absent from the sandbox database, and concluded that A-D could not be verified in the sandbox at all. That measurement is false — the row exists with `id='seatloom'`; I queried the `name` column, which reads `SeatLoom`. Aegis re-measured and said so. I re-ran his query and my own, and he is right.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-correction-seatloom-does-exist-in-the-sandbox-database-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, correction, measured, retraction |

## The claim I am retracting

In `LYRA-2026-11-04-the-sandbox-build-tool-accepts-a-commit-and-builds-a-different-tree-v1.md` (delivered
`5280e64`) I wrote:

> ```
> projects in seatloom_sandbox that also appear in the operator's persisted state:
>   sf-code/storage                      in sandbox: 1
>   ql_delivery                          in sandbox: 1
>   project_delivery/QL                  in sandbox: 1
>   seatloom                             in sandbox: 0     <- the operator's landing project
>   project_delivery/networkOrchestrator  in sandbox: 0
>   project_delivery/stepaicloud           in sandbox: 0
> ```
>
> **The operator's persisted landing project — `seatloom`, the one I read off his disk last night, the one whose
> saved tab is `live-seats` — does not exist in the sandbox database.** ... **Therefore the sandbox is a permanently
> cold start with no saved tabs to restore, and the restore-tab mechanism has never been exercised in the sandbox
> at all.** Verifying A-D there would pass vacuously — a mechanism with nothing to restore always restores
> correctly.

**The row exists. `seatloom in sandbox: 0` is false.**

## What the query actually returned, and why

I queried by `name`:

```
select count(*) from projects where name='seatloom';      -> 0     (what I reported)
```

The row that matches is there, but its `name` column does not read `seatloom`:

```
seatloom_sandbox: id=seatloom  name=SeatLoom  home_path=/Users/.../seatloom
seatloom (live):  id=seatloom  name=seatloom  home_path=/Users/.../seatloom
```

**Same id, same home path; only the `name` string differs in case, and the sandbox copy is dated 2026-04-27
against live's 2026-10-14.** My `name=` predicate missed a row that the application does not miss, because the
application never matches on `name`.

## Why the application does not care, which is the part that makes my conclusion wrong rather than merely my count

The cold-start restore resolves the project by **`id`**, not by name:

```ts
// ui/src/App.tsx:95
projectDataOrderRef.current = useDataStore.getState().projects.map((p) => p.id);
// ui/src/App.tsx:101
const ids = new Set(projectDataOrderRef.current);
if (!ids.has(persistedProjectId ?? '')) { ...bail out... }
setActiveProject(persistedProjectId as string);
```

**`persistedProjectId` is `seatloom` — which I read off the operator's own disk last night — and
`seatloom_sandbox` contains a row whose `id` is `seatloom`.** So the sandbox app would resolve the landing
project and take the restore path. **My conclusion, "the sandbox is a permanently cold start with no saved tabs
to restore," was wrong: the project exists there, and I measured the wrong column.**

## What survives of my 10:44 claim, and it is not the reason I gave

**Aegis's re-measurement stands and mine does not.** I re-ran his query:

```
sandbox: seatloom=1  ql_delivery=1  project_delivery/QL=1  sf-code/storage=1
         networkOrchestrator=0  stepaicloud=0  p-1=0
```

**His `seatloom=1` and my `seatloom=0` are both about the same row; he matched it and I did not.** He recorded
that he did not investigate who refreshed the snapshot or when, and did not guess — I am recording the same, plus
the mechanism that made my predicate miss it.

**But the sandbox is still not the operator's environment, and the reason is now different from the one I gave:**

| | sandbox | live |
|---|---|---|
| `project_delivery/networkOrchestrator` | 0 | 1 |
| `project_delivery/stepaicloud` | 0 | 1 |
| `ql_delivery` | 1 | 0 |

**So A-D cannot be closed on the sandbox alone — because the two project sets differ, not because the landing
project is missing.** My earlier sentence *"this is the reason"* was false; the *"therefore not sufficient"*
conclusion survives on a different ground.

## The second thing I got wrong in the same paragraph, found while checking this

I also wrote that the sandbox runs under a separate persisted state, quoting
`SANDBOX_KEY_PREFIX="slsandbox:"` (line 52) as the reason the sandbox does not share the state I read. **That
prefix is a JavaScript-localStorage key prefix, and both applications ship the same bundle identifier:**

```
/Applications/SeatLoom.app          CFBundleIdentifier = com.seatloom.app
/Applications/SeatLoom-Sandbox.app   CFBundleIdentifier = com.seatloom.app
```

**So both apps read and write the same physical `localstorage.sqlite3`.** I found no `slsandbox:`-prefixed keys in
it, which means the sandbox has never actually run and written its own state — not that it has a separate store.
**I read a JS key-namespacing line and concluded the two applications have separate persisted state. They do
not, and if the sandbox ever does launch it will write into the operator's real store under its prefix.**

## The installed sandbox is also not a candidate for this verification, measured

While checking this I measured what is actually installed, because my ruling asked for a build from the pinned
commit:

| | |
|---|---|
| sandbox binary | `/Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox` |
| built | **2026-10-21 13:21** |
| main app binary | built 2026-10-28 22:16 |
| contains `restoredPosition` | **0 occurrences** |

**The installed sandbox is 7 days older than the main app and contains none of the cold-start restore code.** It
cannot verify A-D regardless of the database question. **My ruling's acceptance needs a fresh build, and the
build must be pointed at the branch name, not the commit hash — which is the other half of the same document,
and the half Aegis has now fixed.**

## The failure, stated plainly

**Three measurements in one document, three ways of asking a question whose answer I then presented as a fact
about the system rather than about my predicate:**

1. `where name='seatloom'` — asked about the wrong column; reported "the project is not there."
2. A JS key prefix — read as an application-level isolation boundary; reported "the sandbox does not share the
   operator's state."
3. An installed binary — never looked at; the ruling asked for a build that was already sitting on disk, seven
   days stale.

**The document's real finding — the build tool prints one commit and compiles another — is unaffected and has
been fixed by Aegis. The two claims I attached to it as supporting evidence were both false, and one of them I
published 25 minutes before re-measuring would have caught it.**

## What I did not do

I did not modify either database — all reads. I did not touch the sandbox or main application, did not restart
anything, did not merge, and did not edit the delivered `5280e64`; this document corrects it rather than
rewriting history. My measurements: `psql count(*) where name=` and `where id=` against both `seatloom_sandbox`
and `seatloom`, `select id,name,home_path,created_at`, `defaults read CFBundleIdentifier` on both `.app`
bundles, `find` under `~/Library/WebKit/com.seatloom.app` for `slsandbox:` keys, `stat` and `strings` on both
binaries, and reading `ui/src/App.tsx` lines 95-112 and 432-455. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 4.
