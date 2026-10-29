# Standing-duty finding: Aegis's conclusion about where the operator's scrollback comes from is wrong, and the measurement that settles it is one command

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-scrollback-source-correction-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, correction, product-truth, mirror, measurement |

## What Aegis concluded, and why it matters

At 03:28 Aegis reported on the operator's observation that the app's scrollback showed content he could
not find in the pipe. His conclusion:

> What the app scrolls back for QL is the saih SESSION FILE — the agent's own conversation record, six
> days deep, 202 MB. What tmux scrolls back is the terminal screen — a 2000-line window covering minutes.
> These are not two views of one stream. They are two different artifacts, and **the app is showing him
> the richer one.**

He is right that a product decision is unwritten. **He is wrong about the source, and the error matters,
because the fix it implies is different.**

## The measurement that settles it

**1. The app never reads `~/.saih`.**

```
$ grep -rn "\.saih" src-tauri/src/ ui/src/ crates/seatloom-core/src/ --include=*.rs --include=*.ts --include=*.tsx
(no output)
```

**2. The app's mirror reads `capture-pane`, bounded by the pane's `history-limit`.** Not a file, not the
pipe:

```
src-tauri/src/commands/session_cmds.rs:146
  /// Base64-encoded snapshot of the tmux pane at attach time (capture-pane).
src-tauri/src/commands/session_cmds.rs:117-119
  /// How many lines of pane scrollback to backfill into the mirror at attach time…
  /// Bounded by the pane's tmux `history-limit`, so asking for more silently returns less.
```

**3. The phrase the operator quoted is in the tmux pane.**

```
$ tmux capture-pane -t QL-seatloom -p -S -2000 | grep -n "Session found"
1935: Session found (2026-10-23, under the QL project directory). Read it for context:

$ tmux capture-pane -t QL-seatloom -p -S -2000 | grep -c "gang"
265
```

So the operator was **reading the tmux mirror**, and the content he quoted was a seat's own output
sitting in that pane's scrollback. That is also why his search of the pipe transcript found nothing: the
sentence was never written to the pipe as a searchable log line — it is rendered pane text.

**4. The sauh file he was pointed at is a different artifact that happens to contain the same
conversation.** `2026-10-23T10-21-43-…jsonl` is 193 MB / 7237 lines, mtime 11:26 today. It exists, it is
the same conversation, and **nothing in the product reads it.** Aegis found the file because he searched
the filesystem for the phrase — which is the right instinct and the wrong inference. Finding a file that
contains the text does not establish that the surface read the file.

## Why the error matters, concretely

Aegis's conclusion implies the app is showing the richer artifact and the pipe/tmux are the poorer ones,
so the work is "say which source the UI uses". **The measured situation is the opposite:** the app shows
the *poorest* of the three — a 2000-line terminal window — while a 193 MB / 7237-line conversation record
sits on disk unread, and a 37 MB pipe transcript sits beside it.

And the 2000-line cap is not theoretical. The phrase the operator quoted is at line **1935 of 2015**,
against a `history-limit` of **2000** with `history_size` 1949. **It is roughly 65 lines from being gone
permanently.** He found it this time; the next person will not.

## What is actually undecided, stated as the product question it is

**Should the mirror show the terminal window, the conversation record, or both — and should it say which?**

That is the decision Aegis correctly identified as unwritten. My measurement narrows it:

- showing the **terminal window only** (today) means the operator loses context every 2000 lines, and
  the content he needs is already on disk;
- showing the **conversation record** means he can search and scroll six days, but it is not what the
  pane is — a seat's terminal and its conversation are different things, and conflating them would make
  the mirror lie about what it is;
- showing **both, labelled**, is the only option that does not lose anything and does not pretend.

**I lean both-and-labelled, and I am not ruling it yet**, because the operator is engaged right now and
this is his surface. What I am ruling is that the question must be answered rather than left as an
accident of `capture-pane`'s default.

## What I did not do

I did not touch any pipe, pane, or history-limit — Aegis is right that the measurement does not need the
pane disturbed, and raising a live seat's `history-limit` mid-session is an operator action. I did not
edit `session_cmds.rs`. I did not dispatch a seat.
