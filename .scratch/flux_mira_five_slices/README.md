# Flux's measurement scripts for `disp-Tn-83tjvS_B6YgOV4dCCF`

Every number in `docs/coordination/tasks/flux/FLUX-2026-10-16-mira-five-slices-verdict-v1.md`
that is not a file:line fact comes from one of these. All are READ ONLY.

| Script | What it measures | Item |
|---|---|---|
| `flux_png_ascii2.py` | decodes a PNG in pure python (zlib + unfilter, no PIL on this machine) and renders it as ASCII by dominant colour, so the icon's mark can be judged by looking | 2 |
| `flux_ico_ascii.py` | extracts a size out of an `.ico` and renders it — this is how the 16x16 Dock judgement was made | 2 |
| `flux_hist_projects.py` | `list_history_projects` against the live DB: 38 / 183 / 36680, and where `(root)` sits in the returned order (position 6, not last) | 1 |
| `flux_hist_order.py` | every project's session list: dated-block newest-first, a dated row never after an undated one, per-project counts summing to the totals; plus a real session's turns checked line-by-line against the ORIGINAL file on disk | 1 |
| `flux_supervisor_feed.py` | the newest 200 ledger rows: how many the OLD switch would have rendered as a bare `• Type`, and how many the NEW translator leaves to its default branch | 4 |

Run them with the repo's own binary, e.g.

```
cd <checkout on track> && cargo build -p seatloom-mcp-emit
python3 .scratch/flux_mira_five_slices/flux_hist_projects.py
python3 .scratch/flux_mira_five_slices/flux_png_ascii2.py src-tauri/icons/32x32.png --cols 32
python3 .scratch/flux_mira_five_slices/flux_ico_ascii.py src-tauri/icons/icon.ico 16
```

The `psql` numbers (acceptance cards by status, the five `flux_verdict_ref`s, the 70-file merge
diff, the byte-identical file comparison) are one-liners recorded inline in the verdict doc.
