# Standing-duty finding: the app's persisted state guarantees the first-screen card cannot pass, and it explains the `ql_delivery` mystery

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-first-screen-card-cannot-pass-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, acceptance, blocking, card-defect, persisted-state |

## The defect

`acc-disp-first-screen-default-tab` is the card Aegis ranked **first** — the one he told the operator
takes ten seconds and is the first thing he sees every time he opens the product. Its verification:

> 1. quit the app completely
> 2. reopen, click nothing
> 3. look at which page the left rail has highlighted — **expected: Dispatch (dispatches)**
> 4. quit again, reopen, repeat
> 5. incidentally confirm the Mirror page still works

**On the machine the operator actually uses, reopening restores project `project_delivery/QL` and tab
`live-seats`. The expected value cannot appear.**

## How I measured it — from the app's own persisted state, not from the code

The store persists under key `seatloom-app-store` in the Tauri webview's LocalStorage. I read it
directly from the running app's data directory:

```
length: 2160
top-level: ['state', 'version']
activeProjectId: project_delivery/QL
navCollapsed: False
projectUIStates: 9 entries
  'p-1':                              activeTab='dispatches'
  '(root)':                           activeTab='live-seats'
  'seatloom':                         activeTab='live-seats'
  'project_delivery/QL':              activeTab='live-seats'
  'Assistant_Aegis':                  activeTab='live-seats'
  'sf-code/storage':                  activeTab='sessions'
  'ql_delivery':                      activeTab='live-seats'
  'project_delivery/networkOrchestrator': activeTab='live-seats'
  'project_delivery/stepaicloud':     activeTab='live-seats'
```

Two facts decide the card:

**1. `activeProjectId` is `project_delivery/QL`, not `seatloom`.** The restore effect at
`App.tsx:435-446` reads `projectUIStates[activeProjectId]` and, when a saved state exists, calls
`setActiveTab(savedState.activeTab)`. So the tab that comes back is the one saved **for the restored
project**.

**2. `project_delivery/QL`'s saved tab is `live-seats`.** Not `dispatches`.

So: quit → reopen → the app returns to `project_delivery/QL` on the **Mirror** page. The operator is told
to expect **Dispatch** and will see **Mirror**.

The only project whose saved tab is `dispatches` is `p-1` — which is **not** the restored project, and
not a project the operator is asked to be in.

## Why this is a card defect and not a product defect

The code is doing exactly what it is designed to do: restore where you were. `App.tsx:133` sets the
*initial* tab to `'dispatches'`, and that is what a **first** launch shows. The card's expected value is
correct for a first launch and wrong for a **re**launch, and the card's own steps 1 and 4 are relaunches.

This is the second card this round whose expected value is right about one thing and wrong about the
thing its own steps actually do. Last time it was 10 seats versus a project-scoped list of 7. This time
it is a first-launch default versus a per-project restore.

## The `ql_delivery` mystery, solved

Last round I reported that `acc-disp-bc20UzOznJQ8hCgbHw876`'s step 4 names a project `ql_delivery` that
does not exist, and called it a wrong id. **That was incomplete, and the app's own state explains it.**
`ql_delivery` is a key in `projectUIStates` — the operator was once in a project with that id, and it
has since been renamed or removed from `projects`. The card did not invent it; it read the app's stale
persisted state.

That makes the earlier finding worse rather than better: **the card's step 4 tells the operator to switch
to a project he can no longer select, because the id survives only in the client's storage.** And the
current `activeProjectId` is `project_delivery/QL` — the successor, presumably — so the app is on a
project the card does not mention at all.

## What should change

For `acc-disp-first-screen-default-tab`, the verification must distinguish the two cases it is testing:

- **first launch after install** (no saved state): expect Dispatch. That is what `useState<Tab>('dispatches')`
  gives.
- **relaunch** (saved state exists): expect **the project and tab you were last on**, which today is
  `project_delivery/QL` on Mirror. That is the restore working, not a defect.

A card that says "quit and reopen, expect Dispatch" is testing the wrong thing, and the operator will
rightly conclude the product forgot his setting.

For `acc-disp-bc20UzOznJQ8hCgbHw876`, step 4's project id should be `project_delivery/QL`, and the card
should say the seat list is project-scoped (7 under `seatloom`) rather than the table's 10.

**Whether to cancel-and-reopen either card is not my call** — the work both verify is real and delivered,
and decided rows are immutable. I am recording the measurements so the operator does not hit them at the
moment he is trying to accept correct work.

## What I did not do

I did not edit either card. I did not clear the persisted state — that would change what the operator
sees on his next launch, which is his call, not mine. I did not touch `App.tsx`.
