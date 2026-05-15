#!/usr/bin/env bun
/**
 * Preflight pour la migration monorepo (voir ~/vps/move.md).
 *
 * Détermine :
 *  - quelle phase est prête à lancer (ou en cours selon le state file)
 *  - les blockers qui empêchent l'avancement
 *  - l'état de tous les pré-requis (outils, repos, artifacts)
 *
 * Usage :
 *   bun scripts/move-preflight.ts            # check + résumé lisible
 *   bun scripts/move-preflight.ts --json     # machine-readable
 *   bun scripts/move-preflight.ts --strict   # exit 1 si bloqué
 *   bun scripts/move-preflight.ts --next     # print uniquement la phase suivante
 *
 * Exit :
 *   0 = tout OK, prêt à avancer
 *   1 = au moins un check critique échoué (avec --strict)
 *   2 = erreur interne
 */

const VPS = process.env.VPS ?? `${process.env.HOME}/vps`;
const RG = process.env.RG ?? `${process.env.HOME}/rg`;
const RPB = process.env.RPB ?? `${process.env.HOME}/rpb-dashboard`;
const READY = `${VPS}/.migration-ready`;
const STATE = `${VPS}/.migration-state.json`;

const args = new Set(process.argv.slice(2));
const JSON_OUT = args.has("--json");
const STRICT = args.has("--strict");
const NEXT_ONLY = args.has("--next");

type Severity = "info" | "warn" | "error";
type Check = { name: string; ok: boolean; severity: Severity; detail: string; phase_tag?: string };
const checks: Check[] = [];
const add = (name: string, ok: boolean, detail: string, severity: Severity = "error", phase_tag?: string) =>
  checks.push({ name, ok, severity, detail, phase_tag });

async function sh(cmd: string, cwd?: string): Promise<{ out: string; code: number }> {
  const proc = Bun.spawn(["bash", "-c", cmd], { cwd, stdout: "pipe", stderr: "pipe" });
  const out = await new Response(proc.stdout).text();
  const code = await proc.exited;
  return { out: out.trim(), code };
}

const exists = async (p: string) => {
  try { await Bun.file(p).stat(); return true; } catch { return false; }
};

const isGitRepo = async (dir: string) => (await sh(`git -C ${dir} rev-parse --is-inside-work-tree`)).code === 0;

const gitDirtyCount = async (dir: string) => {
  const r = await sh(`git -C ${dir} status --porcelain 2>/dev/null | wc -l`);
  return Number(r.out || 0);
};

const readJson = async (p: string): Promise<any> => {
  try { return await Bun.file(p).json(); } catch { return null; }
};

// ── Checks ─────────────────────────────────────────────────

if (!JSON_OUT && !NEXT_ONLY) console.error("🔍 Preflight — migration monorepo");

// 1. Paths et repos
for (const [name, path] of [["VPS", VPS], ["RG", RG], ["RPB", RPB]] as const) {
  add(`dir:${name}`, await exists(path), path, "error", "pre");
}

const rgIsGit = await isGitRepo(RG);
const rpbIsGit = await isGitRepo(RPB);
const vpsIsGit = await isGitRepo(VPS);
add("git:rg:init", rgIsGit, `${RG} git repo`, "error", "pre");
add("git:rpb:init", rpbIsGit, `${RPB} git repo`, "error", "pre");
add("git:vps:init", vpsIsGit, `${VPS} git repo (requis Phase 2+)`,
    vpsIsGit ? "info" : "warn", "0.5");

const rgDirty = rgIsGit ? await gitDirtyCount(RG) : 0;
const rpbDirty = rpbIsGit ? await gitDirtyCount(RPB) : 0;
add("git:rg:clean", rgDirty === 0, `${rgDirty} fichiers dirty`, "warn", "0");
add("git:rpb:clean", rpbDirty === 0, `${rpbDirty} fichiers dirty`, "warn", "0");

// 2. Outils
const bunVersion = (await sh("bun --version")).out;
add("tool:bun:>=1.3.12",
    /^1\.3\.(1[2-9]|[2-9]\d)/.test(bunVersion) || /^1\.[4-9]/.test(bunVersion),
    `bun=${bunVersion}`, "error", "pre");

const n2bVersion = (await sh("n2b --version")).out.replace(/^n2b\s+/, "");
add("tool:n2b:0.4.x", n2bVersion.startsWith("0.4."), `n2b=${n2bVersion || "absent"}`, "warn", "pre");

const plugins = await readJson(`${process.env.HOME}/.claude/plugins/installed_plugins.json`);
const bunAgent = plugins?.plugins?.["bun-agent@plugins"]?.[0];
add("tool:plugin-bun-agent", Boolean(bunAgent),
    bunAgent ? `v${bunAgent.version} (${bunAgent.scope})` : "non installé", "warn", "pre");

// 3. Versions harmonisées
const rgPkg = await readJson(`${RG}/package.json`);
const rpbPkg = await readJson(`${RPB}/package.json`);
const rgPM = rgPkg?.packageManager ?? "?";
const rpbPM = rpbPkg?.packageManager ?? "?";
add("version:pm:sync", rgPM === rpbPM,
    `rg=${rgPM} rpb=${rpbPM} sys=bun@${bunVersion}`, "info", "pre");

const nextRg = rgPkg?.catalog?.next ?? "?";
const nextRpb = rpbPkg?.dependencies?.next ?? "?";
add("version:next:align", nextRg === nextRpb,
    `rg.catalog=${nextRg} rpb.deps=${nextRpb}`, "info", "pre");

// 4. Structure apps/packages sources
const rgApps = (await sh(`ls ${RG}/apps/ 2>/dev/null`)).out;
const rpbPackages = (await sh(`ls ${RPB}/packages/ 2>/dev/null`)).out;
add("apps:rg:expected", /azalee/.test(rgApps) && /website/.test(rgApps),
    `${RG}/apps = [${rgApps.split("\n").join(", ")}]`, "error", "pre");
add("apps:rg:no-achillea", !/achillea/.test(rgApps), "achillea retiré", "info", "pre");
add("packages:rg",
    /config-ts/.test((await sh(`ls ${RG}/packages/`)).out),
    `${RG}/packages`, "error", "pre");
add("packages:rpb",
    /rppb-api/.test(rpbPackages) && /shared/.test(rpbPackages),
    `${RPB}/packages = [${rpbPackages.split("\n").join(", ")}]`, "error", "pre");

// 5. .migration-ready artifacts
for (const f of ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"]) {
  add(`ready:${f}`, await exists(`${READY}/${f}`), `${READY}/${f}`, "error", "1");
}
for (const s of ["website", "azalee", "rpb-dashboard", "rpb-bot"]) {
  add(`ready:systemd:${s}`, await exists(`${READY}/systemd/${s}.service`),
      `${READY}/systemd/${s}.service`, "warn", "5");
}
for (const n of ["rosegriffon", "rpbey"]) {
  add(`ready:nginx:${n}`,
      await exists(`${READY}/nginx/${n}.conf`) || await exists(`${VPS}/infra/nginx/${n}.conf`) || await exists(`${VPS}/nginx/${n}.conf`),
      `nginx/${n}.conf disponible`, "warn", "5");
}

// 6. Bot dist (Phase 3/6 requirement)
const botDistExists = await exists(`${RPB}/bot/dist/index.js`);
add("rpb:bot:dist", botDistExists,
    botDistExists ? "bot/dist/index.js OK" : `rebuild: cd ${RPB} && bun run bot:build`,
    "warn", "3");

// 7. Prisma migrations
const prismaMig = Number((await sh(
  `ls ${RPB}/prisma/migrations/ 2>/dev/null | grep -v migration_lock | wc -l`,
)).out);
add("prisma:migrations", prismaMig === 4,
    `${prismaMig} migrations (expected 4)`, "info", "3");

// 8. vps contenu pré-consolidation
for (const d of ["scripts", "agents", "docs", "rust", "nginx", "systemd"]) {
  add(`vps:${d}`, await exists(`${VPS}/${d}`) || await exists(`${VPS}/infra/${d}`),
      `${VPS}/${d}/ (ou infra/)`, "warn", "pre");
}

// 9. Backup systemd
const bakDir1 = `${VPS}/systemd/.bak`;
const bakDir2 = `${VPS}/infra/systemd/.bak`;
const bakReady = (await exists(bakDir1)) || (await exists(bakDir2));
add("backup:systemd-dir", bakReady, bakReady ? "dir .bak présent" : "à créer (Phase 0)", "info", "0");

// 10. Maintenance mode
const maintOn = /return 503/.test(
  await Bun.file(`/etc/nginx/conf.d/rosegriffon.conf`).text().catch(() => ""),
);
add("maintenance:active", maintOn,
    maintOn ? "maintenance active" : "ATTENTION: prod live (activer avant Phase 7)",
    maintOn ? "info" : "warn", "pre");

// 11. Submodule bun-agent
const bunAgentHasInnerGit = await exists(`${VPS}/agents/bun-agent/.git`);
const vpsHasGitmodules = await exists(`${VPS}/.gitmodules`);
if (bunAgentHasInnerGit && !vpsIsGit) {
  add("submodule:bun-agent", true, "sera converti en submodule Phase 0.5", "info", "0.5");
} else if (bunAgentHasInnerGit && vpsIsGit) {
  add("submodule:bun-agent",
      vpsHasGitmodules && /agents\/bun-agent/.test(
        await Bun.file(`${VPS}/.gitmodules`).text().catch(() => "")),
      vpsHasGitmodules ? "déclaré" : "gitlink orphelin — à réparer", "warn", "0.5");
}

// 12. State file
const state = await readJson(STATE);
const completedPhases: string[] = state?.completed_phases ?? [];
add("state:file", Boolean(state), state ? `state v${state.version}` : "absent (sera créé par bootstrap)",
    "info", "pre");

// ── Détermine prochaine phase ─────────────────────────────

const ALL_PHASES = ["0", "0.5", "1", "2", "3", "4", "5", "6", "7", "8"];

const hasTurbo = await exists(`${VPS}/turbo.json`);
const hasAppsWebsite = await exists(`${VPS}/apps/website`);
const hasAppsRpb = await exists(`${VPS}/apps/rpb-dashboard`);
const hasCatalogInApps = await (async () => {
  const pkg = await readJson(`${VPS}/apps/website/package.json`);
  return pkg?.dependencies?.react === "catalog:";
})();
const hasInfraSystemd = await exists(`${VPS}/infra/systemd/website.service`);
const botDistInMonorepo = await exists(`${VPS}/apps/rpb-bot/dist/index.js`);
const rgOldExists = await exists(`${RG}.old`);

const nextPhase = (() => {
  // Prioriser le state file si présent
  if (state?.last_error?.phase) {
    return { phase: state.last_error.phase, reason: `last_error: ${state.last_error.message}` };
  }
  for (const p of ALL_PHASES) {
    if (completedPhases.includes(p)) continue;
    const checkFunc: Record<string, () => boolean> = {
      "0": () => rgDirty > 0 || rpbDirty > 0,
      "0.5": () => !vpsIsGit,
      "1": () => !hasTurbo,
      "2": () => !hasAppsWebsite,
      "3": () => !hasAppsRpb,
      "4": () => !hasCatalogInApps,
      "5": () => !hasInfraSystemd,
      "6": () => !botDistInMonorepo,
      "7": () => false, // Toujours demander explicitement
      "8": () => !rgOldExists,
    };
    const needed = checkFunc[p]?.();
    if (needed) return { phase: p, reason: `précondition: ${p} pas encore exécutée` };
  }
  return { phase: null, reason: "toutes les phases sont complètes" };
})();

const currentPhase = (() => {
  if (!vpsIsGit) return "pre-0.5";
  if (rgDirty > 0 || rpbDirty > 0) return "pre-0";
  if (!hasTurbo) return "0.5";
  if (!hasAppsWebsite) return "1";
  if (!hasAppsRpb) return "2";
  if (!hasCatalogInApps) return "3";
  if (!hasInfraSystemd) return "4";
  if (!botDistInMonorepo) return "5";
  if (!rgOldExists) return "7-ready";
  return "complete";
})();

// ── Blockers ──────────────────────────────────────────────

const criticalFails = checks.filter((c) => !c.ok && c.severity === "error");
const warns = checks.filter((c) => !c.ok && c.severity === "warn");

const blocker = criticalFails.length > 0
  ? `${criticalFails.length} check(s) critique(s) échoué: ${criticalFails.map((c) => c.name).slice(0, 3).join(", ")}`
  : null;

const summary = {
  ok: criticalFails.length === 0,
  current_phase: currentPhase,
  next_phase: nextPhase.phase,
  next_phase_reason: nextPhase.reason,
  blocker,
  completed_phases: completedPhases,
  critical_fails: criticalFails.length,
  warnings: warns.length,
  total_checks: checks.length,
  checks,
};

// ── Output ─────────────────────────────────────────────────

if (NEXT_ONLY) {
  console.log(nextPhase.phase ?? "complete");
  process.exit(criticalFails.length === 0 ? 0 : 1);
}

if (JSON_OUT) {
  console.log(JSON.stringify(summary, null, 2));
} else {
  console.log();
  // Grouper par phase_tag
  const byPhase: Record<string, Check[]> = {};
  for (const c of checks) {
    const k = c.phase_tag ?? "misc";
    (byPhase[k] ??= []).push(c);
  }
  const phaseOrder = ["pre", "0", "0.5", "1", "2", "3", "4", "5", "6", "7", "8", "misc"];
  for (const k of phaseOrder) {
    if (!byPhase[k]) continue;
    console.log(`  [${k}]`);
    for (const c of byPhase[k]) {
      const icon = c.ok ? "✓" : c.severity === "error" ? "✗" : "⚠";
      console.log(`    ${icon} ${c.name.padEnd(28)} ${c.detail}`);
    }
  }
  console.log();
  console.log(`Phase courante  : ${currentPhase}`);
  console.log(`Prochaine phase : ${nextPhase.phase ?? "—"} (${nextPhase.reason})`);
  console.log(`Complétées      : ${completedPhases.length ? completedPhases.join(", ") : "aucune"}`);
  if (blocker) console.log(`Blocker         : ${blocker}`);
  console.log();
  if (criticalFails.length === 0 && warns.length === 0) {
    console.log("✅ Prêt — lancer `bun scripts/move-phase.ts all` pour A→Z");
  } else if (criticalFails.length === 0) {
    console.log(`✅ Prêt avec ${warns.length} warning(s) non bloquant(s)`);
  } else {
    console.log(`❌ ${criticalFails.length} bloquant(s) critique(s) — corriger avant d'avancer`);
  }
}

if (STRICT && criticalFails.length > 0) process.exit(1);
process.exit(0);
