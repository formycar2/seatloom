# Delivery — `disp-90h4J3F_iAHmTV-myGARs` (Flux): the document-render sanitization, independently verified

- **Dispatch**: `disp-90h4J3F_iAHmTV-myGARs`, issued by seat-aegis-001, kind `verification`.
  Acceptance = Aegis's packet `AEGIS-2026-10-21-sanitize-document-render-v1.md`.
- **Covers implementation dispatch**: **`disp-FzldOwJuxCfxc2TGq9dMw`** — named in the verdict's
  `event_object_refs` (`ref_type='dispatch'`).
- **Branch**: `packet/mira/knowledge-panel` @ `4ed8c76`, verified in Mira's own worktree (clean).
- **Verdict**: **PASS** — with one real gap named (the call site is untested).
- **Verdict event**: `ev-ucN0gvx2vIB_MmDMiZEgz` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-21-sanitize-document-render-verdict-v1.md` | new — the verdict, per criterion |
| `docs/coordination/tasks/flux/disp-90h4J3F_iAHmTV-myGARs-delivery.md` | this note |

No source file was changed. The three mutations I ran to test non-tautology were applied to a copy
of `sanitizeHtml.ts` / `DocumentsWorkspace.tsx` and **restored byte-identically** (`diff` clean, and
the suite re-run green afterwards).

## The criteria, one line each

1. **No execution capability, measured by me** — `<script>`, `on*` attributes (any spelling),
   `javascript:` (plus `JAVASCRIPT:`, leading-whitespace and entity-obfuscated forms), `data:` and
   `vbscript:` all lose it, through a real jsdom DOM (not a regex); surrounding prose survives.
2. **Typography unchanged** — the tag sequence of a normal document is *identical* before/after
   sanitising (`tagsOf(clean) === tagsOf(raw)`), a normal `https` link keeps `href` and `title`, table
   cells keep content; and every `.sl-doc-body` CSS selector's tag is on the allow-list.
3. **Allow-list explicit and not permissive** — `ALLOWED_TAGS`/`ALLOWED_ATTR` exported, grouped with
   reasons; `ALLOW_UNKNOWN_PROTOCOLS: false` + an https?/mailto/tel/ftp URI regexp, so an unlisted
   scheme is dropped by construction.
4. **Tests cannot be tautological** — proven by mutation (see below).

## The mutation evidence

| mutation | result |
|---|---|
| sanitizer returns its input (no sanitising) | **6 attack tests FAIL** |
| `ALLOWED_TAGS: []` (strip everything) | **4 normal-document tests FAIL** |
| **call site** returns the unsanitized html | **all 14 tests still PASS** |

The first two prove the assertions bite in both directions. The third is the finding: **the
`marked` → sanitize → `dangerouslySetInnerHTML` wiring the defect was about has no test.** The
sanitizer's tests render markdown themselves, so reverting `DocumentsWorkspace.tsx:407` to
`return html;` — or another view injecting `marked` output — passes everything. Recommended
follow-up: a `DocumentsWorkspace` test rendering a poisoned `bodyText` and asserting the injected
HTML carries no `script` / `on*` / `javascript:`.

## Gates, run by me in Mira's worktree

`tsc --noEmit` clean · vitest **218/218** (15 files; the 14 new tests pass by name) · `pnpm build`
green · `dompurify@3.4.17` in `package.json` **and** the lockfile, resolved from the existing
internal registry (lockfile host unchanged) — the packet's installability requirement is met.

## Honest limits

- **The rendered document was not observed**: no app built or installed for this dispatch, the screen
  is locked from my seat, and `DocumentsWorkspace`/`AppV2` are imported **nowhere** — I re-verified
  that, so the path is still unreachable and verification is at the unit level (the real functions,
  the real DOM), which is the only level this code runs at today.
- **`<img>` is dropped** (not allow-listed); measured **0 live documents carry image markdown**, so
  nothing is lost today — worth a line when the view is mounted.
- `tauri.conf.json`'s `"csp": null` is untouched, correctly so (the packet scoped this to sanitising).
