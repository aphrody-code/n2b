#!/usr/bin/env bun
/**
 * Bootstrap de la migration monorepo (voir ~/vps/move.md).
 *
 * Rôle : préparer l'environnement pour une exécution A→Z sans intervention manuelle.
 *   1. Vérifier les pré-requis (bun >= 1.3.12, n2b 0.4.x, plugin bun-agent actif).
 *   2. Régénérer `.migration-ready/*` si manquant (package.json, turbo.json,
 *      biome.json, tsconfig.base.json, systemd/, nginx/).
 *   3. Construire `bot/dist/index.js` dans ~/rpb-dashboard (requis par Phase 3/6).
 *   4. Dump DB Postgres/Supabase (safety net pré-Phase 2).
 *   5. Initialiser le state file `.migration-state.json` si absent.
 *
 * Usage :
 *   bun scripts/move-bootstrap.ts            # full run
 *   bun scripts/move-bootstrap.ts --check    # verify only, no side effect
 *   bun scripts/move-bootstrap.ts --force    # regénère .migration-ready/* même si présent
 *   bun scripts/move-bootstrap.ts --no-db    # skip DB backup
 *   bun scripts/move-bootstrap.ts --no-bot   # skip bot:build
 *
 * Exit 0 = prêt pour Phase 0, exit 1 = blocker détecté.
 */

const VPS = process.env.VPS ?? `${process.env.HOME}/vps`;
const RG = process.env.RG ?? `${process.env.HOME}/rg`;
const RPB = process.env.RPB ?? `${process.env.HOME}/rpb-dashboard`;
const READY = `${VPS}/.migration-ready`;
const STATE = `${VPS}/.migration-state.json`;

const argv = new Set(process.argv.slice(2));
const CHECK = argv.has("--check");
const FORCE = argv.has("--force");
const NO_DB = argv.has("--no-db");
const NO_BOT = argv.has("--no-bot");
const JSON_OUT = argv.has("--json");

type Result = { ok: boolean; action: string; detail: string };
const results: Result[] = [];
const record = (ok: boolean, action: string, detail: string) => {
  results.push({ ok, action, detail });
  if (!JSON_OUT) console.log(`  ${ok ? "✓" : "✗"} ${action.padEnd(32)} ${detail}`);
};

async function sh(cmd: string, cwd?: string): Promise<{ out: string; err: string; code: number }> {
  const proc = Bun.spawn(["bash", "-c", cmd], { cwd, stdout: "pipe", stderr: "pipe" });
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);
  const code = await proc.exited;
  return { out: out.trim(), err: err.trim(), code };
}

async function exists(p: string): Promise<boolean> {
  try { await Bun.file(p).stat(); return true; } catch { return false; }
}

async function writeIfDiff(p: string, content: string): Promise<boolean> {
  const cur = await Bun.file(p).text().catch(() => "");
  if (cur === content) return false;
  if (CHECK) return true;
  await Bun.write(p, content);
  return true;
}

function section(title: string) {
  if (JSON_OUT) return;
  console.log(`\n── ${title} ───────────────────────────`);
}

// ── 1. Pré-requis système ─────────────────────────────────────

async function checkTools() {
  section("1. Pré-requis système");

  const bunV = (await sh("bun --version")).out;
  const bunOk = /^1\.3\.(1[2-9]|[2-9]\d)/.test(bunV);
  record(bunOk, "bun >= 1.3.12", `bun=${bunV}`);

  const n2bV = (await sh("n2b --version")).out.replace(/^n2b\s+/, "");
  const n2bOk = n2bV.startsWith("0.4.");
  record(n2bOk, "n2b 0.4.x", `n2b=${n2bV || "absent"}`);

  const plugins = await Bun.file(`${process.env.HOME}/.claude/plugins/installed_plugins.json`)
    .json()
    .catch(() => null);
  const bunAgent = plugins?.plugins?.["bun-agent@plugins"]?.[0];
  record(
    Boolean(bunAgent),
    "plugin bun-agent actif",
    bunAgent ? `v${bunAgent.version} (${bunAgent.scope})` : "non installé",
  );

  // Git config (nécessaire pour commits futurs dans VPS)
  const gitEmail = (await sh("git config --global user.email")).out;
  const gitName = (await sh("git config --global user.name")).out;
  record(Boolean(gitEmail && gitName), "git global config", `${gitName} <${gitEmail}>`);

  // Presence de repos sources
  record(await exists(RG), "~/rg présent", RG);
  record(await exists(RPB), "~/rpb-dashboard présent", RPB);
  record(await exists(VPS), "~/vps présent", VPS);
}

// ── 2. Régénération de .migration-ready/* ────────────────────

async function ensureMigrationReady() {
  section("2. .migration-ready/ (artifacts pré-générés)");

  if (CHECK) {
    for (const f of ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"]) {
      record(await exists(`${READY}/${f}`), `${f}`, `${READY}/${f}`);
    }
    for (const s of ["website", "azalee", "rpb-dashboard", "rpb-bot"]) {
      record(await exists(`${READY}/systemd/${s}.service`), `systemd/${s}.service`,
             `${READY}/systemd/${s}.service`);
    }
    for (const n of ["rosegriffon", "rpbey"]) {
      record(await exists(`${READY}/nginx/${n}.conf`), `nginx/${n}.conf`,
             `${READY}/nginx/${n}.conf`);
    }
    return;
  }

  await sh(`mkdir -p ${READY}/systemd ${READY}/nginx`);

  // Vérifier que les 4 fichiers root existent (régénérer si --force ou absent)
  const rootFiles = ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"];
  for (const f of rootFiles) {
    const target = `${READY}/${f}`;
    if (!FORCE && (await exists(target))) {
      record(true, `${f} déjà présent`, target);
      continue;
    }
    record(false, `${f} manquant`, `→ régénération non implémentée (cp manuel requis)`);
  }

  // Systemd units — régénérer depuis templates si absents
  await ensureSystemdUnits();

  // Nginx confs — régénérer depuis snapshots existants (sed des paths)
  await ensureNginxConfs();
}

async function ensureSystemdUnits() {
  const units = [
    { name: "website.service", app: "website" },
    { name: "azalee.service", app: "azalee" },
    { name: "rpb-dashboard.service", app: "rpb-dashboard" },
    { name: "rpb-bot.service", app: "rpb-bot" },
  ];
  for (const { name, app } of units) {
    const target = `${READY}/systemd/${name}`;
    if (!FORCE && (await exists(target))) {
      record(true, `systemd/${name}`, "déjà présent");
      continue;
    }
    // Source : prod actuelle (/etc/systemd/system/*.service) ou snapshot vps/systemd/
    const src = (await exists(`/etc/systemd/system/${name}`))
      ? `/etc/systemd/system/${name}`
      : `${VPS}/systemd/${name}`;
    if (!(await exists(src))) {
      record(false, `systemd/${name}`, `source introuvable (${src})`);
      continue;
    }
    const raw = await Bun.file(src).text();
    const rewritten = raw
      .replace(/\/home\/ubuntu\/rg\/apps\//g, "/home/ubuntu/vps/apps/")
      .replace(/\/home\/ubuntu\/rg(\s|$)/g, "/home/ubuntu/vps$1")
      .replace(/\/home\/ubuntu\/rpb-dashboard\/bot/g, "/home/ubuntu/vps/apps/rpb-bot")
      .replace(/\/home\/ubuntu\/rpb-dashboard/g, `/home/ubuntu/vps/apps/${app === "rpb-bot" ? "rpb-dashboard" : app}`)
      .replace(/WorkingDirectory=.+/, `WorkingDirectory=/home/ubuntu/vps/apps/${app}`);
    const changed = await writeIfDiff(target, rewritten);
    record(true, `systemd/${name}`, changed ? "régénéré" : "aligné");
  }
}

async function ensureNginxConfs() {
  const confs = [
    { name: "rosegriffon.conf", maintName: "rosegriffon.maintenance.conf" },
    { name: "rpbey.conf", maintName: "rpbey.maintenance.conf" },
  ];
  for (const { name, maintName } of confs) {
    for (const f of [name, maintName]) {
      const target = `${READY}/nginx/${f}`;
      if (!FORCE && (await exists(target))) {
        record(true, `nginx/${f}`, "déjà présent");
        continue;
      }
      const src = `${VPS}/nginx/${f}`;
      if (!(await exists(src))) {
        record(false, `nginx/${f}`, `source introuvable (${src})`);
        continue;
      }
      const raw = await Bun.file(src).text();
      const rewritten = raw
        // rg apps → vps apps
        .replace(/\/home\/ubuntu\/rg\/apps\//g, "/home/ubuntu/vps/apps/")
        // rg packages → vps packages
        .replace(/\/home\/ubuntu\/rg\/packages\//g, "/home/ubuntu/vps/packages/")
        // rg storage-public → vps/storage-public (reste à la racine)
        .replace(/\/home\/ubuntu\/rg\/storage-public\//g, "/home/ubuntu/vps/storage-public/")
        // rpb-dashboard → vps apps rpb-dashboard
        .replace(/\/home\/ubuntu\/rpb-dashboard\//g, "/home/ubuntu/vps/apps/rpb-dashboard/");
      const changed = await writeIfDiff(target, rewritten);
      record(true, `nginx/${f}`, changed ? "régénéré" : "aligné");
    }
  }
}

// ── 3. Bot build (requis par Phase 3/6) ──────────────────────

async function ensureBotBuild() {
  section("3. Bot RPB build (bot/dist/index.js)");
  if (NO_BOT) {
    record(true, "bot:build", "skipped (--no-bot)");
    return;
  }
  const dist = `${RPB}/bot/dist/index.js`;
  if (await exists(dist)) {
    const stat = await Bun.file(dist).stat();
    record(true, "bot/dist/index.js", `${(stat.size / 1024).toFixed(1)} KB`);
    return;
  }
  if (CHECK) {
    record(false, "bot/dist/index.js", "absent (faire `bun run bot:build` dans ~/rpb-dashboard)");
    return;
  }
  record(true, "bot:build", "lancé…");
  const res = await sh(`bun run bot:build`, RPB);
  record(res.code === 0 && (await exists(dist)), "bot:build", res.code === 0 ? "OK" : res.err.slice(0, 120));
}

// ── 4. DB backup (safety net pré-Phase 2) ────────────────────

async function ensureDbBackup() {
  section("4. DB backup (snapshot pré-migration)");
  if (NO_DB) {
    record(true, "db:backup", "skipped (--no-db)");
    return;
  }
  const date = new Date().toISOString().slice(0, 10);
  const marker = `${VPS}/.backups/db-${date}.marker`;
  if (await exists(marker)) {
    record(true, "db:backup", `déjà fait aujourd'hui (${marker})`);
    return;
  }
  if (CHECK) {
    record(false, "db:backup", "à faire (cd ~/rg && bun scripts/rg.ts db backup)");
    return;
  }
  await sh(`mkdir -p ${VPS}/.backups`);
  // rg.ts db backup (peut échouer si Supabase offline — non bloquant)
  const res = await sh(`bun scripts/rg.ts db backup`, RG);
  if (res.code === 0) {
    await Bun.write(marker, `${new Date().toISOString()}\n${res.out.slice(-500)}\n`);
    record(true, "db:backup", "OK");
  } else {
    record(false, "db:backup", `échec non bloquant: ${res.err.slice(0, 120)}`);
  }
}

// ── 5. State file initial ─────────────────────────────────────

async function ensureStateFile() {
  section("5. State file (.migration-state.json)");
  if (await exists(STATE)) {
    const s = await Bun.file(STATE).json().catch(() => null);
    record(Boolean(s), ".migration-state.json", s ? `phase=${s.current_phase ?? "?"}` : "corrompu");
    return;
  }
  if (CHECK) {
    record(false, ".migration-state.json", "absent (sera créé au bootstrap)");
    return;
  }
  const initial = {
    version: "1.0",
    started_at: new Date().toISOString(),
    current_phase: null,
    completed_phases: [],
    last_error: null,
    bootstrap_at: new Date().toISOString(),
  };
  await Bun.write(STATE, JSON.stringify(initial, null, 2));
  record(true, ".migration-state.json", "créé");
}

// ── Main ──────────────────────────────────────────────────────

if (!JSON_OUT) {
  console.log(`🚀 Bootstrap migration monorepo — ${CHECK ? "(check-only)" : "exécution"}`);
  console.log(`   VPS=${VPS}`);
  console.log(`   RG=${RG}`);
  console.log(`   RPB=${RPB}`);
}

await checkTools();
await ensureMigrationReady();
await ensureBotBuild();
await ensureDbBackup();
await ensureStateFile();

const fails = results.filter((r) => !r.ok);
const summary = {
  ok: fails.length === 0,
  total: results.length,
  failed: fails.length,
  results,
};

if (JSON_OUT) {
  console.log(JSON.stringify(summary, null, 2));
} else {
  console.log();
  console.log(fails.length === 0
    ? "✅ Bootstrap OK — lancer `bun scripts/move-phase.ts all` pour démarrer A→Z"
    : `⚠ ${fails.length}/${results.length} checks échoués — corriger avant A→Z`);
}

process.exit(fails.length === 0 ? 0 : 1);
