# node2bun report

- mode : `check`
- racine : `/home/ubuntu/rpb-dashboard`

## `.npmrc`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `npmrc/node-linker` | 'node-linker' (pnpm/yarn) → bunfig.toml : [install].linker = "isolated" | "hoisted" |  |

## `bot/package.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `ecosystem/prisma` | Prisma détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/prisma | `https://bun.sh/guides/ecosystem/prisma` |
| 1:1 | `ecosystem/discord-bot` | discord.js détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/discordjs | `https://bun.sh/guides/ecosystem/discordjs` |
| 1:1 | `ecosystem/zod` | Zod (schema validation) détecté — guide d'intégration Bun : https://zod.dev/ | `https://zod.dev/` |
| 1:1 | `ecosystem/oxlint` | oxlint (linter Rust OXC, ~50× ESLint) détecté — guide d'intégration Bun : https://oxc.rs/docs/guide/usage/linter.html | `https://oxc.rs/docs/guide/usage/linter.html` |

## `bot/prisma.config.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:9 | `imports/bun-native` | remplacer 'dotenv/config' par <auto> — Bun charge .env automatiquement, dotenv inutile | `<auto>` |
| 7:10 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/build-bin.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 29:6 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 8:15 | `api/performance-now` | Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage) |  |
| 33:23 | `api/performance-now` | Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage) |  |

## `bot/scripts/build-oxc.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 15:15 | `api/performance-now` | Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage) |  |
| 42:23 | `api/performance-now` | Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage) |  |

## `bot/scripts/create-missing-roles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 18:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/exchange-manual.js`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 6:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/fetch-message.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 68:14 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/get-twitch-token.js`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/inspect-probot.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/send-social-announcement.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 16:8 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 22:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/scripts/test-bin.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 23:18 | `api/child-process-spawn` | Bun.spawn offre une API plus ergonomique (streams Web, ipc, preload) |  |

## `bot/scripts/test-token.js`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/cron/tasks/DailyStats.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 77:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/cron/tasks/MentionsScan.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 61:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/cron/tasks/SyncRankingRoles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 17:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/cron/tasks/SyncSatrRoles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 11:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/events/AdvancedLogs.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 20:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/events/MutedChannelSync.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/events/ready.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 19:9 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 19:56 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 22:55 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 24:49 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 37:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/generated/prisma/client.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 32:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 16:40 | `api/fileURLToPath` | Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path) |  |

## `bot/src/generated/prisma/internal/class.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 71:50 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 95:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 43:21 | `api/buffer-from-base64` | utiliser atob() / btoa() ou Uint8Array pour du Web-standard |  |

## `bot/src/guards/NotBlacklisted.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:3 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/guards/OwnerOnly.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 9:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/index.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 30:25 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 84:8 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 88:25 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 125:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 129:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 129:47 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/api-server.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 76:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 101:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 156:29 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/bot.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 14:14 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 14:38 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/challonge.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 511:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 512:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 513:20 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/command-generator.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 37:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/logger.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:9 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 17:9 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/prisma.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 2:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 11:30 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 34:28 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/redis.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:3 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/singleton-guard.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 47:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 47:41 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/lib/twitch-bot.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 25:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 26:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `bot/src/tests/RankingGroup.test.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:55 | `imports/bun-native` | remplacer 'vitest' par bun:test — bun:test offre les mêmes fonctionnalités que vitest | `bun:test` |

## `bot/src/tests/mocks.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:21 | `imports/bun-native` | remplacer 'vitest' par bun:test — bun:test offre les mêmes fonctionnalités que vitest | `bun:test` |

## `bot/src/tests/setup.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 2:21 | `imports/bun-native` | remplacer 'vitest' par bun:test — bun:test offre les mêmes fonctionnalités que vitest | `bun:test` |

## `bot/tsconfig.build.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `tsconfig/module-detection` | compilerOptions.moduleDetection absent — 'force' garantit que chaque fichier est ESM (évite les .js traités comme CJS) | `"force"` |

## `bot/tsconfig.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `tsconfig/module-detection` | compilerOptions.moduleDetection absent — 'force' garantit que chaque fichier est ESM (évite les .js traités comme CJS) | `"force"` |
| 1:1 | `tsconfig/verbatim-module-syntax` | moduleResolution=bundler + verbatimModuleSyntax=true est le combo recommandé Bun (force `import type` explicite) | `true` |
| 1:1 | `tsconfig/allow-ts-extensions` | Bun résout les extensions .ts nativement — allowImportingTsExtensions=true permet `import './x.ts'` | `true` |

## `eslint.config.mjs`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:35 | `imports/bun-native` | remplacer 'eslint-config-prettier' par @biomejs/biome — plus besoin de désactiver les règles ESLint qui conflictent avec Prettier — Biome unifie | `@biomejs/biome` |

## `next.config.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 48:9 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 62:5 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `package.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `next/build-turbopack` | script "build"="next build" — Next 16 utilise Turbopack par défaut pour `dev` ; pour `build` ajouter `--turbopack` explicitement accélère le build (50-80%) | `next build --turbopack` |
| 1:1 | `ecosystem/emotion` | Emotion (CSS-in-JS, MUI default) détecté — guide d'intégration Bun : https://emotion.sh/ | `https://emotion.sh/` |
| 1:1 | `ecosystem/fontsource` | Fontsource Roboto (self-host) détecté — guide d'intégration Bun : https://fontsource.org/ | `https://fontsource.org/` |
| 1:1 | `ecosystem/mui` | MUI icons détecté — guide d'intégration Bun : https://mui.com/material-ui/material-icons/ | `https://mui.com/material-ui/material-icons/` |
| 1:1 | `ecosystem/mui-x` | MUI X Charts détecté — guide d'intégration Bun : https://mui.com/x/react-charts/ | `https://mui.com/x/react-charts/` |
| 1:1 | `ecosystem/prisma` | Prisma détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/prisma | `https://bun.sh/guides/ecosystem/prisma` |
| 1:1 | `ecosystem/clsx` | clsx (className concat) détecté — guide d'intégration Bun : https://github.com/lukeed/clsx | `https://github.com/lukeed/clsx` |
| 1:1 | `ecosystem/express` | Express détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/express | `https://bun.sh/guides/ecosystem/express` |
| 1:1 | `ecosystem/graphql-yoga` | GraphQL Yoga détecté — guide d'intégration Bun : https://the-guild.dev/graphql/yoga-server/v3/integrations/integration-with-bun | `https://the-guild.dev/graphql/yoga-server/v3/integrations/integration-with-bun` |
| 1:1 | `ecosystem/lucide` | Lucide icons (default shadcn) détecté — guide d'intégration Bun : https://lucide.dev/ | `https://lucide.dev/` |
| 1:1 | `ecosystem/nextjs` | Next.js détecté — guide d'intégration Bun : https://bun.sh/guides/ecosystem/nextjs | `https://bun.sh/guides/ecosystem/nextjs` |
| 1:1 | `ecosystem/react-hook-form` | React Hook Form (forms) détecté — guide d'intégration Bun : https://react-hook-form.com/ | `https://react-hook-form.com/` |
| 1:1 | `ecosystem/sonner` | Sonner (toasts, compat shadcn) détecté — guide d'intégration Bun : https://sonner.emilkowal.ski/ | `https://sonner.emilkowal.ski/` |
| 1:1 | `ecosystem/zod` | Zod (schema validation) détecté — guide d'intégration Bun : https://zod.dev/ | `https://zod.dev/` |
| 1:1 | `ecosystem/biome` | Biome (linter + formatter Rust, ~100× ESLint+Prettier) détecté — guide d'intégration Bun : https://biomejs.dev/ | `https://biomejs.dev/` |
| 1:1 | `ecosystem/swc` | SWC CLI détecté — guide d'intégration Bun : https://swc.rs/ | `https://swc.rs/` |

## `packages/rppb-api/package.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `ecosystem/zod` | Zod (schema validation) détecté — guide d'intégration Bun : https://zod.dev/ | `https://zod.dev/` |

## `packages/rppb-api/tsconfig.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `tsconfig/module-detection` | compilerOptions.moduleDetection absent — 'force' garantit que chaque fichier est ESM (évite les .js traités comme CJS) | `"force"` |
| 1:1 | `tsconfig/verbatim-module-syntax` | moduleResolution=bundler + verbatimModuleSyntax=true est le combo recommandé Bun (force `import type` explicite) | `true` |
| 1:1 | `tsconfig/allow-ts-extensions` | Bun résout les extensions .ts nativement — allowImportingTsExtensions=true permet `import './x.ts'` | `true` |
| 1:1 | `tsconfig/no-emit` | moduleResolution=bundler typiquement couplé à noEmit=true (Bun émet le JS, tsc ne fait que le type-check) | `true` |

## `packages/shared/tsconfig.json`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:1 | `tsconfig/module-detection` | compilerOptions.moduleDetection absent — 'force' garantit que chaque fichier est ESM (évite les .js traités comme CJS) | `"force"` |
| 1:1 | `tsconfig/verbatim-module-syntax` | moduleResolution=bundler + verbatimModuleSyntax=true est le combo recommandé Bun (force `import type` explicite) | `true` |
| 1:1 | `tsconfig/allow-ts-extensions` | Bun résout les extensions .ts nativement — allowImportingTsExtensions=true permet `import './x.ts'` | `true` |
| 1:1 | `tsconfig/no-emit` | moduleResolution=bundler typiquement couplé à noEmit=true (Bun émet le JS, tsc ne fait que le type-check) | `true` |

## `prisma.config.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:9 | `imports/bun-native` | remplacer 'dotenv/config' par <auto> — Bun charge .env automatiquement, dotenv inutile | `<auto>` |

## `prisma/seed-anime.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 5:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `prisma/seed-beyblades.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 13:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `prisma/seed-parts.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 10:23 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 12:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `prisma/seed-products.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 15:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/analyze-ranking.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/analyze-seasons.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/check-40k-role.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/check-character-roles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/check-point-roles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/cleanup-commands.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 13:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/draft-thread.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 33:36 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 34:29 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fast-scrape-bey-library.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 13:6 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |
| 153:13 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/fetch-avatars-api.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:20 | `imports/bun-native` | remplacer 'axios' par <global fetch> — fetch est global dans Bun — pas besoin d'axios | `<global fetch>` |
| 6:20 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fetch-google-sheet.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fetch-wb-history.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 9:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fix-episode-titles-bakuten.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 7:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fix-episode-titles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 10:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 12:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/fix-gacha-duplicates.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 10:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 13:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/generate-bey-manifest.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:18 | `api/path-join-dirname` | dans un ESM Bun, path.join(import.meta.dir, ...) évite __dirname |  |
| 7:21 | `api/path-join-dirname` | dans un ESM Bun, path.join(import.meta.dir, ...) évite __dirname |  |

## `scripts/get-lotte-followers.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 27:36 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 28:29 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/get-user-details.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 25:36 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 26:29 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/import-satr-bladers.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 7:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/import-satr-sheet.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 9:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 11:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/import-wb-bladers.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/link-discord-participants.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 35:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 36:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 37:20 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/link-product-parts.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 44:13 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/list-roles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 6:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/massive-discord-sync.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 3:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 4:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/purge-commands.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:43 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/scrape-anime.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 14:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 436:13 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/scrape-bey-library-full.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 285:11 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/scrape-episode-titles-en.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 10:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 12:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/scrape-episode-titles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 14:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/screenshot-all.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 28:7 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |

## `scripts/screenshot-discord.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 7:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 16:8 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |

## `scripts/screenshot.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 23:8 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |

## `scripts/search-bird.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 30:36 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 31:29 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/seed-card-stats.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 10:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/seed-drop1.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 17:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 20:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/seed-quadstrike-and-titles.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 10:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/send-qr-dm.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:24 | `imports/bun-native` | remplacer 'ws' par WebSocket — WebSocket client/serveur est natif dans Bun | `WebSocket` |
| 45:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 79:25 | `api/crypto-createHash` | Bun.hash / Bun.CryptoHasher est plus rapide (voir runtime/hashing) |  |
| 37:5 | `api/buffer-from-base64` | utiliser atob() / btoa() ou Uint8Array pour du Web-standard |  |

## `scripts/send-validation.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 42:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/subscribe-twitch-local.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 6:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/subscribe-twitch.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 4:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 5:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 6:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/subscribe-youtube.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 1:24 | `imports/bun-native` | remplacer 'undici' par <global fetch> — fetch/WebSocket sont natifs dans Bun — undici inutile | `<global fetch>` |

## `scripts/sync-admins.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:23 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 19:8 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 29:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/sync-bey-library-complete.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 340:7 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/sync-bey-library.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 116:9 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/sync-products.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 71:9 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/sync-staff-db.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:23 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 32:8 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 42:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/sync-wb-ranking.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 7:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/sync-weights-cleaned.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 60:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/sync-weights.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 34:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/sync-youtube-beytube.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 7:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 14:16 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/test-kyoya-card.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 269:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/test-kyoya-gif.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 415:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 387:5 | `api/process-stdout-write` | Bun.stdout.write() est l'équivalent natif Bun de process.stdout.write |  |

## `scripts/tour-video.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 274:7 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |

## `scripts/trailer-pro.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 275:7 | `api/fs-existsSync` | fs.existsSync(path) suivi d'un fs.mkdirSync(path) — chemin probablement un dossier, Bun.file().exists() inadapté (utiliser fs.mkdirSync(path, { recursive: true })) |  |
| 30:25 | `api/performance-now` | Bun.nanoseconds() offre une horloge haute précision (retourne nanosecondes depuis démarrage) |  |

## `scripts/update-anime-images-hd.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 7:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 9:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/update-anime-metadata.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 8:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 10:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/update-card-stats.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 10:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 12:46 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `scripts/update-mudae-panel.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 60:14 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/(admin)/admin/page.tsx`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 269:35 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/(admin)/admin/tournaments/[id]/actions.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 143:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 144:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/(admin)/admin/tournaments/actions.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 22:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/(marketing)/tv/page.tsx`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 21:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/auth/callback/challonge/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 77:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 85:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/auth/challonge/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 20:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/auth/dev-login/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:7 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/auth/magic-link/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 51:23 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/auth/mobile/callback/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 29:20 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 30:24 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/external/v1/leaderboard/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 20:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/api/webhooks/twitch/route.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 30:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/robots.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/app/sitemap.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 68:19 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/generated/prisma/client.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 32:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 16:40 | `api/fileURLToPath` | Bun.fileURLToPath() est équivalent (ou utiliser import.meta.dir/path) |  |

## `src/generated/prisma/internal/class.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 71:50 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 95:48 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 43:21 | `api/buffer-from-base64` | utiliser atob() / btoa() ou Uint8Array pour du Web-standard |  |

## `src/hooks/useBotEvents.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 40:16 | `api/eventsource-new` | EventSource est global dans Bun (Bun.EventSource) — plus besoin de la dep 'eventsource' |  |

## `src/lib/auth-client.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 15:9 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/auth.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 8:3 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 9:3 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 49:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 50:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 65:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 66:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 69:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 70:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/bot-config.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 6:10 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 9:28 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/challonge.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 76:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 77:28 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 100:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 101:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 102:28 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 135:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 136:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 175:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 176:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/prisma.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 2:17 | `imports/bun-native` | remplacer 'pg' par Bun.sql — Bun.sql est un client PostgreSQL natif | `Bun.sql` |
| 7:26 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 25:5 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/seo-utils.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 12:17 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/lib/twitch.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 5:18 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 6:22 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |
| 7:21 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |

## `src/proxy.ts`

| ligne | règle | message | remplacement |
| --- | --- | --- | --- |
| 35:16 | `api/process-env` | Bun.env est un alias plus court de process.env (préférence stylistique) |  |


