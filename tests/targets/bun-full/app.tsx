// tests/targets/bun-full/app.tsx
//
// Fixture canonique n2b — exhaustive Bun-natif + Material Design 3.
//
// Couvre :
//   - Bun.serve full-stack (HTML routes, WebSocket upgrade, fetch, error)
//   - JSX natif (sans React DOM build) — `react-jsx` runtime via tsconfig
//   - Loader CSS natif (Bun bundler) — `import "./globals.css"`
//   - Hot reload (Bun.serve { development.hmr } + import.meta.hot)
//   - Bun.build avec plugins, minifier, loader custom, macros
//   - bun:test runner — séparé dans `app.test.tsx` (sinon pollution morte
//     quand on lance `bun app.tsx` hors mode test)
//   - bun:sqlite (file-backed embedded DB, WAL — `using` côté call-site
//     uniquement, pas dans le constructeur factory)
//   - Bun.SQL (Postgres tagged template literals)
//   - Bun.redis (singleton client lazy — pas Bun.RedisClient)
//   - Bun.spawn / Bun.$ shell
//   - Bun.file/Bun.write (lazy, atomic, sendfile/copy_file_range)
//   - Bun.password (bcrypt/argon2id natif)
//   - Bun.Glob, Bun.S3Client (NB : Bun.cron n'existe pas — schedule via
//     systemd timer / cron Linux)
//   - HTMLRewriter (streaming HTML rewriting)
//   - Bun.CryptoHasher, Bun.hash (xxHash3, wyhash)
//   - bun:ffi (dlopen + inline cc())
//   - bun build --compile (single-file executable cross-compile)
//
// Material Design 3 — composants + tokens basés sur https://m3.material.io
//   - Color roles (primary, on-primary, surface, surface-container, etc.)
//   - Elevation tokens (level0..level5)
//   - Motion (https://m3.material.io/styles/motion/overview/how-it-works) :
//       * Duration tokens : short1..short4, medium1..medium4, long1..long4, extra-long1..4
//       * Easing tokens : standard, emphasized, emphasized-decelerate, emphasized-accelerate
//       * Transitions implémentées via CSS custom properties + transitions
//   - Shape (full, large, medium, small, extra-small)
//   - Typography (display, headline, title, body, label × small/medium/large)
//
// Cible n2b : 0 finding 'imports/bun-native', 0 finding 'api/*' (déjà tout-Bun).
// Le scan détecte les bonnes pratiques (info), pas les régressions.

import { Database } from "bun:sqlite";
import { sql } from "bun";
import { dlopen, FFIType } from "bun:ffi";
import { renderToReadableStream } from "react-dom/server";

// NOTE: les tests bun:test vivent dans `app.test.tsx` (séparation pour
// éviter la pollution morte si on lance `bun app.tsx` hors mode test).

// ---------------------------------------------------------------------------
// 1. CSS imports (Bun natif — pas de webpack/vite/postcss)
// ---------------------------------------------------------------------------

import "./styles/m3-tokens.css";
import "./styles/m3-motion.css";

// ---------------------------------------------------------------------------
// 2. Material Design 3 — design tokens TypeScript
//    Source: https://m3.material.io/styles/motion/overview/how-it-works
// ---------------------------------------------------------------------------

export const MotionDuration = {
	short1: 50,
	short2: 100,
	short3: 150,
	short4: 200,
	medium1: 250,
	medium2: 300,
	medium3: 350,
	medium4: 400,
	long1: 450,
	long2: 500,
	long3: 550,
	long4: 600,
	extraLong1: 700,
	extraLong2: 800,
	extraLong3: 900,
	extraLong4: 1000,
} as const;

export const MotionEasing = {
	standard: "cubic-bezier(0.2, 0, 0, 1)",
	standardAccelerate: "cubic-bezier(0.3, 0, 1, 1)",
	standardDecelerate: "cubic-bezier(0, 0, 0, 1)",
	emphasized: "cubic-bezier(0.2, 0, 0, 1)", // M3 emphasized = spring-like
	emphasizedAccelerate: "cubic-bezier(0.3, 0, 0.8, 0.15)",
	emphasizedDecelerate: "cubic-bezier(0.05, 0.7, 0.1, 1)",
	legacy: "cubic-bezier(0.4, 0, 0.2, 1)",
} as const;

export const ColorRole = {
	primary: "var(--md-sys-color-primary)",
	onPrimary: "var(--md-sys-color-on-primary)",
	primaryContainer: "var(--md-sys-color-primary-container)",
	onPrimaryContainer: "var(--md-sys-color-on-primary-container)",
	secondary: "var(--md-sys-color-secondary)",
	onSecondary: "var(--md-sys-color-on-secondary)",
	surface: "var(--md-sys-color-surface)",
	surfaceContainer: "var(--md-sys-color-surface-container)",
	surfaceContainerHigh: "var(--md-sys-color-surface-container-high)",
	onSurface: "var(--md-sys-color-on-surface)",
	onSurfaceVariant: "var(--md-sys-color-on-surface-variant)",
	outline: "var(--md-sys-color-outline)",
	error: "var(--md-sys-color-error)",
	onError: "var(--md-sys-color-on-error)",
} as const;

export const Elevation = {
	level0: "var(--md-sys-elevation-level0)",
	level1: "var(--md-sys-elevation-level1)",
	level2: "var(--md-sys-elevation-level2)",
	level3: "var(--md-sys-elevation-level3)",
	level4: "var(--md-sys-elevation-level4)",
	level5: "var(--md-sys-elevation-level5)",
} as const;

export const Shape = {
	none: "0",
	extraSmall: "4px",
	small: "8px",
	medium: "12px",
	large: "16px",
	extraLarge: "28px",
	full: "9999px",
} as const;

// ---------------------------------------------------------------------------
// 3. Composants M3 (JSX natif Bun — pas de React import nécessaire pour
//    le runtime, mais on conserve les types via `react/jsx-runtime`)
// ---------------------------------------------------------------------------

interface FilledButtonProps {
	label: string;
	onClick?: string; // attribute string for client hydration
	disabled?: boolean;
	icon?: string;
}

export function FilledButton({
	label,
	onClick,
	disabled,
	icon,
}: FilledButtonProps) {
	// M3 filled-button : forme `full`, primary container, state layer
	// Transition motion : medium2 / emphasized
	const styleTransition = `background-color ${MotionDuration.medium2}ms ${MotionEasing.emphasized}, box-shadow ${MotionDuration.short3}ms ${MotionEasing.standardDecelerate}`;
	return (
		<button
			class="m3-button m3-button--filled"
			style={`transition: ${styleTransition}; border-radius: ${Shape.full};`}
			disabled={disabled}
			onClick={onClick}
		>
			{icon ? <span class="m3-button__icon">{icon}</span> : null}
			<span class="m3-button__label">{label}</span>
		</button>
	);
}

interface CardProps {
	title: string;
	body: string;
	variant?: "elevated" | "filled" | "outlined";
}

export function Card({ title, body, variant = "elevated" }: CardProps) {
	const elevation =
		variant === "elevated" ? Elevation.level1 : Elevation.level0;
	// Motion : long1 emphasized-decelerate au scroll-in (M3 spec)
	const enterMotion = `opacity ${MotionDuration.long1}ms ${MotionEasing.emphasizedDecelerate}`;
	return (
		<article
			class={`m3-card m3-card--${variant}`}
			style={`box-shadow: ${elevation}; border-radius: ${Shape.medium}; transition: ${enterMotion};`}
		>
			<h3 class="m3-card__title md-typescale-title-medium">{title}</h3>
			<p class="m3-card__body md-typescale-body-medium">{body}</p>
		</article>
	);
}

export function App() {
	return (
		<main
			class="m3-app"
			style={`background: ${ColorRole.surface}; color: ${ColorRole.onSurface};`}
		>
			<header class="m3-top-app-bar md-typescale-title-large">
				<h1>Bun + Material Design 3</h1>
			</header>
			<section class="m3-grid">
				<Card
					title="Bun.serve"
					body="Full-stack HTTP + WebSocket en un appel"
				/>
				<Card
					title="bun:sqlite"
					body="Embedded SQLite avec WAL + using statements"
					variant="filled"
				/>
				<Card
					title="bun:ffi"
					body="dlopen + inline C via cc()"
					variant="outlined"
				/>
			</section>
			<footer>
				<FilledButton label="Action principale" icon="✓" />
			</footer>
		</main>
	);
}

// ---------------------------------------------------------------------------
// 4. SQLite embarqué (bun:sqlite, file-backed, WAL, using statements)
// ---------------------------------------------------------------------------

function openDb(path: string): Database {
	// NOTE : pas de `using` ici — le scope de cette fonction se termine au
	// `return`, ce qui appellerait `Symbol.dispose` AVANT que les appelants
	// utilisent la DB. Le `using` se justifie côté call-site, dans le bloc
	// qui englobe l'usage complet.
	const db = new Database(path, { create: true, strict: true });
	db.exec("PRAGMA journal_mode = WAL;");
	db.exec(`
    CREATE TABLE IF NOT EXISTS posts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      title TEXT NOT NULL,
      body TEXT NOT NULL,
      created_at INTEGER NOT NULL DEFAULT (unixepoch())
    );
  `);
	return db;
}

function insertPost(db: Database, title: string, body: string): number {
	const stmt = db.prepare<{ lastInsertRowid: number }, [string, string]>(
		"INSERT INTO posts (title, body) VALUES (?, ?) RETURNING id AS lastInsertRowid",
	);
	const row = stmt.get(title, body);
	return row?.lastInsertRowid ?? 0;
}

function* iteratePosts(
	db: Database,
): IterableIterator<{ id: number; title: string }> {
	// .iterate() pour streaming (pas de Vec entier en mémoire).
	const q = db.prepare<{ id: number; title: string }, []>(
		"SELECT id, title FROM posts ORDER BY id",
	);
	yield* q.iterate();
}

// ---------------------------------------------------------------------------
// 5. Bun.SQL (Postgres tagged template literals) + RedisClient
// ---------------------------------------------------------------------------

async function fetchUser(id: number) {
	// Bun.sql interpolation safe par défaut (parameterized).
	const rows = await sql`SELECT id, email FROM users WHERE id = ${id}`;
	return rows[0];
}

async function cacheGet(key: string): Promise<string | null> {
	// Bun.redis = singleton client lazy (Bun 1.2+). Pas besoin de
	// connect/close manuel — Bun gère le pool en interne.
	return await Bun.redis.get(key);
}

// ---------------------------------------------------------------------------
// 6. Bun.spawn + Bun.$ shell
// ---------------------------------------------------------------------------

async function gitHead(): Promise<string> {
	const proc = Bun.spawn(["git", "rev-parse", "HEAD"], {
		stdout: "pipe",
		stderr: "inherit",
	});
	const sha = await new Response(proc.stdout).text();
	return sha.trim();
}

async function listFiles(dir: string) {
	// Shell tagged template — pas d'echo des secrets, pas d'injection.
	const text = await Bun.$`find ${dir} -type f -name '*.tsx'`.text();
	return text.split("\n").filter(Boolean);
}

// ---------------------------------------------------------------------------
// 7. Bun.file / Bun.write — lazy I/O atomique
// ---------------------------------------------------------------------------

async function readConfig(path: string) {
	const file = Bun.file(path);
	if (!(await file.exists())) return null;
	return await file.json();
}

async function writeReport(path: string, report: object) {
	// Écriture atomique (sendfile/copy_file_range/splice sur Linux).
	await Bun.write(path, JSON.stringify(report, null, 2));
}

// ---------------------------------------------------------------------------
// 8. Bun.password — hashing natif (bcrypt + argon2id)
// ---------------------------------------------------------------------------

async function hashPassword(plain: string): Promise<string> {
	return await Bun.password.hash(plain, { algorithm: "argon2id" });
}

async function verifyPassword(plain: string, hash: string): Promise<boolean> {
	return await Bun.password.verify(plain, hash);
}

// ---------------------------------------------------------------------------
// 9. Bun.Glob, Bun.cron, Bun.S3Client, Bun.CookieMap
// ---------------------------------------------------------------------------

async function findTsxFiles(): Promise<string[]> {
	const glob = new Bun.Glob("**/*.tsx");
	const matches: string[] = [];
	for await (const file of glob.scan({ cwd: ".", onlyFiles: true })) {
		matches.push(file);
	}
	return matches;
}

// Bun n'expose pas d'API `Bun.cron` — schedule via setInterval ou un
// cron côté infra (systemd timer, cron Linux). On garde une fonction
// déclenchable manuellement à des fins de démonstration.
async function dailyJob() {
	const head = await gitHead();
	console.log(`[daily] run on ${head}`);
}

async function uploadAvatar(userId: number, body: ReadableStream) {
	const s3 = new Bun.S3Client({
		accessKeyId: Bun.env.S3_KEY ?? "",
		secretAccessKey: Bun.env.S3_SECRET ?? "",
		bucket: "avatars",
	});
	const key = `users/${userId}/avatar`;
	await s3.write(key, body);
	return s3.presign(key, { expiresIn: 3600 });
}

function parseCookies(header: string): Map<string, string> {
	// Bun.CookieMap accepte un objet/array, pas une string Cookie header
	// directement. Pour parser un header, on splitte explicitement.
	const out = new Map<string, string>();
	for (const part of header.split(";")) {
		const eq = part.indexOf("=");
		if (eq < 0) continue;
		out.set(
			part.slice(0, eq).trim(),
			decodeURIComponent(part.slice(eq + 1).trim()),
		);
	}
	return out;
}

// ---------------------------------------------------------------------------
// 10. HTMLRewriter — streaming HTML transform (CSS-selector based)
// ---------------------------------------------------------------------------

function injectThemeColor(html: Response, color: string): Response {
	return new HTMLRewriter()
		.on('meta[name="theme-color"]', {
			element(el) {
				el.setAttribute("content", color);
			},
		})
		.on("html", {
			element(el) {
				el.setAttribute("data-md-color-scheme", "dark");
			},
		})
		.transform(html);
}

// ---------------------------------------------------------------------------
// 11. Bun.CryptoHasher / Bun.hash
// ---------------------------------------------------------------------------

function fileFingerprint(bytes: Uint8Array): string {
	const hasher = new Bun.CryptoHasher("sha256");
	hasher.update(bytes);
	return hasher.digest("hex");
}

function quickHash(bytes: Uint8Array): bigint {
	return Bun.hash.xxHash3(bytes);
}

// ---------------------------------------------------------------------------
// 12. bun:ffi — dlopen + inline cc() (Phase 4 fixture)
// ---------------------------------------------------------------------------

const lib = (() => {
	if (process.platform === "linux") {
		return dlopen("libm.so.6", {
			cos: { args: [FFIType.f64], returns: FFIType.f64 },
		});
	}
	return null;
})();

function nativeCos(x: number): number {
	return lib?.symbols.cos(x) ?? Math.cos(x);
}

// ---------------------------------------------------------------------------
// 13. Bun.serve — full-stack (HTML routes + WebSocket upgrade + fetch)
// ---------------------------------------------------------------------------

const server = Bun.serve({
	port: 3000,
	hostname: "0.0.0.0",
	development: {
		// Hot reload côté serveur (Bun.serve { development: true })
		// et HMR côté client (import.meta.hot dans le navigateur).
		hmr: true,
		console: true,
	},
	// Routes typed (Bun 1.2+).
	routes: {
		"/api/posts": {
			async GET() {
				const db = openDb("./data/blog.db");
				const posts = Array.from(iteratePosts(db));
				return Response.json(posts);
			},
			async POST(req) {
				const body = (await req.json()) as { title: string; body: string };
				const db = openDb("./data/blog.db");
				const id = insertPost(db, body.title, body.body);
				return Response.json({ id }, { status: 201 });
			},
		},
		"/api/users/:id": async (req) => {
			const id = Number(req.params.id);
			const u = await fetchUser(id);
			return Response.json(u ?? { error: "not found" }, {
				status: u ? 200 : 404,
			});
		},
		"/health": new Response("ok", { status: 200 }),
	},
	async fetch(req, srv) {
		const url = new URL(req.url);

		// WebSocket upgrade
		if (url.pathname === "/ws") {
			const ok = srv.upgrade(req, { data: { connectedAt: Date.now() } });
			return ok ? undefined : new Response("upgrade failed", { status: 400 });
		}

		// SSR JSX — passe par react-dom/server (Bun n'a PAS de
		// Bun.renderToReadableStream natif). React 19 sait exploiter le
		// ReadableStream optimisé exposé par Bun (cf. facebook/react#28941).
		if (url.pathname === "/") {
			const stream = await renderToReadableStream(<App />);
			return new Response(stream, {
				headers: { "content-type": "text/html; charset=utf-8" },
			});
		}

		return new Response("not found", { status: 404 });
	},
	websocket: {
		open(ws) {
			ws.subscribe("global");
			ws.publish("global", JSON.stringify({ event: "user-joined" }));
		},
		message(ws, message) {
			ws.publish("global", message);
		},
		close(ws, code, reason) {
			console.log(`ws closed code=${code} reason=${reason}`);
		},
	},
	error(err) {
		return Response.json({ error: err.message }, { status: 500 });
	},
});

console.log(`Listening on ${server.url}`);

// ---------------------------------------------------------------------------
// 14. Bun.build — bundler programmatique avec plugins, minifier, loader,
//     macros (et single-file executable via --compile)
// ---------------------------------------------------------------------------

export async function buildBundle() {
	const result = await Bun.build({
		entrypoints: ["./app.tsx"],
		outdir: "./dist",
		target: "browser",
		format: "esm",
		sourcemap: "linked",
		minify: { whitespace: true, syntax: true, identifiers: true },
		splitting: true,
		naming: {
			entry: "[dir]/[name]-[hash].[ext]",
			chunk: "[name]-[hash].[ext]",
			asset: "[name]-[hash].[ext]",
		},
		define: {
			"process.env.NODE_ENV": JSON.stringify("production"),
			__BUILD_TIME__: JSON.stringify(new Date().toISOString()),
		},
		loader: {
			".svg": "text",
			".png": "file",
		},
		plugins: [
			// Plugin custom (résolution alias)
			{
				name: "alias-md3",
				setup(build) {
					build.onResolve({ filter: /^@md3\// }, (args) => ({
						path: args.path.replace("@md3/", "./vendor/md3/"),
					}));
					build.onLoad({ filter: /\.css\?inline$/ }, async (args) => ({
						contents: `export default ${JSON.stringify(await Bun.file(args.path.replace("?inline", "")).text())};`,
						loader: "js",
					}));
				},
			},
		],
	});
	if (!result.success) {
		for (const log of result.logs) console.error(log);
		throw new Error("build failed");
	}
	return result;
}

// Single-file executable cross-compile :
//   bun build --compile --target=bun-linux-x64 --minify --sourcemap ./app.tsx
//   bun build --compile --target=bun-windows-x64 ./app.tsx
//   bun build --compile --target=bun-darwin-arm64 ./app.tsx

// ---------------------------------------------------------------------------
// 15. Test runner — voir `app.test.tsx` (séparé pour éviter pollution morte
//     hors `bun test`).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 16. Macros (Bun) — résolus à la compilation, non au runtime
// ---------------------------------------------------------------------------

import { buildId } from "./macros/build-id.ts" with { type: "macro" };

console.log(`build id: ${buildId()}`); // Inlined at compile time

// ---------------------------------------------------------------------------
// 17. Single-file executable embedded resources
// ---------------------------------------------------------------------------

// Pour `bun build --compile`, on peut embarquer des assets statiques :
//   import logo from "./assets/logo.png" with { type: "file" };
//   import schema from "./schema.json" with { type: "json" };
// Le binaire final inclut tout — déployable seul sur Linux/Windows/macOS.

export {
	fetchUser,
	cacheGet,
	uploadAvatar,
	injectThemeColor,
	fileFingerprint,
	quickHash,
	nativeCos,
	findTsxFiles,
	parseCookies,
	dailyJob,
	buildBundle,
	openDb,
	insertPost,
	hashPassword,
	verifyPassword,
};
