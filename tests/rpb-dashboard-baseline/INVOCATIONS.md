# Invocations `n2b` utilisées par rpb-dashboard

Extraites de `rpb-dashboard/bun/README.md` et `rpb-dashboard/bun/MIGRATION_PLAN.md` — ces invocations définissent le contrat CLI-as-API que le refactor doit préserver. Tout test de contrat (`crates/n2b-cli/tests/contract.rs`) doit rejouer ces commandes et comparer à `tests/rpb-dashboard-baseline/`.

## Invocations canoniques

```bash
# Baseline capture (phase initiale d'un projet)
n2b . --report text  > bun/reports/n2b-baseline.txt
n2b . --report md    > bun/reports/n2b-baseline.md
n2b . --report json  > bun/reports/n2b-baseline.json
n2b . --report jsonl > bun/reports/n2b-baseline.jsonl
n2b . --report sarif > bun/reports/n2b-baseline.sarif

# Meta
n2b rules            > bun/reports/n2b-rules.txt      # format text implicite
n2b prompt .         > bun/reports/n2b-llm-prompt.md  # pour LLM agent

# Après migration (diff vs baseline)
n2b . --report md    > bun/reports/n2b-after.md
diff bun/reports/n2b-baseline.md bun/reports/n2b-after.md

# Fix ciblé (jamais global sur rpb-dashboard selon MIGRATION_PLAN.md Phase 1)
n2b . --fix   # appliqué après review des findings
```

## Contrat implicite

- Le flag `--report` accepte `text|json|jsonl|md|markdown|sarif` (les deux `md` et `markdown` sont équivalents).
- La sortie `--report json` est consommée par des scripts Bun et par l'agent Claude (`rpb-dashboard/.claude/commands/n2b.md`). Son schéma doit rester stable par version.
- Exit code 0 = aucun finding, 1 = findings en mode check, 2 = erreur.
- `n2b rules` (sans `--report`) produit un format texte lisible humainement.
- `n2b prompt .` génère du markdown pour copier-coller dans un LLM.

## Observations pré-refactor

1. **Le schéma `schema/v2.json` est drifté** — le JSON émis par le binaire v0.2.0 utilise `schema_version`, `$schema`, `files_scanned`, `findings_total` à la racine, alors que le schéma officiel documente `version`, `tool_version`, `summary.files_scanned`. Le refactor aligne le schéma sur l'implémentation (casser le schéma casserait rpb-dashboard).
2. **Le champ `tool` vaut `"node2bun"`** dans le JSON, pas `"n2b"` comme documenté dans le schéma. À préserver (c'est ce que rpb-dashboard voit).
3. **Baseline capturée le 2026-04-18** avec `n2b 0.2.0` installé en `/usr/local/bin/n2b`.
