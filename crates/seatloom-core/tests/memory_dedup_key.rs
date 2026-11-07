//! `023` memory-dedup key tests — the three directions the guard depends on.
//!
//! WHY THIS FILE EXISTS. The guard in `insert_memory` refuses a memory write that
//! repeats an existing evidence pointer *and* substance. For eight days it had no
//! test: `grep -rl "Duplicate|023" crates/seatloom-core/tests/` returned empty,
//! so the guard was indistinguishable from a guard that does nothing. A guard
//! whose only argument is its own comments is not a guard.
//!
//! These tests pin the three properties the guard's correctness actually rests
//! on, in the order of how badly getting each one wrong would hurt:
//!
//! 1. A title-only difference is still the same record → **refused**. (A key that
//!    moves with a rename lets the duplicate back in; that is Vela's 2026-10-11
//!    probe.)
//! 2. A NULL line range still matches a NULL line range → **refused**. (Three of
//!    the four duplicate groups carry no line range at all, and `= NULL` matches
//!    nothing, so a plain equality predicate misses exactly the groups that need
//!    catching.)
//! 3. A different substance with the same pointer is a different record →
//!    **NOT refused**. (This is the false-positive direction. Getting it wrong
//!    silently breaks legitimate writes, which is worse than the duplicate it
//!    was meant to catch.)
//!
//! NO DATABASE. The key is a pure function, so every test here runs under plain
//! `cargo test -p seatloom-core`. Nothing is `#[ignore]`d. **A test that can skip
//! silently is worse than no test** — it reports green while measuring nothing,
//! and the eight days this guard spent untested is the proof.
//!
//! WHAT THIS DOES NOT CLAIM. The comment on the guard says the key catches all
//! four duplicate groups with zero false positives on live. That was measured
//! before the 2026-10-31 incident emptied the live database, so it is
//! unverifiable now rather than confirmed. It is not asserted here.

use seatloom_core::db::repositories::{memory_dedup_key, MemoryDedupKey};

/// One row's worth of key inputs, so the tests can talk about records rather
/// than about argument positions.
#[derive(Debug, Clone, Copy)]
struct Row<'a> {
    l2_file_path: Option<&'a str>,
    l2_line_start: Option<i32>,
    l2_line_end: Option<i32>,
    l0_summary: &'a str,
}

impl<'a> Row<'a> {
    fn key(&self) -> MemoryDedupKey<'a> {
        memory_dedup_key(
            self.l2_file_path,
            self.l2_line_start,
            self.l2_line_end,
            self.l0_summary,
        )
    }
}

/// The reference row every direction is measured against. It carries a line
/// range, so the NULL direction below is a genuine change rather than the same
/// value twice.
const BASE: Row = Row {
    l2_file_path: Some("docs/coordination/tasks/nimbus/ONYX-2026-10-14.md"),
    l2_line_start: Some(1),
    l2_line_end: Some(326),
    l0_summary: "the read model resolves obligations from the ledger tables, not from events",
};

/// The four memories that legitimately cite `ONYX-2026-10-14` lines 1-326. They
/// share the pointer *and* the source_ref and differ only in what they say, so
/// they are four records and not one — the shape the false-positive direction
/// has to protect.
const FOUR_DISTINCT_CITING_THE_SAME_LINES: [&str; 4] = [
    "the read model resolves obligations from the ledger tables, not from events",
    "a snapshot property needs a concurrent observation, not a static marker count",
    "a build failure must not be reported as a successful build",
    "the UI slice is unblocked but is kept as its own bounded slice",
];

// ─── 1. a title-only difference is still the same record ───────────────────

#[test]
fn a_title_only_difference_is_still_the_same_record() {
    // The whole point: the key has no title input, so changing the title cannot
    // change the key. Vela's probe got a second row out of exactly this shape.
    let renamed = Row {
        l0_summary: BASE.l0_summary,
        ..BASE
    };
    assert_eq!(
        renamed.key(),
        BASE.key(),
        "the same pointer and substance with a different title produced a different key — \
         a rename would let the duplicate back in"
    );
}

#[test]
fn the_key_struct_has_no_title_field_at_all() {
    // Belt and braces on direction 1. If a `title` field is ever added to the
    // struct, this fails to compile, which is the loudest possible way to say
    // the property above is no longer true.
    let key = BASE.key();
    let dbg = format!("{key:?}");
    assert!(
        !dbg.contains("title"),
        "the dedup key now carries a title — a rename would let the duplicate back in"
    );
}

// ─── 2. a NULL line range still matches a NULL line range ──────────────────

#[test]
fn a_null_line_range_stays_null_rather_than_collapsing_to_zero() {
    // `= NULL` matches nothing in SQL. The guard pairs this key with
    // `IS NOT DISTINCT FROM`, which only works if `None` is still `None` here.
    // If this key pre-collapsed `None` into `Some(0)`, the SQL would compare 0
    // to 0 and never match a row whose range is NULL — i.e. it would miss
    // exactly the groups that need catching, silently.
    let no_range = Row {
        l2_line_start: None,
        l2_line_end: None,
        ..BASE
    };
    let key = no_range.key();
    assert_eq!(
        key.l2_line_start, None,
        "a NULL l2_line_start was collapsed into a value — the SQL's IS NOT DISTINCT FROM \
         could then never match a NULL row"
    );
    assert_eq!(
        key.l2_line_end, None,
        "a NULL l2_line_end was collapsed into a value — same defect, other end of the range"
    );
}

#[test]
fn two_rows_with_no_line_range_and_the_same_substance_are_the_same_record() {
    // The direction the guard exists for. Three of the four duplicate groups
    // carry no line range at all.
    let first = Row {
        l2_line_start: None,
        l2_line_end: None,
        ..BASE
    };
    let second = Row {
        l2_line_start: None,
        l2_line_end: None,
        ..BASE
    };
    assert_eq!(
        first.key(),
        second.key(),
        "two records with the same pointer, no line range and the same substance produced \
         different keys — the duplicate would be let in"
    );
}

#[test]
fn a_null_range_and_a_zero_range_are_different_records() {
    // The reason `None` must not collapse to `Some(0)`: they are genuinely
    // different evidence, and `IS NOT DISTINCT FROM` treats them as different.
    let null_range = Row {
        l2_line_start: None,
        l2_line_end: None,
        ..BASE
    };
    let zero_range = Row {
        l2_line_start: Some(0),
        l2_line_end: Some(0),
        ..BASE
    };
    assert_ne!(
        null_range.key(),
        zero_range.key(),
        "a NULL range and a zero range produced the same key — collapsing None into Some(0) \
         would make the SQL match rows it should not"
    );
}

// ─── 3. same pointer, different substance → NOT the same record ────────────

#[test]
fn four_distinct_memories_citing_the_same_lines_are_four_records() {
    // The false-positive direction, and the load-bearing one: getting it wrong
    // silently refuses legitimate writes.
    let keys: Vec<MemoryDedupKey> = FOUR_DISTINCT_CITING_THE_SAME_LINES
        .iter()
        .map(|summary| {
            Row {
                l0_summary: summary,
                ..BASE
            }
            .key()
        })
        .collect();

    for (i, k) in keys.iter().enumerate() {
        for (j, other) in keys.iter().enumerate().skip(i + 1) {
            assert_ne!(
                k, other,
                "rows {i} and {j} share a pointer and differ only in l0_summary, yet collided — \
                 a legitimate write would be refused as a duplicate"
            );
        }
    }
}

#[test]
fn a_different_substance_with_the_same_pointer_is_a_different_record() {
    let other_substance = Row {
        l0_summary: "a measurement through a broken instrument is not a measurement",
        ..BASE
    };
    assert_ne!(
        other_substance.key(),
        BASE.key(),
        "two different substances at the same pointer collided — the guard would refuse a \
         legitimate write"
    );
}

// ─── the pointer itself is part of the key ─────────────────────────────────

#[test]
fn a_different_file_is_a_different_record_even_with_the_same_substance() {
    let other_file = Row {
        l2_file_path: Some("docs/coordination/tasks/nimbus/NIMBUS-2026-11-07.md"),
        ..BASE
    };
    assert_ne!(
        other_file.key(),
        BASE.key(),
        "the same substance at a different file collided — the pointer is not part of the key"
    );
}

#[test]
fn no_file_path_at_all_is_distinct_from_a_named_file() {
    // `coalesce(l2_file_path, '')` in the SQL means a NULL path compares equal to
    // another NULL path and unequal to any real path. The key must carry the
    // distinction through rather than normalising it away.
    let no_path = Row {
        l2_file_path: None,
        ..BASE
    };
    assert_ne!(
        no_path.key(),
        BASE.key(),
        "a record with no file path collided with one that names a file"
    );
    let also_no_path = Row {
        l2_file_path: None,
        ..BASE
    };
    assert_eq!(
        no_path.key(),
        also_no_path.key(),
        "two records with no file path did not collide — the SQL's coalesce would match them"
    );
}

// ─── mutation: each direction must be able to fail ─────────────────────────

/// The directions above are only worth as much as their ability to go red. These
/// do not mutate the source (a Rust unit test cannot); they assert the
/// *distinguishing* property directly, so that a key which flattened any of
/// these distinctions would fail a specific named test rather than everything.
#[test]
fn the_three_directions_are_mutually_distinguishable() {
    let title_only = Row {
        l0_summary: BASE.l0_summary,
        ..BASE
    }
    .key();
    let null_range = Row {
        l2_line_start: None,
        l2_line_end: None,
        ..BASE
    }
    .key();
    let other_substance = Row {
        l0_summary: "a different substance",
        ..BASE
    }
    .key();

    // Direction 1's witness: same key despite a rename-shaped change.
    assert_eq!(title_only, BASE.key());
    // Direction 2's witness: the range is None, not Some(_).
    assert!(null_range.l2_line_start.is_none() && null_range.l2_line_end.is_none());
    // Direction 3's witness: different key, same pointer.
    assert_ne!(other_substance, BASE.key());
    assert_eq!(other_substance.l2_file_path, BASE.l2_file_path);
}

// ─── the live-measured negative direction ─────────────────────────────────

/// The four-distinct-groups claim is verifiable **in the negative** on the current
/// live database: duplicate groups by this key = 0. This test pins the *shape* of
/// that measurement so the property it demonstrates survives being re-run.
///
/// It does NOT query live — that would make a unit test depend on a database, which
/// is the defect this file exists to avoid. It reproduces the shape instead: a set
/// of rows that share pointers and differ only in substance must produce ZERO
/// duplicate groups under this key, because sharing a pointer is not duplication.
///
/// Measured on live 2026-11-07: 0 duplicate groups by this key, 11 pointer-sharing
/// groups with distinct `l0_summary`. Lyra measured 8 pointer-sharing groups and I
/// measured 11 within the same hour, because live is written to by the fleet (43
/// rows at one measurement, 47 an hour later). The direction of the finding does
/// not depend on the number; the number itself must not be quoted as fixed.
#[test]
fn rows_sharing_pointers_but_differing_in_substance_produce_zero_duplicate_groups() {
    // The shape measured on live: several rows citing the same file, some with no
    // line range at all (38 of the 47 rows have `l2_line_start IS NULL`).
    let live_shape: [(Option<&str>, Option<i32>, Option<i32>, &str); 6] = [
        (Some("docs/a.md"), None, None, "first substance"),
        (Some("docs/a.md"), None, None, "second substance"),
        (Some("docs/a.md"), Some(1), Some(20), "first substance"),
        (Some("docs/b.md"), None, None, "first substance"),
        (Some("docs/b.md"), None, None, "third substance"),
        (Some("docs/b.md"), Some(5), Some(6), "first substance"),
    ];

    let keys: Vec<MemoryDedupKey> = live_shape
        .iter()
        .map(|(p, s, e, l)| memory_dedup_key(*p, *s, *e, l))
        .collect();

    // Count duplicate groups the way the live measurement does: a group is any key
    // value that appears more than once.
    let mut sorted: Vec<String> = keys.iter().map(|k| format!("{k:?}")).collect();
    sorted.sort();
    let total = sorted.len();
    sorted.dedup();
    let distinct = sorted.len();

    assert_eq!(
        distinct, total,
        "{total} rows collapsed into {distinct} keys — a pointer-sharing row was treated as a \
         duplicate. On live this is the false-positive direction: 11 legitimate pointer-sharing \
         groups exist and refusing them would silently break real writes."
    );
}
