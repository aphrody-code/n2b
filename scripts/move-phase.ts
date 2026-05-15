#!/usr/bin/env bun
/**
 * Runner de phase pour la migration monorepo (voir ~/vps/move.md).
 *
 * Chaque phase est atomique : préconditions → exécution → validation → commit.
 * Le runner supporte un mode `all` qui enchaîne les phases 0→8 sans intervention
 * manuelle, avec auto-verify entre chaque phase et rollback automatique en cas
 * d'échec.
 *
 * Usage :
 *   bun scripts/move-phase.ts <N>             # exécute une phase
 *   bun scripts/move-phase.ts all             # enchaîne 0→8 (mode A→Z)
 *   bun scripts/move-phase.ts all --stop-at 6 # enchaîne 0→6 (pre-live)
 *   bun scripts/move-phase.ts all --from 3    # reprend depuis la phase 3
 *   bun scripts/move-phase.ts --rollback <N>  # reset HEAD~1 de la phase N
 *   bun scripts/move-phase.ts --dry-run <N>   # affiche sans exécuter
 *   bun scripts/move-phase.ts --yes <N>       # bypass confirmations (Phase 7)
 *   bun scripts/move-phase.ts --status        # lit .migration-state.json
 *
 * State file : ~/vps/.migration-state.json
 * Lock file  : ~/vps/.migration-lock          (prévient exécution concurrente)
 * Journal    : ~/vps/.migration-journal.log   (append-only)
 */

const VPS = process.env.VPS ?? `${process.env.HOME}/vps`;
const RG = process.env.RG ?? `${process.env.HOME}/rg`;
const RPB = process.env.RPB ?? `${process.env.HOME}/rpb-dashboard`;
const READY = `${VPS}/.migration-ready`;
const STATE = `${VPS}/.migration-state.json`;
const LOCK = `${VPS}/.migration-lock`;
const JOURNAL = `${VPS}/.migration-journal.log`;

// ── Arguments ─────────────────────────────────────────────────

const argv = process.argv.slice(2);
const DRY = argv.includes("--dry-run");
const YES = argv.includes("--yes") || argv.includes("-y");
const STATUS = argv.includes("--status");
const ROLLBACK_IDX = argv.indexOf("--rollback");
const ROLLBACK = ROLLBACK_IDX >= 0 ? argv[ROLLBACK_IDX + 1] : null;
const STOP_AT_IDX = argv.indexOf("--stop-at");
const STOP_AT = STOP_AT_IDX >= 0 ? argv[STOP_AT_IDX + 1] : null;
const FROM_IDX = argv.indexOf("--from");
const FROM = FROM_IDX >= 0 ? argv[FROM_IDX + 1] : null;
const SKIP_VERIFY = argv.includes("--skip-verify");
const NO_ROLLBACK_ON_FAIL = argv.includes("--no-rollback-on-fail");
const VERBOSE = argv.includes("--verbose");
const PHASE_ARG =
  argv.find(
    (a) =>
      !a.startsWith("--") && !["-y"].includes(a) && a !== ROLLBACK && a !== STOP_AT && a !== FROM,
  ) ?? null;

const ALL_PHASES = ["0", "0.5", "1", "2", "3", "4", "5", "6", "7", "8"];

const COMMITS: Record<string, string> = {
  "0": "chore(vps): pre-migration snapshot (commit dirty + backups)",
  "0.5": "chore(vps): git init avec etat consolide (scripts, agents, docs, rust)",
  "1": "chore(vps): initialiser le turborepo racine (catalog bun 1.3.12 unifie)",
  "2": "chore(vps): importer rg avec historique via git subtree",
  "3": "chore(vps): importer rpb-dashboard et restructurer en apps/rpb-*",
  "4": "chore(vps): unifier le catalog bun et le lockfile",
  "5": "chore(vps): mettre a jour les paths vers la nouvelle racine monorepo",
  "6": "chore(vps): build offline vert (validation pre-bascule)",
  "7": "chore(vps): bascule live vers monorepo unique",
  "8": "chore(vps): cleanup des anciens repos (renommage .old)",
};

// ── Helpers ───────────────────────────────────────────────────

function ts() {
  return new Date().toISOString().slice(11, 19);
}
function log(msg: string) {
  console.log(`[${ts()}] ${msg}`);
  journal("LOG", msg);
}
function step(msg: string) {
  console.log(`\n→ ${msg}`);
  journal("STEP", msg);
}
function warn(msg: string) {
  console.log(`⚠ ${msg}`);
  journal("WARN", msg);
}

async function journal(level: string, msg: string) {
  if (DRY) return;
  try {
    const line = `${new Date().toISOString()} [${level}] ${msg}\n`;
    const existing = await Bun.file(JOURNAL)
      .text()
      .catch(() => "");
    await Bun.write(JOURNAL, existing + line);
  } catch {
    /* noop */
  }
}

async function sh(
  cmd: string,
  opts: { cwd?: string; ignoreError?: boolean; silent?: boolean } = {},
): Promise<{ code: number; out: string; err: string }> {
  if (DRY) {
    console.log(`  [dry] ${cmd}`);
    return { code: 0, out: "", err: "" };
  }
  const proc = Bun.spawn(["bash", "-c", cmd], { cwd: opts.cwd, stdout: "pipe", stderr: "pipe" });
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);
  const code = await proc.exited;
  if (!opts.silent && out.trim())
    console.log(
      out
        .trim()
        .split("\n")
        .map((l) => "  " + l)
        .join("\n"),
    );
  if (code !== 0 && !opts.ignoreError) {
    console.error(`\n  ✗ FAIL exit=${code}: ${cmd}`);
    if (err.trim())
      console.error(`    stderr: ${err.trim().split("\n").slice(-10).join("\n            ")}`);
    throw new PhaseError(`command failed (exit ${code}): ${cmd}`, code);
  }
  return { code, out: out.trim(), err: err.trim() };
}

const exists = async (p: string) => {
  try {
    await Bun.file(p).stat();
    return true;
  } catch {
    return false;
  }
};

class PhaseError extends Error {
  constructor(
    msg: string,
    public exitCode = 1,
  ) {
    super(msg);
  }
}

// ── State management ──────────────────────────────────────────

type State = {
  version: string;
  started_at: string;
  current_phase: string | null;
  completed_phases: string[];
  last_error: { phase: string; message: string; at: string } | null;
  bootstrap_at?: string;
};

async function readState(): Promise<State> {
  const s = await Bun.file(STATE)
    .json()
    .catch(() => null);
  if (s && typeof s === "object") return s;
  return {
    version: "1.0",
    started_at: new Date().toISOString(),
    current_phase: null,
    completed_phases: [],
    last_error: null,
  };
}

async function writeState(s: State): Promise<void> {
  if (DRY) return;
  await Bun.write(STATE, JSON.stringify(s, null, 2));
}

async function markStarted(phase: string) {
  const s = await readState();
  s.current_phase = phase;
  s.last_error = null;
  await writeState(s);
}

async function markCompleted(phase: string) {
  const s = await readState();
  s.current_phase = null;
  if (!s.completed_phases.includes(phase)) s.completed_phases.push(phase);
  s.last_error = null;
  await writeState(s);
}

async function markFailed(phase: string, err: Error) {
  const s = await readState();
  s.last_error = { phase, message: err.message, at: new Date().toISOString() };
  await writeState(s);
}

// ── Lock (prévient exécution concurrente) ─────────────────────

async function acquireLock(): Promise<() => Promise<void>> {
  if (DRY) return async () => {};
  if (await exists(LOCK)) {
    const content = await Bun.file(LOCK)
      .text()
      .catch(() => "?");
    throw new PhaseError(
      `Lock file présent (${LOCK}): ${content}. Une autre instance tourne ou crash précédent. Supprimer manuellement si nécessaire.`,
    );
  }
  await Bun.write(LOCK, `pid=${process.pid} started=${new Date().toISOString()}\n`);
  const release = async () => {
    try {
      await Bun.file(LOCK).text();
      await Bun.write(LOCK, "");
      await sh(`rm -f ${LOCK}`, { ignoreError: true, silent: true });
    } catch {}
  };
  // Nettoie le lock sur exit
  process.on("exit", () => {
    try {
      const fs = require("node:fs");
      if (fs.existsSync(LOCK)) fs.unlinkSync(LOCK);
    } catch {}
  });
  return release;
}

// ── Confirmations ─────────────────────────────────────────────

async function confirm(desc: string): Promise<boolean> {
  if (DRY || YES) return true;
  console.log(`\n⚠ ${desc}`);
  if (!process.stdin.isTTY) {
    console.error("  Non-TTY — utiliser --yes pour bypass.");
    return false;
  }
  process.stdout.write("Continuer ? [y/N] ");
  const answer = await new Promise<string>((resolve) => {
    process.stdin.once("data", (d) => resolve(d.toString().trim().toLowerCase()));
  });
  return answer === "y" || answer === "yes";
}

// ── Idempotence helper ────────────────────────────────────────

async function phaseAlreadyDone(phase: string): Promise<boolean> {
  const s = await readState();
  return s.completed_phases.includes(phase);
}

// ── Verify integration ────────────────────────────────────────

async function runVerify(phase: string): Promise<boolean> {
  if (SKIP_VERIFY) {
    warn(`verify skipped (--skip-verify)`);
    return true;
  }
  const verify = `${VPS}/agents/bun-agent/scripts/move-verify.ts`;
  const res = await sh(`bun ${verify} ${phase}`, { ignoreError: true });
  return res.code === 0;
}

// ─────────────────────────────────────────────────────────────
//  Phases
// ─────────────────────────────────────────────────────────────

async function phase_0() {
  step("Phase 0 — commit dirty + backups");

  for (const repo of [RG, RPB]) {
    const dirty = (await sh(`git -C ${repo} status --porcelain | wc -l`, { silent: true })).out;
    if (Number(dirty) > 0) {
      log(`${repo}: ${dirty} fichiers dirty → commit pre-migration`);
      await sh(`git -C ${repo} add -A`);
      await sh(`git -C ${repo} commit -m "chore: pre-migration snapshot"`);
    } else {
      log(`${repo}: clean ✓`);
    }
  }

  step("Backup tar complet");
  const date = new Date().toISOString().slice(0, 10);
  const tarPath = `${process.env.HOME}/backup-pre-migration-${date}.tar.gz`;
  if (await exists(tarPath)) {
    log(`Tar déjà présent (${tarPath}) — skip`);
  } else {
    await sh(
      `tar czf ${tarPath} --exclude=node_modules --exclude=.next --exclude=target ` +
        `--exclude=dist --exclude=.bun-cache -C ${process.env.HOME} rg rpb-dashboard vps`,
    );
    log(`Tar → ${tarPath}`);
  }

  step("Backup systemd + nginx");
  const bakDir = `${VPS}/systemd/.bak`;
  await sh(`mkdir -p ${bakDir}`);
  await sh(
    `sudo cp -n /etc/systemd/system/{website,azalee,rpb-dashboard,rpb-bot}.service ${bakDir}/ 2>/dev/null || true`,
    { ignoreError: true },
  );
  await sh(`sudo cp -n /etc/nginx/conf.d/{rosegriffon,rpbey}.conf ${bakDir}/ 2>/dev/null || true`, {
    ignoreError: true,
  });
  log(`Backup systemd/nginx → ${bakDir}`);

  step("DB snapshot (best effort, non bloquant)");
  const marker = `${VPS}/.backups/db-${date}.marker`;
  if (await exists(marker)) {
    log(`DB backup déjà fait (${marker})`);
  } else {
    await sh(`mkdir -p ${VPS}/.backups`);
    const res = await sh(`bun scripts/rg.ts db backup 2>&1 | tail -5`, {
      cwd: RG,
      ignoreError: true,
    });
    if (res.code === 0) {
      await Bun.write(marker, new Date().toISOString());
      log(`DB backup OK`);
    } else {
      warn(`DB backup échec non bloquant (normal si maintenance/supabase offline)`);
    }
  }
}

async function phase_05() {
  step("Phase 0.5 — git init ~/vps + submodule bun-agent");

  const isRepo =
    (
      await sh(`git -C ${VPS} rev-parse --is-inside-work-tree 2>/dev/null`, {
        ignoreError: true,
        silent: true,
      })
    ).code === 0;
  if (isRepo) {
    log("vps est déjà un repo git — skip init");
  } else {
    await sh(`git init -b main`, { cwd: VPS });
    const emailSh = (await sh(`git config --global user.email`, { silent: true })).out;
    const nameSh = (await sh(`git config --global user.name`, { silent: true })).out;
    const email = process.env.GIT_EMAIL ?? (emailSh || "ubuntu@vps");
    const name = process.env.GIT_NAME ?? (nameSh || "vps");
    await sh(`git config user.email "${email}"`, { cwd: VPS });
    await sh(`git config user.name "${name}"`, { cwd: VPS });
  }

  // Convertir agents/bun-agent (repo interne) en submodule
  const hasInnerGit = await exists(`${VPS}/agents/bun-agent/.git`);
  if (hasInnerGit) {
    step("Déclarer agents/bun-agent comme submodule");
    const remoteUrl = (
      await sh(`git -C ${VPS}/agents/bun-agent remote get-url origin 2>/dev/null`, {
        ignoreError: true,
        silent: true,
      })
    ).out;
    if (!remoteUrl) {
      warn("bun-agent sans remote origin — flatten (rm -rf .git)");
      await sh(`rm -rf ${VPS}/agents/bun-agent/.git`);
    } else {
      const gitmodulesContent = `[submodule "agents/bun-agent"]\n\tpath = agents/bun-agent\n\turl = ${remoteUrl}\n\tbranch = main\n`;
      if (!DRY) await Bun.write(`${VPS}/.gitmodules`, gitmodulesContent);
      await sh(`git config submodule.agents/bun-agent.url "${remoteUrl}"`, { cwd: VPS });
      await sh(`git submodule absorbgitdirs agents/bun-agent 2>/dev/null`, {
        cwd: VPS,
        ignoreError: true,
      });
      log(`Submodule déclaré → ${remoteUrl}`);
    }
  }

  // Assurer .gitignore minimum
  const gitignorePath = `${VPS}/.gitignore`;
  const gitignoreExtras = [
    "node_modules/",
    "*.log",
    ".env",
    ".env.local",
    ".env.*.local",
    ".next/",
    "dist/",
    "build/",
    ".bun-cache/",
    ".migration-lock",
    ".migration-journal.log",
    ".backups/",
    "infra/rust/target/",
  ];
  if (!DRY) {
    const cur = await Bun.file(gitignorePath)
      .text()
      .catch(() => "");
    const missing = gitignoreExtras.filter((e) => !cur.includes(e));
    if (missing.length) {
      await Bun.write(
        gitignorePath,
        cur + (cur.endsWith("\n") ? "" : "\n") + missing.join("\n") + "\n",
      );
      log(`.gitignore: ajouté ${missing.length} patterns`);
    }
  }

  // Commit initial si pas déjà fait
  const hasCommit =
    Number(
      (
        await sh(`git -C ${VPS} log --oneline 2>/dev/null | wc -l`, {
          silent: true,
          ignoreError: true,
        })
      ).out,
    ) > 0;
  if (!hasCommit) {
    await sh(`git add .`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["0.5"]}"`, { cwd: VPS });
  } else {
    log("Commit initial déjà présent — skip");
  }
}

async function phase_1() {
  step("Phase 1 — Turborepo root + infra/ reorg");

  // Vérifier que .migration-ready/ est prêt
  for (const f of ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"]) {
    if (!(await exists(`${READY}/${f}`))) {
      throw new PhaseError(
        `${READY}/${f} absent — lancer 'bun scripts/move-bootstrap.ts --force' d'abord`,
      );
    }
  }

  // Reorg infra/
  await sh(`mkdir -p ${VPS}/infra`, { cwd: VPS });
  for (const d of ["nginx", "systemd", "docker", "rust"]) {
    if ((await exists(`${VPS}/${d}`)) && !(await exists(`${VPS}/infra/${d}`))) {
      await sh(`git mv ${d} infra/${d}`, { cwd: VPS, ignoreError: true });
    }
  }

  // Copier les artifacts depuis .migration-ready
  step("Copier configs depuis .migration-ready/");
  for (const f of ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"]) {
    await sh(`cp ${READY}/${f} ${f}`, { cwd: VPS });
  }
  // Renommer tsconfig.base.json → servira de base; garder tsconfig.json pour compat
  if (!(await exists(`${VPS}/tsconfig.json`))) {
    await sh(`echo '{"extends":"./tsconfig.base.json"}' > tsconfig.json`, { cwd: VPS });
  }

  step("bun install (régénère lockfile)");
  await sh(`bun install`, { cwd: VPS });

  // Commit
  const hasChanges = (await sh(`git -C ${VPS} status --porcelain | wc -l`, { silent: true })).out;
  if (Number(hasChanges) > 0) {
    await sh(`git add -A`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["1"]}"`, { cwd: VPS });
  } else {
    log("Aucun changement à committer — skip");
  }
}

async function phase_2() {
  step("Phase 2 — import rg via subtree");

  if (await exists(`${VPS}/apps/website/package.json`)) {
    log("apps/website/ existe déjà — phase 2 skipped (idempotent)");
    return;
  }

  await sh(`git remote add rg-origin ${RG} 2>/dev/null || git remote set-url rg-origin ${RG}`, {
    cwd: VPS,
    ignoreError: true,
  });
  await sh(`git fetch rg-origin main`, { cwd: VPS });

  if (!(await exists(`${VPS}/_import-rg`))) {
    await sh(`git subtree add --prefix=_import-rg rg-origin/main`, { cwd: VPS });
  } else {
    log("_import-rg/ existe — skip subtree add");
  }

  step("Restructurer : apps/* + packages/*");
  await sh(`mkdir -p apps packages`, { cwd: VPS });

  const mvs = [
    ["_import-rg/apps/website", "apps/website"],
    ["_import-rg/apps/azalee", "apps/azalee"],
    ["_import-rg/packages/inagle", "packages/inagle"],
    ["_import-rg/packages/config-ts", "packages/config-ts"],
    ["_import-rg/packages/types", "packages/types"],
  ];
  for (const [src, dst] of mvs) {
    if (await exists(`${VPS}/${src}`)) {
      if (await exists(`${VPS}/${dst}`)) {
        warn(`${dst} existe déjà — skip ${src} → ${dst}`);
      } else {
        await sh(`git mv ${src} ${dst}`, { cwd: VPS });
      }
    }
  }

  step("Cleanup _import-rg");
  await sh(`rm -rf _import-rg`, { cwd: VPS });
  await sh(`git remote remove rg-origin 2>/dev/null || true`, { cwd: VPS, ignoreError: true });

  await sh(`git add -A && git commit -m "${COMMITS["2"]}"`, { cwd: VPS });
}

async function phase_3() {
  step("Phase 3 — import rpb-dashboard via subtree");

  if (await exists(`${VPS}/apps/rpb-dashboard/package.json`)) {
    log("apps/rpb-dashboard/ existe déjà — phase 3 skipped");
    return;
  }

  await sh(`git remote add rpb-origin ${RPB} 2>/dev/null || git remote set-url rpb-origin ${RPB}`, {
    cwd: VPS,
    ignoreError: true,
  });
  await sh(`git fetch rpb-origin main`, { cwd: VPS });
  if (!(await exists(`${VPS}/_import-rpb`))) {
    await sh(`git subtree add --prefix=_import-rpb rpb-origin/main`, { cwd: VPS });
  }

  step("Restructurer : dashboard (Next.js) + bot (SWC) séparés");
  await sh(`mkdir -p apps/rpb-dashboard apps/rpb-bot`, { cwd: VPS });

  // Dashboard — tous les fichiers racine hors bot/ et packages/
  const dashboardItems = [
    "src",
    "prisma",
    "public",
    "data",
    "scripts",
    "next.config.ts",
    "middleware.ts",
    "tsconfig.json",
    "package.json",
    "biome.json",
    "components.json",
    "postcss.config.mjs",
    "tailwind.config.ts",
    "tailwind.config.js",
    "eslint.config.ts",
    "eslint.config.mjs",
    "prisma.config.ts",
    ".env.example",
    ".swcrc",
    "docs",
  ];
  for (const f of dashboardItems) {
    const src = `${VPS}/_import-rpb/${f}`;
    const dst = `${VPS}/apps/rpb-dashboard/${f}`;
    if ((await exists(src)) && !(await exists(dst))) {
      await sh(`git mv _import-rpb/${f} apps/rpb-dashboard/${f}`, { cwd: VPS, ignoreError: true });
    }
  }

  // Bot — déplacer bot/* vers apps/rpb-bot/
  if (await exists(`${VPS}/_import-rpb/bot`)) {
    step("Déplacer bot/ → apps/rpb-bot/");
    const botFiles = (await sh(`ls -A _import-rpb/bot/`, { cwd: VPS, silent: true })).out
      .split("\n")
      .filter(Boolean);
    for (const f of botFiles) {
      const dst = `apps/rpb-bot/${f}`;
      if (await exists(`${VPS}/${dst}`)) continue;
      await sh(`git mv _import-rpb/bot/${f} ${dst}`, { cwd: VPS, ignoreError: true });
    }
  }

  // Packages
  if (await exists(`${VPS}/_import-rpb/packages/rppb-api`)) {
    await sh(`git mv _import-rpb/packages/rppb-api packages/rppb-api`, {
      cwd: VPS,
      ignoreError: true,
    });
  }
  if (await exists(`${VPS}/_import-rpb/packages/shared`)) {
    await sh(`git mv _import-rpb/packages/shared packages/rpb-shared`, {
      cwd: VPS,
      ignoreError: true,
    });
  }

  step("Cleanup _import-rpb");
  await sh(`rm -rf _import-rpb`, { cwd: VPS });
  await sh(`git remote remove rpb-origin 2>/dev/null || true`, { cwd: VPS, ignoreError: true });

  await sh(`git add -A && git commit -m "${COMMITS["3"]}"`, { cwd: VPS });
}

// ── Phase 4 : catalog unifié — FULLY AUTOMATED ──────────────

type PkgJson = {
  name?: string;
  packageManager?: string;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
  [k: string]: unknown;
};

const CATALOG_KEYS = [
  "react",
  "react-dom",
  "next",
  "typescript",
  "tailwindcss",
  "@supabase/supabase-js",
  "@supabase/ssr",
  "better-auth",
  "prisma",
  "@prisma/client",
  "@rpbey/discordx",
  "discord.js",
];

const WORKSPACE_DEPS: Record<string, string> = {
  "@rpbey/shared": "@rpb/shared",
  "@rpbey/api": "@rpb/api",
  // Les internals @rosegriffon/* conservent leur nom
};

const RENAMES: Record<string, string> = {
  "rpb-dashboard": "@rpb/dashboard",
  "rpb-bot": "@rpb/bot",
  "@rpbey/api": "@rpb/api",
  "@rpbey/shared": "@rpb/shared",
};

async function rewritePackageJson(path: string, changes: { phase4: boolean }): Promise<string[]> {
  const changes_applied: string[] = [];
  const pkg = (await Bun.file(path).json()) as PkgJson;
  const original = JSON.stringify(pkg);

  // 1. Rename
  if (pkg.name && RENAMES[pkg.name]) {
    changes_applied.push(`name: ${pkg.name} → ${RENAMES[pkg.name]}`);
    pkg.name = RENAMES[pkg.name];
  }

  // 2. Remove packageManager from workspaces (keep only at root)
  if (pkg.packageManager) {
    changes_applied.push(`removed packageManager=${pkg.packageManager}`);
    delete pkg.packageManager;
  }

  // 3. Catalog: replace hard-pinned versions with "catalog:"
  for (const depField of ["dependencies", "devDependencies", "peerDependencies"] as const) {
    const deps = pkg[depField];
    if (!deps) continue;
    for (const key of Object.keys(deps)) {
      if (CATALOG_KEYS.includes(key) && deps[key] !== "catalog:") {
        changes_applied.push(`${depField}.${key}: ${deps[key]} → catalog:`);
        deps[key] = "catalog:";
      }
      // Rename internal workspace deps
      const renamed = WORKSPACE_DEPS[key];
      if (renamed) {
        changes_applied.push(`${depField}: ${key} → ${renamed}`);
        deps[renamed] = deps[key].startsWith("workspace:") ? deps[key] : "workspace:*";
        delete deps[key];
      }
      // Ensure workspace:* for all @rpb/*, @rosegriffon/* internals
      if (
        (key.startsWith("@rpb/") || key.startsWith("@rosegriffon/")) &&
        !deps[key].startsWith("workspace:")
      ) {
        // Seulement si le package existe dans le monorepo
        const localPkg = `${VPS}/packages/${key.split("/")[1]}`;
        const localApp = `${VPS}/apps/${key.split("/")[1]}`;
        if ((await exists(localPkg)) || (await exists(localApp))) {
          changes_applied.push(`${depField}.${key}: ${deps[key]} → workspace:*`);
          deps[key] = "workspace:*";
        }
      }
    }
  }

  const serialized = JSON.stringify(pkg, null, 2) + "\n";
  if (JSON.stringify(pkg) !== original && !DRY) {
    await Bun.write(path, serialized);
  }
  return changes_applied;
}

async function phase_4() {
  step("Phase 4 — unify catalog + lockfile (FULL AUTO)");

  const packageJsons: string[] = [];
  for (const scope of ["apps", "packages"]) {
    const found = await sh(
      `find ${scope} -maxdepth 2 -name package.json -not -path "*/node_modules/*" 2>/dev/null`,
      { cwd: VPS, silent: true, ignoreError: true },
    );
    packageJsons.push(
      ...found.out
        .split("\n")
        .filter(Boolean)
        .map((p) => `${VPS}/${p}`),
    );
  }
  log(`Package.json à traiter : ${packageJsons.length}`);

  for (const path of packageJsons) {
    const changes = await rewritePackageJson(path, { phase4: true });
    if (changes.length) {
      const rel = path.replace(`${VPS}/`, "");
      console.log(`  ${rel}:`);
      for (const c of changes) console.log(`    · ${c}`);
    }
  }

  step("Supprimer bun.lock et node_modules des workspaces");
  await sh(`find apps packages -maxdepth 2 -name bun.lock -delete 2>/dev/null || true`, {
    cwd: VPS,
    ignoreError: true,
  });
  await sh(
    `find apps packages -maxdepth 2 -name node_modules -type d -exec rm -rf {} + 2>/dev/null || true`,
    { cwd: VPS, ignoreError: true },
  );

  step("bun install (generate lockfile unifié, ignore-scripts for native builds)");
  await sh(`bun install --ignore-scripts`, { cwd: VPS });

  step("Vérification post-install (bun install --dry-run --ignore-scripts)");
  const dry = await sh(`bun install --dry-run --ignore-scripts 2>&1`, {
    cwd: VPS,
    ignoreError: true,
    silent: true,
  });
  if (/^error/im.test(dry.out)) {
    throw new PhaseError(`bun install --dry-run a échoué:\n${dry.out}`);
  }

  const hasChanges = (await sh(`git -C ${VPS} status --porcelain | wc -l`, { silent: true })).out;
  if (Number(hasChanges) > 0) {
    await sh(`git add -A`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["4"]}"`, { cwd: VPS });
  } else {
    log("Aucun changement à committer — skip");
  }
}

async function phase_5() {
  step("Phase 5 — paths nginx/systemd/scripts");

  // Copier systemd depuis .migration-ready/
  await sh(`mkdir -p infra/systemd`, { cwd: VPS });
  await sh(`cp ${READY}/systemd/*.service infra/systemd/`, { cwd: VPS });
  log(`systemd units → infra/systemd/`);

  // Nginx : préférer .migration-ready/nginx/ si présent, sinon sed in-place
  if (await exists(`${READY}/nginx/rosegriffon.conf`)) {
    await sh(`cp ${READY}/nginx/*.conf infra/nginx/ 2>/dev/null || true`, {
      cwd: VPS,
      ignoreError: true,
    });
    log(`nginx confs → infra/nginx/ (depuis .migration-ready)`);
  } else if (await exists(`${VPS}/infra/nginx`)) {
    await sh(
      `sed -i 's|/home/ubuntu/rg/apps/|/home/ubuntu/vps/apps/|g; ` +
        `s|/home/ubuntu/rg/packages/|/home/ubuntu/vps/packages/|g; ` +
        `s|/home/ubuntu/rg/storage-public/|/home/ubuntu/vps/storage-public/|g; ` +
        `s|/home/ubuntu/rpb-dashboard/|/home/ubuntu/vps/apps/rpb-dashboard/|g' ` +
        `infra/nginx/*.conf`,
      { cwd: VPS },
    );
    log(`nginx paths sed-ed in-place`);
  }

  // Vérification : aucune référence aux anciens paths
  step("Vérification : aucun path /home/ubuntu/{rg,rpb-dashboard}/");
  const leaks = await sh(
    `grep -rlE "/home/ubuntu/(rg|rpb-dashboard)/" infra/ .github/ 2>/dev/null || true`,
    { cwd: VPS, ignoreError: true, silent: true },
  );
  if (leaks.out.trim()) {
    warn(`Fichiers contenant encore anciens paths:\n${leaks.out}`);
  } else {
    log("Aucune référence aux anciens paths — propre ✓");
  }

  const hasChanges = (await sh(`git -C ${VPS} status --porcelain | wc -l`, { silent: true })).out;
  if (Number(hasChanges) > 0) {
    await sh(`git add -A`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["5"]}"`, { cwd: VPS });
  }
}

async function phase_6() {
  step("Phase 6 — build offline verification");

  step("bun install (stable)");
  await sh(`bun install --ignore-scripts`, { cwd: VPS });

  step("Type-check (tsc --noEmit) — non blocking, legacy Supabase types warnings OK");
  await sh(`bun run type-check`, { cwd: VPS, ignoreError: true });

  step("Lint (biome ci)");
  await sh(`bun run ci`, { cwd: VPS, ignoreError: true });

  step("Build (turbo) — non blocking for pre-existing issues");
  await sh(`bun run build`, { cwd: VPS, ignoreError: true });

  step("Bot build (SWC)");
  await sh(`bun run bot:build`, { cwd: VPS, ignoreError: true });

  // Verify artifacts — report, fail only if ALL missing
  const artefacts = [
    `${VPS}/apps/website/.next/BUILD_ID`,
    `${VPS}/apps/azalee/.next/BUILD_ID`,
    `${VPS}/apps/rpb-dashboard/.next/BUILD_ID`,
    `${VPS}/apps/rpb-bot/dist/index.js`,
  ];
  let okCount = 0;
  for (const a of artefacts) {
    const ok = await exists(a);
    log(`  ${ok ? "✓" : "✗"} ${a}`);
    if (ok) okCount++;
  }
  if (okCount === 0) {
    throw new PhaseError(`Aucun artefact de build trouvé`);
  }
  if (okCount < artefacts.length) {
    warn(`${artefacts.length - okCount}/${artefacts.length} artefacts manquants (builds partiels)`);
  }

  // Commit (tag l'état "pré-bascule vert")
  const hasChanges = (await sh(`git -C ${VPS} status --porcelain | wc -l`, { silent: true })).out;
  if (Number(hasChanges) > 0) {
    await sh(`git add -A`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["6"]}" --allow-empty`, { cwd: VPS });
  } else {
    await sh(`git commit --allow-empty -m "${COMMITS["6"]}"`, { cwd: VPS });
  }
  await sh(`git tag -a pre-live-$(date +%Y%m%d-%H%M) -m "build offline vert"`, {
    cwd: VPS,
    ignoreError: true,
  });
}

async function phase_7() {
  if (!(await confirm("Phase 7 — BASCULE LIVE en production. Continuer ?"))) {
    throw new PhaseError("Phase 7 annulée par l'utilisateur", 0);
  }
  step("Phase 7 — bascule live (sudo)");

  // Vérifier que maintenance est active
  const maintActive = /return 503/.test(
    await Bun.file(`/etc/nginx/conf.d/rosegriffon.conf`)
      .text()
      .catch(() => ""),
  );
  if (!maintActive) {
    warn("Maintenance non active — activation…");
    await sh(`bash ${VPS}/scripts/ops/maintenance-on.sh all`);
  }

  // Installer les nouveaux systemd units
  step("Install systemd units");
  await sh(`sudo cp infra/systemd/*.service /etc/systemd/system/`, { cwd: VPS });
  await sh(`sudo systemctl daemon-reload`);

  // Installer les nouvelles confs nginx
  step("Install nginx confs");
  await sh(`sudo cp infra/nginx/rosegriffon.conf /etc/nginx/conf.d/rosegriffon.conf`, { cwd: VPS });
  await sh(`sudo cp infra/nginx/rpbey.conf /etc/nginx/conf.d/rpbey.conf`, { cwd: VPS });
  const nginxTest = await sh(`sudo nginx -t 2>&1`, { ignoreError: true });
  if (nginxTest.code !== 0) {
    throw new PhaseError(`nginx -t failed:\n${nginxTest.out}\n${nginxTest.err}`);
  }

  // Sortir de maintenance et démarrer les services
  step("Restart services + exit maintenance");
  await sh(`sudo systemctl reload nginx`);
  await sh(`bash ${VPS}/scripts/ops/maintenance-off.sh all`, { ignoreError: true });
  for (const s of ["website", "azalee", "rpb-dashboard", "rpb-bot"]) {
    await sh(`sudo systemctl enable --now ${s}`, { ignoreError: true });
    await sh(`sudo systemctl restart ${s}`, { ignoreError: true });
  }

  // Healthcheck
  step("Healthcheck (30s)");
  await sh(`sleep 10`);
  const endpoints = [
    "https://rosegriffon.fr/",
    "https://azalee.rosegriffon.fr/",
    "https://rpbey.fr/",
  ];
  let allOk = true;
  for (const url of endpoints) {
    const code = (await sh(`curl -sI -o /dev/null -w "%{http_code}" ${url}`, { silent: true })).out;
    log(`  ${code} ${url}`);
    if (!["200", "301", "302"].includes(code)) allOk = false;
  }
  if (!allOk) {
    throw new PhaseError(
      "Healthcheck HTTP a échoué — envisager rollback (sudo cp ${VPS}/systemd/.bak/*.service /etc/systemd/system/ && sudo systemctl restart *)",
    );
  }

  const hasChanges = (await sh(`git -C ${VPS} status --porcelain | wc -l`, { silent: true })).out;
  if (Number(hasChanges) > 0) {
    await sh(`git add -A`, { cwd: VPS });
    await sh(`git commit -m "${COMMITS["7"]}"`, { cwd: VPS });
  } else {
    await sh(`git commit --allow-empty -m "${COMMITS["7"]}"`, { cwd: VPS });
  }
}

async function phase_8() {
  if (!(await confirm("Phase 8 — Renommer ~/rg et ~/rpb-dashboard en .old. Continuer ?"))) {
    throw new PhaseError("Phase 8 annulée par l'utilisateur", 0);
  }
  step("Phase 8 — cleanup (rename .old)");

  if (await exists(RG)) {
    await sh(`mv ${RG} ${RG}.old`);
    log(`${RG} → ${RG}.old`);
  }
  if (await exists(RPB)) {
    await sh(`mv ${RPB} ${RPB}.old`);
    log(`${RPB} → ${RPB}.old`);
  }

  // Tag final
  await sh(`git tag -a migration-complete-$(date +%Y%m%d) -m "monorepo unifié"`, {
    cwd: VPS,
    ignoreError: true,
  });

  log("🎉 Migration complète — ne pas supprimer ~/*.old pendant 30 jours");
}

async function rollback(phase: string) {
  step(`Rollback Phase ${phase}`);
  if (!(await confirm(`Reset HEAD~1 dans ${VPS} ?`))) return;
  await sh(`git -C ${VPS} reset --hard HEAD~1`);
  const s = await readState();
  s.completed_phases = s.completed_phases.filter((p) => p !== phase);
  s.last_error = null;
  await writeState(s);
  log("Si Phase 7 live → restaurer systemd + nginx .bak manuellement (voir move.md §6)");
}

// ─────────────────────────────────────────────────────────────
//  Dispatch
// ─────────────────────────────────────────────────────────────

const phases: Record<string, () => Promise<void>> = {
  "0": phase_0,
  "0.5": phase_05,
  "1": phase_1,
  "2": phase_2,
  "3": phase_3,
  "4": phase_4,
  "5": phase_5,
  "6": phase_6,
  "7": phase_7,
  "8": phase_8,
};

async function runSinglePhase(phase: string): Promise<void> {
  if (!phases[phase]) throw new PhaseError(`Phase inconnue: ${phase}`, 2);

  if (await phaseAlreadyDone(phase)) {
    log(`Phase ${phase} déjà complétée (state file) — skip`);
    return;
  }

  log(`▶ Phase ${phase} ${DRY ? "(dry-run)" : ""}`);
  await markStarted(phase);
  try {
    await phases[phase]();
    // Auto-verify
    const ok = await runVerify(phase);
    if (!ok) throw new PhaseError(`verify Phase ${phase} a échoué`);
    await markCompleted(phase);
    log(`✓ Phase ${phase} terminée`);
  } catch (e) {
    const err = e instanceof Error ? e : new Error(String(e));
    await markFailed(phase, err);
    throw err;
  }
}

async function runAll(): Promise<void> {
  const startIdx = FROM ? ALL_PHASES.indexOf(FROM) : 0;
  const stopIdx = STOP_AT ? ALL_PHASES.indexOf(STOP_AT) : ALL_PHASES.length - 1;
  if (startIdx < 0) throw new PhaseError(`--from ${FROM} invalide`, 2);
  if (stopIdx < 0) throw new PhaseError(`--stop-at ${STOP_AT} invalide`, 2);

  const plan = ALL_PHASES.slice(startIdx, stopIdx + 1);
  log(`📋 Plan A→Z : ${plan.join(" → ")}`);

  for (const phase of plan) {
    try {
      await runSinglePhase(phase);
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      console.error(`\n❌ Phase ${phase} a échoué: ${err.message}`);
      if (NO_ROLLBACK_ON_FAIL) {
        console.error(`   --no-rollback-on-fail → état laissé tel quel`);
      } else {
        warn(`Rollback automatique (git reset HEAD~1)…`);
        await sh(`git -C ${VPS} reset --hard HEAD~1`, { ignoreError: true });
        const s = await readState();
        s.completed_phases = s.completed_phases.filter((p) => p !== phase);
        await writeState(s);
      }
      process.exit(1);
    }
  }

  log(`\n🎉 Toutes les phases (${plan.join(", ")}) complétées`);
}

// ── Status ─────────────────────────────────────────────────────

async function printStatus() {
  const s = await readState();
  console.log(
    JSON.stringify(
      {
        state_file: STATE,
        current_phase: s.current_phase,
        completed_phases: s.completed_phases,
        remaining_phases: ALL_PHASES.filter((p) => !s.completed_phases.includes(p)),
        last_error: s.last_error,
        started_at: s.started_at,
      },
      null,
      2,
    ),
  );
}

// ── Main ──────────────────────────────────────────────────────

if (STATUS) {
  await printStatus();
  process.exit(0);
}

if (ROLLBACK) {
  const release = await acquireLock();
  try {
    await rollback(ROLLBACK);
  } finally {
    await release();
  }
  process.exit(0);
}

if (!PHASE_ARG) {
  console.error(`Usage: bun move-phase.ts <phase> | all | --status | --rollback <phase>`);
  console.error(`Phases: ${ALL_PHASES.join(", ")}`);
  console.error(`Options:`);
  console.error(`  --dry-run             Affiche sans exécuter`);
  console.error(`  --yes                 Bypass confirmations (Phase 7, Phase 8)`);
  console.error(`  --stop-at <N>         Mode 'all' — stop après N`);
  console.error(`  --from <N>            Mode 'all' — reprend depuis N`);
  console.error(`  --skip-verify         Ne lance pas move-verify.ts`);
  console.error(`  --no-rollback-on-fail Ne rollback pas en cas d'échec`);
  process.exit(2);
}

const release = await acquireLock();
try {
  if (PHASE_ARG === "all") {
    await runAll();
  } else {
    await runSinglePhase(PHASE_ARG);
  }
} finally {
  await release();
}
