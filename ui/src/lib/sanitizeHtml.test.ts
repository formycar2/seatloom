// sanitizeHtml — the allow-list between `marked` and `innerHTML`.
//
// The defect (AEGIS-2026-10-21-sanitize-document-render-v1, measured on
// `track/infra-foundation`): `marked` emits `<script>`, `onerror=` and
// `javascript:` verbatim, and `DocumentsWorkspace` injects its output with
// `dangerouslySetInnerHTML`. Body text comes from database documents written by seats
// and Aegis. `tauri.conf.json` sets `"csp": null`, so there is no second line of
// defence. Rendering a document and executing it were the same action.
//
// The operator's ruling: "功能完全不变 —— 你看到的文档还是那个文档, 只是里面的纸条
// 不再被执行". That sentence is the test contract: the attacks must lose their
// execution capability, AND a normal document must render identically. Testing only the
// first direction would let a sanitiser that strips everything pass.

import { describe, expect, it } from 'vitest';
import { marked } from 'marked';
import { sanitizeHtml, ALLOWED_TAGS, ALLOWED_ATTR } from './sanitizeHtml';

/** Render markdown the way DocumentsWorkspace does, then sanitize. */
const renderDoc = (markdown: string) =>
  sanitizeHtml(marked.parse(markdown, { async: false }) as string);

describe('the three measured attack payloads lose their execution capability', () => {
  it('a <script> tag does not survive', () => {
    const out = renderDoc('hello\n\n<script>alert("executed")</script>');
    expect(out.toLowerCase()).not.toContain('<script');
    expect(out.toLowerCase()).not.toContain('alert(');
    // The prose around it is untouched.
    expect(out).toContain('hello');
  });

  it('an inline event-handler attribute does not survive', () => {
    const out = renderDoc('<img src=x onerror="alert(1)">');
    expect(out.toLowerCase()).not.toContain('onerror');
    expect(out.toLowerCase()).not.toContain('<img');
    // No `on*` of any kind, not just the one in the payload.
    expect(out.toLowerCase()).not.toMatch(/\son[a-z]+\s*=/);
  });

  it('a javascript: URL does not survive', () => {
    const out = renderDoc('[click me](javascript:alert(2))');
    expect(out.toLowerCase()).not.toContain('javascript:');
    expect(out.toLowerCase()).not.toContain('alert(');
  });

  it('an obfuscated javascript: URL does not survive either', () => {
    for (const payload of [
      '[a](JAVASCRIPT:alert(1))',
      '[a](  javascript:alert(1))',
      '[a](java&#115;cript:alert(1))',
    ]) {
      const out = renderDoc(payload);
      const flat = out.toLowerCase().replace(/&#x?[0-9a-f]+;/g, '');
      expect(flat, payload).not.toContain('javascript:');
      expect(flat, payload).not.toContain('alert(');
    }
  });

  it('a destination marked will not even parse stays inert text, which is also safe', () => {
    // A tab inside the destination means `marked` emits no anchor at all — the markdown
    // survives as plain text and nothing is executable. Asserted as its own case so a
    // future parser change cannot silently turn it into a live link.
    const out = renderDoc('[a](java\tscript:alert(1))');
    expect(out.toLowerCase()).not.toContain('href');
    expect(out).not.toContain('<a ');
  });

  it('a data: URI does not survive', () => {
    const out = renderDoc('[a](data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==)');
    expect(out.toLowerCase()).not.toContain('data:');
  });

  it('an inline style attribute does not survive (not on the allow-list)', () => {
    const out = renderDoc('<p style="position:fixed;top:0">x</p>');
    expect(out.toLowerCase()).not.toContain('style=');
    expect(out).toContain('x');
  });
});

describe('a normal document renders identically', () => {
  const doc = [
    '# Title',
    '',
    'A paragraph with **bold**, *italic* and `code`.',
    '',
    '## A list',
    '',
    '- one',
    '- two',
    '- three',
    '',
    '1. first',
    '2. second',
    '',
    '> a quote',
    '',
    '```js',
    'const x = 1;',
    '```',
    '',
    '| a | b |',
    '| - | - |',
    '| 1 | 2 |',
    '',
    '---',
    '',
    '[a link](https://example.com/doc)',
  ].join('\n');

  it('keeps every structural element the CSS is written against', () => {
    const out = renderDoc(doc);
    for (const tag of [
      'h1',
      'h2',
      'p',
      'strong',
      'em',
      'code',
      'pre',
      'ul',
      'ol',
      'li',
      'blockquote',
      'table',
      'thead',
      'tbody',
      'tr',
      'th',
      'td',
      'hr',
      'a',
    ]) {
      expect(out, tag).toContain(`<${tag}`);
    }
    expect(out).toContain('Title');
    expect(out).toContain('bold');
    expect(out).toContain('const x = 1;');
    expect(out).toContain('a quote');
  });

  it('keeps a normal http(s) link usable', () => {
    const out = renderDoc(doc);
    expect(out).toContain('href="https://example.com/doc"');
    // and the link text survives with it
    expect(out).toContain('>a link<');
  });

  it("renders the same structure before and after sanitising (the operator's ruling, tested)", () => {
    const raw = marked.parse(doc, { async: false }) as string;
    const clean = sanitizeHtml(raw);
    // Same tag sequence — sanitiser removed nothing from a normal document.
    const tagsOf = (html: string) => (html.match(/<\/?([a-z0-9]+)/gi) ?? []).join(' ');
    expect(tagsOf(clean)).toBe(tagsOf(raw));
  });

  it("leaves a markdown table's cell content intact", () => {
    const out = renderDoc('| a | b |\n| - | - |\n| 1 | 2 |');
    expect(out).toContain('<td>1</td>');
    expect(out).toContain('<td>2</td>');
  });
});

describe('the allow-list is an allow-list', () => {
  it('names the tags and attributes it permits, so a reader can check them', () => {
    expect(ALLOWED_TAGS).toContain('pre');
    expect(ALLOWED_TAGS).toContain('th');
    expect(ALLOWED_TAGS).not.toContain('script');
    expect(ALLOWED_TAGS).not.toContain('img');
    expect(ALLOWED_TAGS).not.toContain('iframe');
    expect(ALLOWED_ATTR).toContain('href');
    expect(ALLOWED_ATTR).not.toContain('style');
    expect(ALLOWED_ATTR).not.toContain('src');
  });

  it('does not permit an event handler or a script tag under any spelling', () => {
    const denied = ['script', 'iframe', 'object', 'embed', 'form', 'input', 'button'];
    for (const tag of denied) expect(ALLOWED_TAGS).not.toContain(tag);
    for (const attr of ['onclick', 'onerror', 'onload', 'style', 'src', 'formaction']) {
      expect(ALLOWED_ATTR).not.toContain(attr);
    }
  });

  it('keeps HTML a document author wrote as text, not as markup', () => {
    // A document that literally shows the reader some HTML must not execute it.
    const out = sanitizeHtml('<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>');
    expect(out.toLowerCase()).not.toContain('<script');
    expect(out).toContain('&lt;script&gt;');
  });
});
