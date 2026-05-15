// tests/targets/bun-full/app.test.tsx
//
// Test suite séparée — chargée uniquement par `bun test` (le binaire de
// runner enregistre describe/it/test au top-level). Lancée par `bun app.tsx`
// ces hooks seraient morts (pollution silencieuse).

import { describe, expect, it, mock, test } from "bun:test";
import {
	FilledButton,
	MotionDuration,
	MotionEasing,
	hashPassword,
	verifyPassword,
} from "./app";

describe("M3 motion tokens", () => {
	it("respecte la spec M3 (durations en ms)", () => {
		expect(MotionDuration.short1).toBe(50);
		expect(MotionDuration.extraLong4).toBe(1000);
	});

	it("emphasized = bezier (0.2, 0, 0, 1)", () => {
		expect(MotionEasing.emphasized).toContain("cubic-bezier");
	});
});

describe("FilledButton", () => {
	it("retourne du JSX renderable", () => {
		const node = <FilledButton label="OK" />;
		expect(node).toBeTruthy();
	});
});

test("Bun.password roundtrip", async () => {
	const hash = await hashPassword("super-secret");
	expect(await verifyPassword("super-secret", hash)).toBe(true);
	expect(await verifyPassword("wrong", hash)).toBe(false);
});

// Mock un module entier (Bun 1.2+) — appliqué AVANT que les imports
// résolvent au-dessus, sinon no-op. Pour les CSS imports, le loader CSS
// natif est antérieur au mock module : préférer `--preload` pour ces cas.
mock.module("./styles/m3-tokens.css", () => ({}));
