// sanitizeHtml — the allow-list that stands between `marked` and `innerHTML`.
//
// WHY THIS EXISTS. AEGIS-2026-10-21-sanitize-document-render-v1. `marked` passes
// `<script>`, `onerror=`, and `javascript:` through its output verbatim (measured on
// `track/infra-foundation`, not assumed), and `DocumentsWorkspace` injects that output
// with `dangerouslySetInnerHTML`. The body comes from database documents, written by
// seats and Aegis. Rendering a document and executing it are the same action in a
// browser — there is no "display but do not run" mode — and `tauri.conf.json` has
// `"csp": null`, so there is no second line of defence.
//
// The operator's ruling (2026-10-21): "功能完全不变 —— 你看到的文档还是那个文档,
// 只是里面的纸条不再被执行". So this module preserves the typography and removes the
// execution capability, and nothing else.
//
// # Why an allow-list and not a blocklist
//
// A blocklist enumerates what is known to be dangerous today; an allow-list enumerates
// what the product actually renders. A document viewer needs a small, closed set of
// formatting tags. Anything else — including anything invented after this file was
// written — is dropped by construction rather than by omission. The operator's
// acceptance criteria include "不许为了让测试过去而把白名单开成全放过", which is why the
// lists below are explicit and why the tests assert on both directions.
//
// # Scope of this change
//
// This does NOT mount or enable the document view (that is a separate decision the
// operator has not made). It defuses a path that is currently unreachable — `AppV2` is
// dead code — so that mounting it later does not arm it.

import DOMPurify from 'dompurify';

/**
 * Tags a coordination document needs. Everything else is removed.
 *
 * Grouped by role so the next person can see why each is here rather than having to
 * guess from a flat list of names.
 */
export const ALLOWED_TAGS = [
  // Section headings — T5 acceptance cards and contracts lean on them heavily.
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  // Body text.
  'p',
  'br',
  'hr',
  'div',
  'span',
  // Emphasis.
  'strong',
  'b',
  'em',
  'i',
  'u',
  's',
  'del',
  'ins',
  'sub',
  'sup',
  'small',
  'mark',
  'abbr',
  // Lists.
  'ul',
  'ol',
  'li',
  'dl',
  'dt',
  'dd',
  // Code (inline and fenced) — the most common tag in these documents.
  'code',
  'pre',
  'kbd',
  'samp',
  'var',
  // Quotes and details.
  'blockquote',
  'q',
  'cite',
  'details',
  'summary',
  'figcaption',
  'figure',
  // Tables.
  'table',
  'thead',
  'tbody',
  'tfoot',
  'tr',
  'th',
  'td',
  'caption',
  'colgroup',
  'col',
  // Links.
  'a',
];

/**
 * Attributes a document needs. Note what is NOT here: no `style`, no `id`, no
 * `class`, no `srcset`, no `formaction`, and no `on*` of any kind. `href` is present
 * because a link without a destination is not a link; `title` because a link with a
 * tooltip is one of the few affordances the current CSS relies on.
 */
export const ALLOWED_ATTR = [
  'href',
  'title',
  'alt',
  'colspan',
  'rowspan',
  'align',
  'start',
  'open',
];

/**
 * URL schemes a link may use. `javascript:` is the one that matters, and it is
 * excluded by construction: this is an allow-list, so a scheme must be listed to
 * survive.
 *
 * `mailto:` and `tel:` are listed because coordination documents reference people;
 * `http`/`https` because they reference sources. Relative links (`#section`,
 * `./file.md`) carry no scheme and are matched by DOMPurify's
 * `ALLOW_UNKNOWN_PROTOCOLS: false`.
 */
export const ALLOWED_URI_REGEXP = /^(?:https?|mailto|tel|ftp):/i;

/** A DOMPurify config that allows nothing beyond the lists above. */
export const SANITIZE_CONFIG = {
  ALLOWED_TAGS: [...ALLOWED_TAGS],
  ALLOWED_ATTR: [...ALLOWED_ATTR],
  ALLOWED_URI_REGEXP,
  // Not `true`: an unknown scheme must be dropped, not kept.
  ALLOW_UNKNOWN_PROTOCOLS: false,
  // Data URIs can carry script in some contexts and are not needed by these docs.
  ALLOW_DATA_ATTR: false,
  // The documents are trusted prose, not a sandboxed application surface.
  WHOLE_DOCUMENT: false,
  RETURN_DOM: false,
  RETURN_DOM_FRAGMENT: false,
  // Keep this strict: the default behaviour on a stripped attribute is to remove it,
  // which is what we want, and there is no reason to relax it.
  KEEP_CONTENT: true,
};

/**
 * Strip the execution capability from `marked`'s output while keeping the typography.
 *
 * Returns the sanitized HTML string. The function is total: a `null`/`undefined`
 * window (server-side, or a test without jsdom) yields the input unchanged rather
 * than throwing, because a sanitizer that crashes its caller is not a sanitizer —
 * but the production path always has a window, so it always sanitizes.
 */
export function sanitizeHtml(dirty: string): string {
  if (typeof window === 'undefined' || typeof document === 'undefined') {
    // No DOM to sanitize against — DOMPurify needs one. Better to return the input
    // and let the caller decide than to swallow the problem silently: the
    // DocumentsWorkspace path is desktop-only, where a window always exists.
    return dirty;
  }
  return DOMPurify.sanitize(dirty, SANITIZE_CONFIG);
}

export default sanitizeHtml;
