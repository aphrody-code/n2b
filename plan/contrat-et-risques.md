# Contrat gelé, registre de risques & séquençage

## 1. Le contrat externe gelé — impact phase par phase

Surfaces consommées par `/home/ubuntu/rpb-dashboard` via subprocess. Filet de sécurité :
`tests/compare-baseline.sh` (12 comparaisons octet-à-octet) +
`crates/n2b-cli/tests/contract.rs` (9 tests).

| Surface | Vérité | Impact du refactor | Phase | Mitigation |
|---|---|---|---|---|
| **Rule IDs** | `n2b-rules/src/*.rs` → `registry/*.toml` | **Inchangés.** Phase 1 = transcription pure. Nouveaux IDs seulement *ajoutés* (Phases 3-6). | 1, 4 | diff `n2b rules --report=json` contre baseline `rules.json` à chaque phase |
| **Format JSON** | `schema/v2.json` | Champ `compat` **ajouté optionnel** — rétro-compatible, `schema_version` reste `2`. `report_card` additif (`--migrate` seulement). | 3, 5 | champ dans `properties` pas `required` ; test rétro-compat dédié |
| **Flags CLI** | `n2b-cli/src/cli/args.rs` | Inchangés. `--migrate --scaffold-polyfills` ajouté (opt-in). `--report=json` enrichi (additif). | 5, 6 | aucun flag retiré ni resignifié |
| **Exit codes** | `n2b-cli/src/commands/scan.rs:52-63` | Un `imports/*` 🔴 passe `warn`→`error` → peut faire passer un exit `1`→`2`. **Changement voulu.** | 3 | documenté CHANGELOG ; baselines régénérées le capturent ; c'est le comportement correct (🔴 doit bloquer) |
| **ABI cdylib v1** | `n2b-native/src/lib.rs` | **Aucun impact.** `n2b-native` est hors graphe, ne touche pas au schéma. | — | aucune |
| **Sortie `--fix`** | baselines | PS4 : `--fix` ne réécrit plus les lignes commentées. **Breaking justifié (fix de bug).** | 0 | CHANGELOG + baselines régénérées + commit explicite |
| **Manifeste `n2b.json`** | `schema/n2b.schema.json` | **Nouvelle surface publique** (pas une surface gelée existante). Opt-in, rétro-compatible avec l'absence de manifeste. Semi-gelée dès publication : évolutions additives sans bump, breaking → bump `version`. | 4, 5 | schéma versionné ; `version` du manifeste distinct de `schema_version` du `Finding` |

### Les trois changements de sortie assumés

1. **Phase 0 / PS4** — `--fix` cesse de réécrire les commandes commentées. Fix de bug.
2. **Phase 2 / PS1** — les faux positifs sur identifiant homonyme disparaissent → moins
   de findings. Amélioration de précision.
3. **Phase 3** — le champ `compat` apparaît dans la sortie ; les `imports/*` 🔴 montent
   en `error`. Enrichissement + sévérité correcte.

Chacun est : (a) justifié dans le message de commit, (b) capturé par une régénération de
baseline assumée, (c) documenté au `CHANGELOG.md`. **Jamais de changement silencieux.**

### Ce qui n'est PAS touché

- Aucun Rule ID renommé ou supprimé.
- Aucun flag CLI retiré.
- `schema_version` reste `2` (pas de `v3.json`).
- L'ABI cdylib reste v1.
- Le format des 5 rendus (text/json/jsonl/md/sarif) reste structurellement identique —
  seul le *contenu* (champ `compat`, moins de faux positifs) évolue.

## 2. Registre de risques

| # | Risque | Prob. | Impact | Mitigation | Phase |
|---|---|---|---|---|---|
| R1 | La transcription Rust→`.toml` (Phase 1) introduit une divergence subtile | moyenne | élevé | diff baseline octet-à-octet obligatoire ; transcrire par petits lots commités | 1 |
| R2 | Le parseur de `nodejs-compat.mdx` casse au prochain bump canary (format markdown libre) | élevée | moyen | parser tolérant + test sur le mdx commité ; un changement de format fait échouer le test *explicitement* | 4 |
| R3 | L'AST oxc rate un pattern que la regex attrapait (réexport, alias) | moyenne | moyen | proptests nominaux ; documenter les limites ; pas de suppression du mode `Text` de secours | 2 |
| R4 | `xtask sync-coverage --check` en CI dépend de `upstream/` (gitignoré) | certaine | moyen | snapshot commité `registry/.upstream-snapshot.toml` ; `--check` compare hors-ligne ; job périodique rafraîchit | 4, 7 |
| R5 | Un `template` de réécriture produit du code qui ne tourne pas sous Bun | moyenne | élevé | repo témoin réel + `bun test` vert = critère dur Phase 5 ; rewrites risqués = `manual` par défaut | 5 |
| R6 | Régénération de baseline masque une vraie régression | moyenne | élevé | chaque phase régénère *et justifie* finement ; la régénération finale (Phase 7) ne doit montrer aucun diff inattendu | 7 |
| R7 | `rpb-dashboard` valide en mode strict et casse sur le champ `compat` | faible | moyen | champ dans le schéma `properties` → validateur conforme l'accepte ; courtoisie : prévenir | 3 |
| R8 | Le codegen recréé (PS6) produit un `schema.rs` différent du fichier commité | moyenne | faible | commit de régénération séparé *avant* le reste, pour isoler le diff | 0 |
| R9 | Phase 1 retarde tout (chemin critique, bloque 2/3/4) | moyenne | élevé | Phase 1 = refactor pur, pas de design ouvert ; périmètre fermé ; commits fréquents | 1 |
| R10 | Découpe `n2b-cli` (Phase 7.5) casse des chemins de module | faible | faible | optionnelle, dernier commit isolé ; si casse → retirer, non nécessaire au « parfait » | 7 |
| R11 | Le manifeste `n2b.json` ouvre une surface de config trop large dès le v1 (scope creep) | moyenne | moyen | périmètre v1 fermé et documenté (cf. [05 §10](05-manifeste-n2b-json.md)) — `extends`, hooks, profils explicitement hors-scope, tous additifs plus tard | 4 |

## 3. Séquençage & dépendances

```
Phase 0 ──┬─→ Phase 1 ──┬─→ Phase 2 ───────────────┐
(socle)   │  (registre) ├─→ Phase 3 ──→ Phase 6    ├─→ Phase 7
          │             └─→ Phase 4 ──→ Phase 5 ───┘  (garde-fous)
          │
   PS6 (codegen) ────────────────────→ requis par Phase 3
```

### Chemin critique

`Phase 0 → Phase 1 → Phase 4 → Phase 5 → Phase 7`

C'est la chaîne la plus longue. Phases 2, 3, 6 sont des branches qui rejoignent en
Phase 7.

### Parallélisation possible (après Phase 1)

- **Phase 2** (AST) et **Phase 4** (couverture) sont indépendantes — parallélisables.
- **Phase 3** (compat/schéma) est indépendante de 2 et 4 — parallélisable.
- **Phase 5** attend Phase 4. **Phase 6** attend Phase 3.

### Jalons

| Jalon | Phases | État livrable |
|---|---|---|
| **M1 — Socle sain** | 0 | codegen réparé, duplication tuée, repo propre. Sortie inchangée (sauf PS4). |
| **M2 — Registre en place** | 0, 1 | règles data-driven, PS3 résolu. Sortie octet-identique. |
| **M3 — Précision & couverture** | + 2, 3, 4 | AST-first (0 faux positif), compat exposé, 0 trou de couverture. |
| **M4 — Migration mécanique** | + 5, 6 | `--migrate` mécanique, report card, 🔴 → bunpp. Pilier 2 ≈ 100 %. |
| **M5 — Verrouillé** | + 7 | CI anti-drift, doc à jour, les 4 critères de « parfait » vérifiés. |

### Estimation de charge relative

| Phase | Charge | Pourquoi |
|---|---|---|
| 0 | moyenne | 6 PS, mais chacun ciblé ; PS6 (codegen) est le plus incertain |
| 1 | **élevée** | ~200 entrées à transcrire sans divergence — minutieux, chemin critique |
| 2 | élevée | l'`ImportGraph` oxc + refonte du matching = vrai travail d'ingénierie |
| 3 | moyenne | mécanique une fois le codegen réparé ; le gros est la saisie de `modules.toml` |
| 4 | **élevée** | `xtask` + 6 nouveaux scanners + `shell.rs` réel |
| 5 | **élevée** | ~130 `rewrite` à écrire + report card + repo témoin |
| 6 | faible | `bunpp_cmd.rs` existe déjà ; juste le câblage registre ↔ bunpp |
| 7 | moyenne | tests + CI + doc ; 7.5 (découpe CLI) optionnelle |

## 4. Définition de « fini »

Le refactor est terminé quand **les 5 jalons sont atteints** et que les **4 critères de
parfait** ([README.md](README.md)) passent en CI :

1. `cargo xtask sync-coverage --check` → 0 module/API Node sans entrée registre.
2. Toute entrée 🟢/🟡 a une `rewrite` non-`manual` (ou `codemod_hint` justifié).
3. La report card d'un repo Node réel = résidu manuel explicite et justifié.
4. Proptest « identifiant homonyme » = 0 faux positif.

Plus : `cargo test --workspace` + `bash tests/compare-baseline.sh` +
`cargo clippy -D warnings` + `cargo fmt --check` + `bun run codegen:schema:check` — tous
verts.
