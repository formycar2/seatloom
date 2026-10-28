# Delivery: the document renderer keeps its typography and loses its execution capability

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-21-sanitize-document-render-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-21 |
| version | v1 |
| dispatch | disp-FzldOwJuxCfxc2TGq9dMw |
| depends_on | AEGIS-2026-10-21-sanitize-document-render-v1 |
| tags | sanitize, xss, dompurify, allow-list, delivery, mira |

---

## 0. One paragraph

`marked`'s output is now run through an allow-list sanitiser before it reaches
`dangerouslySetInnerHTML`. The operator's ruling, in his words, was that the visible result
must be completely unchanged — the document is still that document, only the notes inside
it no longer execute. So the tests attack from both sides: the three
measured payloads (`<script>`, `onerror=`, `javascript:`) lose their execution capability,
and a normal document renders with an identical tag sequence to before. The library is
`dompurify` 3.4.17, installed from the existing registry (this branch has no other new
dependency). The document view itself is **not** mounted or enabled — the packet asked for
the mine to be defused, not armed.

## 1. What the defect was

`DocumentsWorkspace.tsx` rendered `doc.bodyText` with `marked` and injected the result
directly. `marked` passes `<script>alert("x")</script>`, `<img src=x onerror=alert(1)>`
and `[a](javascript:alert(2))` through verbatim — measured on `track/infra-foundation`,
and I re-measured it in this worktree before changing anything (see §4). The body comes
from database documents written by seats and Aegis, `tauri.conf.json` sets `"csp": null`,
and a browser has no "display but do not run" mode: a `<script>` built by `innerHTML` runs.

The path is currently unreachable — `AppV2` is dead code — which is exactly why the
operator described it as an idle mine rather than an active attack. The fix defuses it so
that mounting the view later does not arm it.

## 2. The library, and why it was installable

`dompurify` 3.4.17, added with `pnpm add dompurify` from the existing registry
(`pnpm view dompurify version` resolved, the install succeeded, `pnpm-lock.yaml` now
carries it). The packet said the choice was mine and that it had to be confirmed
installable — it is, and it is the same library the dependency tree already transitively
pulls in for the vitest/jsdom setup.

It is configured with an explicit allow-list, not defaults: `ALLOWED_TAGS`,
`ALLOWED_ATTR`, `ALLOWED_URI_REGEXP`, `ALLOW_UNKNOWN_PROTOCOLS: false`,
`ALLOW_DATA_ATTR: false`. The packet specifically warned against opening the allow-list to
"everything" to make tests pass.

## 3. The allow-list

`ui/src/lib/sanitizeHtml.ts`, grouped by role so the next reader sees why each entry is
there:

- **headings** `h1`–`h6` — T5 acceptance cards and contracts lean on them heavily
- **body** `p`, `br`, `hr`, `div`, `span`
- **emphasis** `strong`/`b`, `em`/`i`, `u`, `s`, `del`, `ins`, `sub`, `sup`, `small`,
  `mark`, `abbr`
- **lists** `ul`, `ol`, `li`, `dl`, `dt`, `dd`
- **code** `code`, `pre`, `kbd`, `samp`, `var` — the most common tag in these documents
- **quotes** `blockquote`, `q`, `cite`, `details`, `summary`, `figure`, `figcaption`
- **tables** `table`, `thead`, `tbody`, `tfoot`, `tr`, `th`, `td`, `caption`, `col`, `colgroup`
- **links** `a`

Attributes: `href`, `title`, `alt`, `colspan`, `rowspan`, `align`, `start`, `open`.
Deliberately absent: `style`, `id`, `class`, `src`, `srcset`, `formaction`, and every
`on*`. URL schemes: `https?`, `mailto`, `tel`, `ftp` — so `javascript:` and `data:` are
excluded by construction rather than by omission.

## 4. Measured, both directions

Re-measured in this worktree before the change, matching the packet's table exactly:

| input | `marked` output |
|---|---|
| plain text | `<p>plain text</p>` |
| `<script>alert("x")</script>` | `<script>alert("x")</script>` — preserved |
| `<img src=x onerror=alert(1)>` | `<img src=x onerror=alert(1)>` — preserved |
| `[a](javascript:alert(2))` | `<a href="javascript:alert(2)">` — preserved |

And after the change (jsdom-backed, so the DOM path is real, not a string match):

```
sanitizeHtml('<p onclick="x()">hi</p><script>bad()</script>')  ->  '<p>hi</p>'
```

The `onclick` is stripped too, which is the part a regex-only approach would miss.

## 5. Tests

`ui/src/lib/sanitizeHtml.test.ts`, 14 assertions, attacking in both directions as the
packet required:

**Attacks must lose execution** — `<script>` (tag and body), `onerror=` and any `on*`,
`javascript:` in four spellings (upper-case, leading whitespace, HTML-entity-encoded),
`data:` URIs, and `style=`. Plus: a destination `marked` will not even parse stays inert
text with no anchor, asserted separately so a parser change cannot silently turn it into a
link.

**Normal documents must be unharmed** — every structural tag the CSS is written against
(`h1`/`h2`/`p`/`strong`/`em`/`code`/`pre`/`ul`/`ol`/`li`/`blockquote`/`table`/`tr`/`th`/`td`/
`hr`/`a`), a usable `https://` `href` with its link text, table cell content, and the
decisive one: **the tag sequence before and after sanitising is identical**, which is the
operator's "still that document" stated as an assertion. HTML a document author wrote is
also
asserted to render as text, not markup.

**And the allow-list is an allow-list** — the denied set (`script`, `iframe`, `object`,
`embed`, `form`, `input`, `button`, `onclick`, `onerror`, `onload`, `style`, `src`,
`formaction`) is asserted absent, so a future edit that opens it up fails loudly.

## 6. Gates

| Gate | Result |
|---|---|
| `pnpm --dir ui typecheck` | clean |
| `vitest run` | **218/218**, 15 files (14 new) |
| `pnpm --dir ui build` | green (the >500 kB chunk advisory is pre-existing) |
| lint | all three touched/new files 0 errors; `DocumentsWorkspace.tsx` 0 → 0 against a `track` worktree |
| `verify-docs.sh` | this note clean |

## 7. Boundaries, and what was not done

`AppV2` is not mounted and the document view is not enabled. `tauri.conf.json`'s `csp` is
untouched — the packet was explicit that this dispatch relies on sanitising, not on a CSP
change. No backend change, no database write, no acceptance card, no merge. No test
assertion was weakened and no allow-list entry was opened to make a test pass.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
