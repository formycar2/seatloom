// Image token estimation and concluded-image dropping (packet
// AEGIS-2026-10-22-image-token-estimation-v1).
//
// The bug this file exists to pin down: `ESTIMATED_IMAGE_CHARS = 4800` counted
// every image as ~1200 tokens regardless of its real size. Measured on a live
// SeatLoom seat, screenshots average ~1.4MB (~350k tokens) — a ~300x
// underestimate, which made `shouldCompact` never fire and re-sent the images
// every turn. 6 images were 67% of that context.
//
// Both halves are covered here, in BOTH directions, because a fix that only
// proves "the number got bigger" could just as well be an unconditional purge:
//
//   1. estimateTokens: a large image counts as large.
//   2. dropConcludedImages (via buildSessionContext): an image with a later
//      conclusion loses its base64 and the conclusion survives.
//   3. dropConcludedImages: an image with NO conclusion is kept — the
//      counter-example that makes (2) meaningful.
//   4. The last two images are never dropped, even with conclusions after them.

import type { AgentMessage } from "@saih/agent-core";
import { describe, expect, it } from "vitest";
import { estimateTokens } from "../src/core/compaction/index.ts";
import { buildSessionContext, type SessionEntry, type SessionMessageEntry } from "../src/core/session-manager.ts";

// ============================================================================
// Fixtures
// ============================================================================

/** A ~1MB base64 payload. Length is what matters; content is irrelevant. */
function largeImageData(): string {
	return "A".repeat(1024 * 1024);
}

/** A user turn carrying one image, optionally alongside text. */
function imageUserEntry(id: string, parentId: string | null, data: string): SessionMessageEntry {
	return {
		type: "message",
		id,
		parentId,
		timestamp: "2025-01-01T00:00:00Z",
		message: {
			role: "user",
			content: [{ type: "text", text: "这是截图,你看一下" }, { type: "image", data, mimeType: "image/png" }],
			timestamp: 1,
		},
	};
}

/** An assistant turn with text — a conclusion about whatever preceded it. */
function assistantTextEntry(id: string, parentId: string | null, text: string): SessionMessageEntry {
	return {
		type: "message",
		id,
		parentId,
		timestamp: "2025-01-01T00:00:00Z",
		message: {
			role: "assistant",
			content: [{ type: "text", text }],
			api: "anthropic-messages",
			provider: "anthropic",
			model: "claude-test",
			usage: {
				input: 1,
				output: 1,
				cacheRead: 0,
				cacheWrite: 0,
				totalTokens: 2,
				cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
			},
			stopReason: "stop",
			timestamp: 1,
		},
	};
}

function userTextEntry(id: string, parentId: string | null, text: string): SessionMessageEntry {
	return {
		type: "message",
		id,
		parentId,
		timestamp: "2025-01-01T00:00:00Z",
		message: { role: "user", content: text, timestamp: 1 },
	};
}

/** Link the chain so buildSessionProjection sees one linear path. */
function chained(entries: SessionMessageEntry[]): SessionEntry[] {
	return entries.map((entry, index) => ({ ...entry, parentId: index === 0 ? null : entries[index - 1].id }));
}

/** Every image block's base64 in a projected context, flattened. */
function imagePayloads(messages: AgentMessage[]): string[] {
	const payloads: string[] = [];
	for (const message of messages) {
		if (message.role !== "user" && message.role !== "toolResult") continue;
		if (typeof message.content === "string") continue;
		for (const block of message.content) {
			if (block.type === "image") payloads.push(block.data);
		}
	}
	return payloads;
}

/** Every text block in a projected context, flattened. */
function allText(messages: AgentMessage[]): string {
	const parts: string[] = [];
	for (const message of messages) {
		if (typeof message.content === "string") {
			parts.push(message.content);
			continue;
		}
		for (const block of message.content) {
			if (block.type === "text") parts.push(block.text);
		}
	}
	return parts.join("\n");
}

// ============================================================================
// 1. The estimate itself
// ============================================================================

describe("estimateTokens counts an image at its real payload size", () => {
	it("a 1MB image is estimated as ~1MB, not ~4800 chars", () => {
		const data = largeImageData();
		const message: AgentMessage = {
			role: "user",
			content: [{ type: "image", data, mimeType: "image/png" }],
			timestamp: 1,
		};

		const tokens = estimateTokens(message);

		// The old constant produced 4800/4 = 1200 tokens for this message. Assert
		// the order of magnitude instead of an exact figure, so a change to the
		// chars/4 divisor does not silently invalidate the test — but keep the
		// floor high enough that 1200 fails loudly.
		expect(tokens).toBeGreaterThan(200_000);
		// And an upper bound: a 1MB image must not be counted as 10MB. Without
		// this the first assertion alone could pass on a nonsense inflate.
		expect(tokens).toBeLessThan(300_000);
	});

	it("a tiny image is still estimated as tiny — the fix is not 'images are huge'", () => {
		const message: AgentMessage = {
			role: "user",
			content: [{ type: "image", data: "AAAA", mimeType: "image/png" }],
			timestamp: 1,
		};

		// 4 chars of payload -> 1 token. The estimate tracks the payload, it does
		// not assume a size class.
		expect(estimateTokens(message)).toBe(1);
	});

	it("an image with no payload is counted at the cap, never as cheap", () => {
		// A reference rather than inline bytes. This is the branch where guessing
		// small would resurrect the original bug: an unknown image counted as
		// ~1200 tokens makes shouldCompact skip a turn that is in fact enormous.
		const message: AgentMessage = {
			role: "user",
			content: [{ type: "image", mimeType: "image/png" } as never],
			timestamp: 1,
		};

		expect(estimateTokens(message)).toBeGreaterThan(200_000);
	});

	it("a pathological image is capped so one block cannot decide the whole history", () => {
		// normalizeToolResultImages can pass an original through when the backend
		// is unavailable, so a huge payload is reachable. The cap keeps a single
		// block from dominating both compaction decisions.
		const message: AgentMessage = {
			role: "user",
			content: [{ type: "image", data: "A".repeat(64 * 1024 * 1024), mimeType: "image/png" }],
			timestamp: 1,
		};

		const tokens = estimateTokens(message);
		// 8MB cap -> ~2M tokens, not ~16M.
		expect(tokens).toBeLessThanOrEqual((8 * 1024 * 1024) / 4);
		expect(tokens).toBeGreaterThan(1_000_000);
	});
});

// ============================================================================
// 2 & 3. The drop, in both directions
// ============================================================================

describe("buildSessionContext replaces concluded images and keeps the rest", () => {
	it("a concluded image loses its base64 while the conclusion survives", () => {
		const data = largeImageData();
		const entries = chained([
			imageUserEntry("e1", null, data),
			assistantTextEntry("e2", "e1", "图里是一个空的项目列表,底部没有分页器。"),
			userTextEntry("e3", "e2", "那我们继续"),
		]);

		const { messages } = buildSessionContext(entries);

		// The base64 is gone.
		expect(imagePayloads(messages)).not.toContain(data);
		// The conclusion is not.
		expect(allText(messages)).toContain("图里是一个空的项目列表");
		// And something in its place says what happened — a turn must not read as
		// if it produced nothing.
		expect(allText(messages)).toContain("此处原有一张截图");
	});

	it("an image with no conclusion yet is kept — the drop is not a purge", () => {
		// THE COUNTER-EXAMPLE. If the implementation dropped every image, the test
		// above would still pass. This one is what makes "已结论覆盖" mean anything.
		const data = largeImageData();
		const entries = chained([
			imageUserEntry("e1", null, data),
			userTextEntry("e2", "e1", "先看看这个,我还没说话"),
		]);

		const { messages } = buildSessionContext(entries);

		expect(imagePayloads(messages)).toContain(data);
	});

	it("the two most recent images survive even when conclusions follow them", () => {
		// The operator is sometimes mid-comparison: "is this the same as that one?"
		// Those are the current subject, not history.
		const older = largeImageData();
		const recent1 = "B".repeat(1024 * 1024);
		const recent2 = "C".repeat(1024 * 1024);
		const entries = chained([
			imageUserEntry("e1", null, older),
			assistantTextEntry("e2", "e1", "第一张是旧的界面。"),
			imageUserEntry("e3", "e2", recent1),
			assistantTextEntry("e4", "e3", "第二张是新的。"),
			imageUserEntry("e5", "e4", recent2),
			assistantTextEntry("e6", "e5", "第三张也是新的,和前一张几乎一样。"),
			userTextEntry("e7", "e6", "好"),
		]);

		const { messages } = buildSessionContext(entries);
		const payloads = imagePayloads(messages);

		expect(payloads).not.toContain(older);
		expect(payloads).toContain(recent1);
		expect(payloads).toContain(recent2);
	});

	it("an image in a toolResult is dropped the same way", () => {
		// The measured case was `toolResult:image` (a screenshot tool), not a
		// user attachment, so the toolResult path needs its own coverage.
		const data = largeImageData();
		const imageResult: SessionMessageEntry = {
			type: "message",
			id: "e1",
			parentId: null,
			timestamp: "2025-01-01T00:00:00Z",
			message: {
				role: "toolResult",
				toolCallId: "call-1",
				toolName: "read",
				isError: false,
				content: [{ type: "image", data, mimeType: "image/png" }],
				timestamp: 1,
			},
		};
		const entries = chained([imageResult, assistantTextEntry("e2", "e1", "截图里显示构建成功了。")]);

		const { messages } = buildSessionContext(entries);

		expect(imagePayloads(messages)).not.toContain(data);
		expect(allText(messages)).toContain("截图里显示构建成功了");
	});
});
