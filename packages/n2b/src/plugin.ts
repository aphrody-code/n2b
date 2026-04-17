import type { BunPlugin } from "bun";
import { applyBunApiRules } from "./rules/bun-apis";
import { applyNodeImportRules } from "./rules/node-imports";
import type { Finding } from "./types";
import { colors as c } from "./util";

export interface Node2BunPluginOptions {
  /**
   * Active les suggestions de migration d'API (Node → Bun.*).
   * Par défaut : false (check uniquement).
   */
  aggressive?: boolean;
  /**
   * Silence les warnings fichier-par-fichier ; affiche uniquement le bilan final.
   */
  quiet?: boolean;
  /**
   * Liste de RegExp — les chemins correspondants sont ignorés.
   */
  ignore?: RegExp[];
  /**
   * "warn" (défaut) — findings loggés mais le build continue.
   * "error"         — le build échoue si des findings sont trouvés.
   */
  onFindings?: "warn" | "error";
  /**
   * Applique les rewrites sûrs et renvoie le contenu transformé à Bun.
   * Équivalent de --fix sur le CLI. Par défaut : false.
   */
  transform?: boolean;
}

interface Accumulator {
  path: string;
  findings: Finding[];
}

const SOURCE_FILTER = /\.[jt]sx?$|\.m[jt]s$|\.c[jt]s$/;

export function node2bunPlugin(opts: Node2BunPluginOptions = {}): BunPlugin {
  const {
    aggressive = false,
    quiet = false,
    ignore = [],
    onFindings = "warn",
    transform = false,
  } = opts;

  // Mutable state réinitialisé à chaque onStart (watch mode).
  let accumulated: Accumulator[] = [];

  return {
    name: "node2bun",

    setup(build) {
      build.onStart(() => {
        accumulated = [];
      });

      build.onLoad({ filter: SOURCE_FILTER }, async ({ path }) => {
        if (ignore.some((re) => re.test(path))) return undefined;

        let source: string;
        try {
          source = await Bun.file(path).text();
        } catch {
          return undefined;
        }

        const nodeImp = applyNodeImportRules(path, source, aggressive);
        const bunApi = applyBunApiRules(path, nodeImp.content, aggressive);
        const all = [...nodeImp.findings, ...bunApi.findings];

        if (all.length > 0) {
          accumulated.push({ path, findings: all });

          if (!quiet) {
            const rel = toRel(path);
            for (const f of all) {
              const loc = `${c.dim}${rel}:${f.line}:${f.col}${c.reset}`;
              const tag = `${c.cyan}${f.ruleId}${c.reset}`;
              const repl = f.replacement
                ? ` ${c.dim}→${c.reset} ${c.green}${ellipsis(f.replacement, 60)}${c.reset}`
                : "";
              console.warn(`${c.yellow}[node2bun]${c.reset} ${loc} ${tag} ${f.message}${repl}`);
            }
          }
        }

        // Mode transform : renvoie le contenu réécrit à Bun (équiv. --fix).
        if (transform && (nodeImp.content !== source || bunApi.content !== nodeImp.content)) {
          const ext = path.split(".").pop() ?? "js";
          return {
            contents: bunApi.content,
            loader: extToLoader(ext),
          };
        }

        return undefined;
      });

      build.onEnd(() => {
        const total = accumulated.reduce((n, a) => n + a.findings.length, 0);
        if (total === 0) {
          if (!quiet)
            console.log(`${c.green}[node2bun]${c.reset} aucune incompatibilité Node→Bun détectée`);
          return;
        }

        const files = accumulated.length;
        const summary =
          `${c.bold}[node2bun]${c.reset} ${c.yellow}${total}${c.reset} finding(s) ` +
          `dans ${c.yellow}${files}${c.reset} fichier(s)`;
        console.warn(`\n${summary}`);

        if (!quiet) {
          console.warn(
            `${c.dim}Conseil : ajouter node2bunPlugin({ transform: true }) pour appliquer les rewrites sûrs,` +
              ` ou lancer \`node2bun . --fix\` sur le projet.${c.reset}`,
          );
        }

        if (onFindings === "error") {
          throw new Error(
            `[node2bun] ${total} incompatibilité(s) Node→Bun — corriger avant de builder (ou passer onFindings: "warn" pour ignorer)`,
          );
        }
      });
    },
  };
}

function toRel(abs: string): string {
  try {
    return abs.startsWith(process.cwd()) ? abs.slice(process.cwd().length + 1) : abs;
  } catch {
    return abs;
  }
}

function ellipsis(s: string, n: number): string {
  return s.length > n ? `${s.slice(0, n - 1)}…` : s;
}

function extToLoader(
  ext: string,
): import("bun").BuildConfig["target"] extends infer _ ? any : never {
  switch (ext) {
    case "tsx":
      return "tsx";
    case "jsx":
      return "jsx";
    case "ts":
    case "mts":
    case "cts":
      return "ts";
    default:
      return "js";
  }
}
