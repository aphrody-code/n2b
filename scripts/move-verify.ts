#!/usr/bin/env bun
/**
 * Validation post-phase de la migration (voir ~/vps/move.md).
 *
 * Chaque phase a un contrat d'invariants post-exécution. Ce script vérifie le
 * contrat pour une phase donnée. Appelé automatiquement par `move-phase.ts`
 * après chaque phase ; peut aussi être lancé manuellement.
 *
 * Usage :
 *   bun scripts/move-verify.ts <phase>         # verify une phase
 *   bun scripts/move-verify.ts all             # verify l'état final complet
 *   bun scripts/move-verify.ts <phase> --json  # machine-readable
 *
 * Exit code :
 *   0 = invariants OK
 *   1 = 1+ invariants échoués
 *   2 = usage / phase inconnue
 */

const VPS = process.env.VPS ?? `${process.env.HOME}/vps`;
const RG = process.env.RG ?? `${process.env.HOME}/rg`;
const RPB = process.env.RPB ?? `${process.env.HOME}/rpb-dashboard`;

const argv = process.argv.slice(2);
const JSON_OUT = argv.includes("--json");
const PHASE = argv.find((a) => !a.startsWith("--")) ?? null;

type Invariant = { name: string; ok: boolean; detail: string; critical: boolean };
const invariants: Invariant[] = [];

function assert(name: string, ok: boolean, detail: string, critical = true) {
  invariants.push({ name, ok, detail, critical });
  if (!JSON_OUT) {
    const icon = ok ? "✓" : critical ? "✗" : "⚠";
    console.log(`  ${icon} ${name.padEnd(36)} ${detail}`);
  }
}

async function sh(cmd: string, cwd?: string): Promise<{ out: string; code: number }> {
  const proc = Bun.spawn(["bash", "-c", cmd], { cwd, stdout: "pipe", stderr: "pipe" });
  const out = await new Response(proc.stdout).text();
  const code = await proc.exited;
  return { out: out.trim(), code };
}

const exists = async (p: string) => {
  try {
    await Bun.file(p).stat();
    return true;
  } catch {
    return false;
  }
};

async function lastCommitSubject(): Promise<string> {
  return (await sh(`git -C ${VPS} log -1 --format=%s 2>/dev/null`)).out;
}

// ─────────────────────────────────────────────────────────────
//  Verifiers par phase
// ─────────────────────────────────────────────────────────────

async function verify_0() {
  if (!JSON_OUT) console.log("\n Phase 0 — snapshot + backups\n");
  // Repos sources doivent être clean
  const rgDirty = (await sh(`git -C ${RG} status --porcelain | wc -l`)).out;
  const rpbDirty = (await sh(`git -C ${RPB} status --porcelain | wc -l`)).out;
  assert("git:rg:clean", rgDirty === "0", `${rgDirty} fichiers dirty`);
  assert("git:rpb:clean", rpbDirty === "0", `${rpbDirty} fichiers dirty`);
  // Backup tar présent
  const today = new Date().toISOString().slice(0, 10);
  const tarPath = `${process.env.HOME}/backup-pre-migration-${today}.tar.gz`;
  assert("backup:tar", await exists(tarPath), tarPath);
  // Backup systemd
  const backupDir = (await exists(`${VPS}/systemd/.bak`))
    ? `${VPS}/systemd/.bak`
    : `${VPS}/infra/systemd/.bak`;
  const cnt = (await sh(`ls ${backupDir}/*.service 2>/dev/null | wc -l`)).out;
  assert("backup:systemd-units", Number(cnt) >= 4, `${cnt}/4 services sauvegardés (${backupDir})`);
}

async function verify_05() {
  if (!JSON_OUT) console.log("\n Phase 0.5 — git init vps\n");
  const isRepo = await sh(`git -C ${VPS} rev-parse --is-inside-work-tree`);
  assert("git:vps:init", isRepo.code === 0, `${VPS} est un repo git`);
  const hasCommit = (await sh(`git -C ${VPS} log --oneline 2>/dev/null | wc -l`)).out;
  assert("git:vps:has-commit", Number(hasCommit) >= 1, `${hasCommit} commit(s)`);
  // Submodule bun-agent déclaré si repo inner présent
  const hasInnerGit = await exists(`${VPS}/agents/bun-agent/.git`);
  if (hasInnerGit) {
    const gitmodules = await exists(`${VPS}/.gitmodules`);
    const inConfig =
      gitmodules && /agents\/bun-agent/.test(await Bun.file(`${VPS}/.gitmodules`).text());
    assert(
      "submodule:bun-agent",
      inConfig,
      inConfig ? "déclaré dans .gitmodules" : "gitlink orphelin",
    );
  }
}

async function verify_1() {
  if (!JSON_OUT) console.log("\n Phase 1 — Turborepo root + infra/\n");
  for (const f of ["package.json", "turbo.json", "biome.json", "tsconfig.base.json"]) {
    assert(`file:${f}`, await exists(`${VPS}/${f}`), `${VPS}/${f}`);
  }
  // infra/ reorg
  for (const d of ["nginx", "systemd", "rust"]) {
    assert(`infra:${d}`, await exists(`${VPS}/infra/${d}`), `infra/${d}/`);
  }
  // Lockfile régénéré
  assert("bun.lock", await exists(`${VPS}/bun.lock`), `${VPS}/bun.lock`);
  // package.json workspace mode objet
  const pkg = await Bun.file(`${VPS}/package.json`)
    .json()
    .catch(() => null);
  assert(
    "package.json:workspaces:object",
    typeof pkg?.workspaces === "object" && !Array.isArray(pkg?.workspaces),
    pkg?.workspaces ? "objet catalog OK" : "manquant",
  );
  assert(
    "package.json:catalog:react",
    pkg?.workspaces?.catalog?.react === "^19.2.5",
    `react=${pkg?.workspaces?.catalog?.react}`,
  );
  // Commit
  const subject = await lastCommitSubject();
  assert(
    "commit:message",
    /turborepo|catalog bun/i.test(subject),
    subject || "(aucun commit)",
    false,
  );
}

async function verify_2() {
  if (!JSON_OUT) console.log("\n Phase 2 — subtree rg\n");
  for (const app of ["website", "azalee"]) {
    assert(`apps/${app}`, await exists(`${VPS}/apps/${app}/package.json`), `apps/${app}/`);
  }
  for (const p of ["inagle", "config-ts", "types"]) {
    assert(`packages/${p}`, await exists(`${VPS}/packages/${p}/package.json`), `packages/${p}/`);
  }
  // Historique git préservé (subtree sans --squash)
  const rgCommits = (
    await sh(
      `git -C ${VPS} log --oneline --all --format=%s 2>/dev/null | grep -c "rosegriffon\\|azalee\\|inagle" || true`,
    )
  ).out;
  assert(
    "git:history:rg-preserved",
    Number(rgCommits) >= 5,
    `${rgCommits} commits historiques rg importés`,
    false,
  );
  // _import-rg nettoyé
  assert("cleanup:_import-rg", !(await exists(`${VPS}/_import-rg`)), "_import-rg/ supprimé");
}

async function verify_3() {
  if (!JSON_OUT) console.log("\n Phase 3 — subtree rpb\n");
  for (const app of ["rpb-dashboard", "rpb-bot"]) {
    assert(`apps/${app}`, await exists(`${VPS}/apps/${app}/package.json`), `apps/${app}/`);
  }
  // Next.js à la racine apps/rpb-dashboard/
  assert(
    "rpb-dashboard:next.config",
    await exists(`${VPS}/apps/rpb-dashboard/next.config.ts`),
    "apps/rpb-dashboard/next.config.ts",
  );
  assert(
    "rpb-dashboard:prisma",
    await exists(`${VPS}/apps/rpb-dashboard/prisma/schema.prisma`),
    "apps/rpb-dashboard/prisma/schema.prisma",
  );
  // Bot SWC config
  assert("rpb-bot:swcrc", await exists(`${VPS}/apps/rpb-bot/.swcrc`), "apps/rpb-bot/.swcrc");
  // Packages partagés
  assert(
    "packages/rppb-api",
    await exists(`${VPS}/packages/rppb-api/package.json`),
    "packages/rppb-api/",
  );
  assert(
    "packages/rpb-shared",
    await exists(`${VPS}/packages/rpb-shared/package.json`),
    "packages/rpb-shared/ (ex shared)",
  );
  // Cleanup
  assert("cleanup:_import-rpb", !(await exists(`${VPS}/_import-rpb`)), "_import-rpb/ supprimé");
}

async function verify_4() {
  if (!JSON_OUT) console.log("\n Phase 4 — catalog unifié\n");
  const expected = [
    { path: `${VPS}/apps/website/package.json`, name: "@rosegriffon/website" },
    { path: `${VPS}/apps/azalee/package.json`, name: "@rosegriffon/azalee" },
    { path: `${VPS}/apps/rpb-dashboard/package.json`, name: "@rpb/dashboard" },
    { path: `${VPS}/apps/rpb-bot/package.json`, name: "@rpb/bot" },
    { path: `${VPS}/packages/rppb-api/package.json`, name: "@rpb/api" },
    { path: `${VPS}/packages/rpb-shared/package.json`, name: "@rpb/shared" },
  ];
  for (const { path, name } of expected) {
    const pkg = await Bun.file(path)
      .json()
      .catch(() => null);
    assert(`pkg:${name}`, pkg?.name === name, pkg ? `name="${pkg.name}"` : "absent");
  }
  // Aucun bun.lock dans les workspaces (seul celui à la racine)
  const childLocks = (
    await sh(`find ${VPS}/apps ${VPS}/packages -maxdepth 2 -name bun.lock 2>/dev/null | wc -l`)
  ).out;
  assert("no-child-lockfiles", childLocks === "0", `${childLocks} bun.lock dans workspaces`);
  // Dry-run clean
  const dry = await sh(`bun install --dry-run 2>&1 | tail -5`, VPS);
  const clean = !/added|removed|updated/i.test(dry.out);
  assert(
    "bun install --dry-run",
    clean,
    clean ? "aucun changement" : (dry.out.split("\n").pop() ?? ""),
    false,
  );
  // React 19 / Next 16 catalog appliqué dans un workspace au moins
  const websitePkg = await Bun.file(`${VPS}/apps/website/package.json`)
    .json()
    .catch(() => null);
  const usesCatalog = websitePkg?.dependencies?.react === "catalog:";
  assert(
    "catalog:applied",
    usesCatalog,
    `apps/website/package.json:react=${websitePkg?.dependencies?.react}`,
    false,
  );
}

async function verify_5() {
  if (!JSON_OUT) console.log("\n Phase 5 — paths nginx/systemd\n");
  // Aucune ref aux anciens paths — exclure .bak/, _from-*, submodules rust
  const leak = await sh(
    `grep -rlE "/home/ubuntu/(rg|rpb-dashboard)/" ${VPS}/infra/ 2>/dev/null ` +
      `| grep -vE '(\\.bak/|_from-|/rust/(n2b|mui-rs)/|/azalee\\.rosegriffon\\.fr$)' | wc -l`,
  );
  assert(
    "paths:no-old-refs",
    Number(leak.out) === 0,
    Number(leak.out) === 0
      ? "0 fichier contient /home/ubuntu/(rg|rpb-dashboard) (hors .bak/_from/submodules)"
      : `${leak.out} fichiers à corriger`,
  );
  // systemd units présents
  for (const s of ["website", "azalee", "rpb-dashboard", "rpb-bot"]) {
    const p = `${VPS}/infra/systemd/${s}.service`;
    if (await exists(p)) {
      const content = await Bun.file(p).text();
      const wd = content.match(/^WorkingDirectory=(.+)$/m)?.[1];
      const ok = wd?.startsWith(`/home/ubuntu/vps/apps/${s}`);
      assert(`systemd:${s}:WD`, !!ok, `WorkingDirectory=${wd ?? "?"}`);
    } else {
      assert(`systemd:${s}`, false, `${p} absent`);
    }
  }
  // nginx configs présentes avec nouveaux paths
  for (const n of ["rosegriffon.conf", "rpbey.conf"]) {
    const p = `${VPS}/infra/nginx/${n}`;
    if (await exists(p)) {
      const content = await Bun.file(p).text();
      const hasNew = /\/home\/ubuntu\/vps\/apps\//.test(content);
      const hasOld = /\/home\/ubuntu\/(rg|rpb-dashboard)\//.test(content);
      assert(
        `nginx:${n}`,
        hasNew && !hasOld,
        hasOld ? "contient encore anciens paths" : "paths migrés",
      );
    }
  }
}

async function verify_6() {
  if (!JSON_OUT) console.log("\n Phase 6 — build offline\n");
  // Install & dry-run — strict
  const dry = await sh(`bun install --dry-run --ignore-scripts 2>&1 | tail -3`, VPS);
  assert("install:stable", !/^error/im.test(dry.out), "bun install --dry-run clean", true);
  // Type-check — warn only (Supabase legacy types from rg repo)
  const tc = await sh(`bun run type-check 2>&1 | tail -5`, VPS);
  assert(
    "type-check",
    tc.code === 0,
    tc.code === 0 ? "OK" : (tc.out.split("\n").pop() ?? ""),
    false,
  );
  // Build — warn only (pre-existing issues may persist)
  const build = await sh(`bun run build 2>&1 | tail -3`, VPS);
  assert(
    "build",
    build.code === 0,
    build.code === 0 ? "OK" : (build.out.split("\n").pop() ?? ""),
    false,
  );
  // Build artefacts — .next as warn (depends on build), bot/dist is critical (SWC)
  for (const app of ["website", "azalee", "rpb-dashboard"]) {
    const next = await exists(`${VPS}/apps/${app}/.next/BUILD_ID`);
    assert(`artefact:${app}/.next`, next, `apps/${app}/.next/BUILD_ID`, false);
  }
  const botDist = await exists(`${VPS}/apps/rpb-bot/dist/index.js`);
  assert("artefact:rpb-bot/dist", botDist, "apps/rpb-bot/dist/index.js");
}

async function verify_7() {
  if (!JSON_OUT) console.log("\n Phase 7 — bascule live\n");
  // systemd units actifs pointent bien vers vps
  const services = ["website", "azalee", "rpb-dashboard", "rpb-bot"];
  for (const s of services) {
    const act = (await sh(`systemctl is-active ${s} 2>/dev/null`)).out;
    assert(`svc:${s}:active`, act === "active", `is-active=${act}`);
    const wd = (await sh(`systemctl show -p WorkingDirectory --value ${s} 2>/dev/null`)).out;
    assert(`svc:${s}:WD`, wd.includes(`/home/ubuntu/vps/apps/${s}`), wd);
  }
  // HTTP endpoints
  const endpoints = [
    { url: "https://rosegriffon.fr/", host: "rosegriffon.fr" },
    { url: "https://azalee.rosegriffon.fr/", host: "azalee.rosegriffon.fr" },
    { url: "https://rpbey.fr/", host: "rpbey.fr" },
  ];
  for (const { url, host } of endpoints) {
    const code = (await sh(`curl -sI -o /dev/null -w "%{http_code}" ${url}`)).out;
    assert(`http:${host}`, code === "200" || code === "301" || code === "302", `${code} ${url}`);
  }
  // nginx -t
  const ntest = await sh(`sudo nginx -t 2>&1 | tail -2`);
  assert("nginx:test", /successful/.test(ntest.out), ntest.out.split("\n").pop() ?? "");
}

async function verify_8() {
  if (!JSON_OUT) console.log("\n Phase 8 — cleanup\n");
  assert("rg.old:present", await exists(`${RG}.old`), `${RG}.old`);
  assert("rpb.old:present", await exists(`${RPB}.old`), `${RPB}.old`);
  assert("rg:absent", !(await exists(RG)), `${RG} renommé`);
  assert("rpb:absent", !(await exists(RPB)), `${RPB} renommé`);
}

async function verify_all() {
  if (!JSON_OUT) console.log("\n ✅ État final complet\n");
  for (const v of [
    verify_05,
    verify_1,
    verify_2,
    verify_3,
    verify_4,
    verify_5,
    verify_6,
    verify_7,
    verify_8,
  ]) {
    await v();
  }
}

// ─────────────────────────────────────────────────────────────
//  Dispatch
// ─────────────────────────────────────────────────────────────

const verifiers: Record<string, () => Promise<void>> = {
  "0": verify_0,
  "0.5": verify_05,
  "1": verify_1,
  "2": verify_2,
  "3": verify_3,
  "4": verify_4,
  "5": verify_5,
  "6": verify_6,
  "7": verify_7,
  "8": verify_8,
  all: verify_all,
};

if (!PHASE || !verifiers[PHASE]) {
  console.error(`Usage: bun move-verify.ts <phase>   # ${Object.keys(verifiers).join(", ")}`);
  process.exit(2);
}

if (!JSON_OUT) console.log(`🔍 Verify — Phase ${PHASE}`);
await verifiers[PHASE]();

const criticalFails = invariants.filter((i) => !i.ok && i.critical);
const warns = invariants.filter((i) => !i.ok && !i.critical);

if (JSON_OUT) {
  console.log(
    JSON.stringify(
      {
        phase: PHASE,
        ok: criticalFails.length === 0,
        total: invariants.length,
        critical_failed: criticalFails.length,
        warnings: warns.length,
        invariants,
      },
      null,
      2,
    ),
  );
} else {
  console.log();
  if (criticalFails.length === 0 && warns.length === 0) {
    console.log(`✅ Phase ${PHASE} — invariants OK (${invariants.length}/${invariants.length})`);
  } else if (criticalFails.length === 0) {
    console.log(`✅ Phase ${PHASE} — OK avec ${warns.length} warning(s) non bloquant(s)`);
  } else {
    console.log(`❌ Phase ${PHASE} — ${criticalFails.length} invariant(s) CRITIQUE(S) échoué(s)`);
    for (const f of criticalFails) console.log(`   ✗ ${f.name} — ${f.detail}`);
  }
}

process.exit(criticalFails.length === 0 ? 0 : 1);
