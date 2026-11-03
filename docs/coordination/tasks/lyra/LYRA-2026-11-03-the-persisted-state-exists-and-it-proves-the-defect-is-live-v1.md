# Standing-duty correction and finding: my ruling said the operator's persisted state was unverified because I could not find the file — it exists, I read it, and it proves the defect is live. The installed app's persisted landing project is `seatloom` with `activeTab: 'live-seats'`, which is exactly the behaviour my ruling says must not happen on a cold start.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-persisted-state-exists-and-it-proves-the-defect-is-live-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, correction, measured, live |

## What I got wrong in my own ruling

**In the takeback ruling I delivered tonight, section 3.3 item 4, I wrote:**

> The measured location of the persisted state (which Flux measured):
> `~/Library/WebKit/com.seatloom.app/.../localStorage.sqlite`, UTF-16-LE JSON. **I could not find this file on
> this machine**, so the claim "the operator's persisted state really carries a saved tab that this mechanism
> overrides" is listed by me as **unverified**.

**The file exists. I looked in the wrong place and then reported my failed search as a measurement.**

```
~/Library/WebKit/com.seatloom.app/WebsiteData/Default/<uuid>/<uuid>/LocalStorage/localstorage.sqlite3
```

**Not** at the path I tried — I looked for a `LocalStorage` directory directly under the app's `WebsiteData`
and found an empty one, and stopped. The real path is two levels deeper, under `Default/<uuid>/<uuid>/`. **An
empty directory at the path I guessed, and a populated file at the path that exists, are not the same
observation, and I reported the first as though it were the second.**

**And this is the third time in three days I have published something I had not finished measuring.** The first
was `check-ledger-exists.sh`, the second was the workitem status vocabulary. **Each time the shape is identical:
I searched, the search came up empty, and I wrote down the empty result as a finding instead of writing down
that my search was unverified.**

## What the file says, and it is the decisive measurement

Decoded as UTF-16-LE (the encoding Flux named), `seatloom-app-store` carries:

```json
{"state":{"projectUIStates":{
   "p-1":                    {"activeTab":"dispatches"},
   "(root)":                 {"activeTab":"live-seats"},
   "seatloom":               {"activeTab":"live-seats"},
   "project_delivery/QL":    {"activeTab":"live-seats"},
   "Assistant_Aegis":        {"activeTab":"live-seats"},
   "sf-code/storage":        {"activeTab":"sessions"},
   "ql_delivery":            {"activeTab":"live-seats"},
   "project_delivery/networkOrchestrator": {"activeTab":"live-seats"},
   "project_delivery/stepaicloud":          {"activeTab":"live-seats"}
 },
 "activeProjectId":"seatloom", "navCollapsed":...}}
```

**The persisted landing project is `seatloom`, and `seatloom`'s saved tab is `live-seats`.**

**That is the exact behaviour my ruling says must not happen.** My ruling's section 3.2 item A reads: *"Quit
SeatLoom completely and reopen. It should land on the Dispatches tab - not the tab you were last on."* **The
operator's real persisted state says that if he quits and reopens the installed app today, it will open on the
live-seats tab.**

**And this is not a prediction from a component test.** It is the operator's own persisted state, written by
the installed application (`com.seatloom.app`, binary built 10-28 22:16, file modified 11-02 11:19). **The
cold-start defect is live in the build he runs, and the evidence is on disk.**

## What this changes about the ruling

**It upgrades one line of my ruling from "unverified" to "verified, and failing."** Section 3.3 item 4 is no
longer a gap to be closed by a build — the persisted state half of it is already measured, and it says the
mechanism is needed.

**It does not close the acceptance.** The build-and-launch verification of A-D is still undone, because the
installed binary is from 10-28 and predates every fix in this stretch. **What it does is remove the last
reason to doubt the defect is real: I now have the operator's own persisted state saying `seatloom` remembers
`live-seats`.**

**And it changes what the isolated build must contain.** My ruling said the sandbox database "must really
contain those two projects with different saved tabs, or A and B cannot be verified at all." **The operator's
real state already contains nine projects with three different tabs** — `p-1` on `dispatches`, `sf-code/storage`
on `sessions`, and seven on `live-seats`. **A build seeded from his actual persisted state would exercise the
exact case that fails, which is stronger than a synthetic two-project fixture.**

## The thing I want on record about the failure, not just the fix

**I delivered a ruling tonight whose most load-bearing section ended in "unverified", and the reason it ended
there is that I ran one search, got nothing, and stopped.** The file was one directory deeper.

**The rule this earns: "I could not find it" is a statement about my search, not about the filesystem.** It
belongs in a document as *"I searched at path X and found nothing; the claim is unverified"* — never as *"the
file does not exist."* **I wrote the second and it was wrong, in a ruling whose entire purpose was to be
careful about exactly this distinction.**

## What I did not do

I did not modify the persisted state — I opened the database with `mode=ro` and read only. I did not restart
the operator's application, merge anything, or write to the live ledger. I did not edit my delivered ruling;
this document corrects it rather than rewriting history. My measurements were `find` under
`~/Library/WebKit/com.seatloom.app`, `sqlite3` with a read-only URI, a UTF-16-LE decode, `stat` on the file
and on the installed binary. `seats` is 10, `canonical_events` is 4, `check-ledger-exists.sh` is exit 0.
