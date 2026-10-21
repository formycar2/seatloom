# Verdict — document-render sanitization: typography unchanged, execution capability removed

- **Verification dispatch**: `disp-90h4J3F_iAHmTV-myGARs` (issued by seat-aegis-001), acceptance
  = Aegis's packet `AEGIS-2026-10-21-sanitize-document-render-v1.md`.
- **Implementation dispatch**: **`disp-FzldOwJuxCfxc2TGq9dMw`** (`verify_dispatch_id` in the payload).
- **Branch under test**: `packet/mira/knowledge-panel` @ `4ed8c76` ("run marked's output through an
  allow-list sanitiser before innerHTML") — Mira's worktree, clean.
- **Verdict event**: `ev-ucN0gvx2vIB_MmDMiZEgz` — the pre-declared id, emitted exactly as required,
  with a `ref_type='dispatch'` ref naming **`disp-FzldOwJuxCfxc2TGq9dMw`** (the implementation
  dispatch), not just this verification dispatch.
- **VERDICT: PASS** — with one finding that is a real gap in the implementation rather than a
  defect in the output: **the call site is untested**, so the sanitizer can be bypassed at the only
  place it matters and every test still passes.

## The three payload classes — no execution capability, measured by me

Driving the **real** `marked` + the **real** `sanitizeHtml` (jsdom-backed, so the DOM path is what
does the work — not a regex):

| payload | marked output | after sanitize |
|---|---|---|
| `<script>alert("executed")</script>` | `<script>…</script>` verbatim | `<script` gone, `alert(` gone, surrounding prose kept |
| `<img src=x onerror="alert(1)">` | `<img src=x onerror=alert(1)>` verbatim | no `on*` attribute of any spelling, no `<img` |
| `[click me](javascript:alert(2))` | `<a href="javascript:alert(2)">` verbatim | no `javascript:`, no `alert(` |
| obfuscated `JAVASCRIPT:` / leading whitespace / `java&#115;cript:` | anchors with those hrefs | all inert (entity-decoded in the flat check) |
| `data:text/html;base64,…` | anchor | dropped |
| `[x](vbscript:msgbox(1))` | anchor | dropped |
| `<svg><script>…</script></svg>`, `<math>…`, `<form><input>` | markup | dropped |

And `onclick` goes too (`<div onclick=… id=data-foo>` → `<div>`), which is the part a string-matching
approach would miss — the DOM probe confirms attributes are removed, not textually hidden.

## A normal document: typography unchanged — the operator's sentence, tested both ways

I checked the allow-list against the CSS the view is actually styled with: every selector
`.sl-doc-body …` (h1–h4, p, code, pre, blockquote, table/td/th, ul/ol/li, a, hr) is on the allow-list.
The test that matters most is the one that compares **before and after sanitising**: the tag sequence
of a normal document is *identical* (`tagsOf(clean) === tagsOf(raw)`), a normal `https` link keeps its
`href` **and** its title, and table cells keep their content. That is "功能完全不变" as an assertion
rather than a promise.

## The allow-list is explicit, readable, and not permissive

`ALLOWED_TAGS` (49 tags, grouped by role with a comment explaining why each group is there) and
`ALLOWED_ATTR` (`href`, `title`, `alt`, `colspan`, `rowspan`, `align`, `start`, `open`) live in one
module and are **exported**, so a reader can check them without reading the sanitizer call. Explicitly
absent: `style`, `id`, `class`, `src`, `srcset`, `formaction`, every `on*`, `script`, `img`,
`iframe`. `ALLOWED_UNKNOWN_PROTOCOLS: false` and an `ALLOWED_URI_REGEXP` limited to
`https?/mailto/tel/ftp`, so an unlisted scheme is dropped by construction. This satisfies
"白名单显式可查且非全放".

## The tests are not tautological — I proved it by mutation

The dispatch's fourth criterion. I mutated the implementation three times, in a copy, and restored
the file byte-identically afterwards (`diff` clean):

| mutation | result |
|---|---|
| `sanitizeHtml` returns its input unchanged (no sanitising) | **6 attack tests FAIL** |
| `ALLOWED_TAGS: []` (strip everything) | **4 normal-document tests FAIL** |
| **the call site** returns `sanitizeHtml(html)`'s input unchanged (bypass the sanitizer) | **all 14 tests still PASS** |

The first two prove the assertions bite. The third is the finding: **nothing pins the call site.** The
sanitizer's own tests render markdown themselves, so the one wiring that the defect was about —
`marked` → sanitize → `dangerouslySetInnerHTML` — has no test. A future edit that reverts line 407 to
`return html;` (or another view that injects `marked` output) passes everything. That is the same
shape this project keeps paying for: the check exists and the thing it stands for is unverified.

## Gates — run by me in Mira's worktree

`tsc --noEmit` clean · vitest **218/218** (15 files, the 14 new sanitize tests passing by name) ·
`pnpm build` green · `dompurify@3.4.17` in `package.json` **and** `pnpm-lock.yaml`, resolved from the
existing internal registry (the lockfile's tarball host is unchanged — `a.z01.normandyfunc.com`,
the Artifactory already in use), so the packet's "must be installable from the existing source" is
met.

## Honest limits

- **The rendered document was not observed.** No app was built or installed for this dispatch, the
  screen is locked from my seat (screencapture fails, assistive access denied), and
  `DocumentsWorkspace`/`AppV2` are imported **nowhere** — I re-verified that, so the path is still
  unreachable and my verification is at the unit level (the real functions, the real DOM), which is
  the only level at which this code currently runs.
- **`<img>` is dropped** (not on the allow-list). Measured: **0 live documents carry image
  markdown** (`!\[` or `<img`), so nothing is lost today — but when the document view is mounted, an
  image-bearing document will silently lose its images. Worth a line in the mounting decision, not a
  defect now.
- **No test pins the call site** (above). Recommended, not required for this verdict: a
  `DocumentsWorkspace` test that renders a poisoned `bodyText` and asserts the injected HTML has no
  `script`/`on*`/`javascript:`.
- `tauri.conf.json`'s `"csp": null` is untouched — the packet scoped this dispatch to sanitising, and
  Mira honoured that boundary. The mine is defused at the one place the packet asked for; the CSP
  remains a second line of defence nobody has built.

## Bottom line

**PASS.** The three payload classes lose their execution capability through a real DOM sanitizer,
a normal document renders with an identical tag sequence, the allow-list is explicit, readable and
non-permissive (verified against the view's own CSS), and the library came from the existing registry
and is locked. The tests bite — proven by mutation, in both directions. One gap for the record: the
call site itself has no test, so the sanitizer can be bypassed at the only place it matters without
a single test going red. That is the follow-up I would write first, and it is the same discipline the
verdict of the two holes recorded two days ago.
