export type Severity = "info" | "warn" | "error";

export interface Finding {
  file: string;
  line: number;
  col: number;
  ruleId: string;
  severity: Severity;
  message: string;
  original: string;
  replacement?: string;
  /** True if the rule can be applied automatically in --fix mode. */
  autofix: boolean;
  /** True if the rule is considered aggressive (API migration). */
  aggressive?: boolean;
}

export interface FileFix {
  file: string;
  before: string;
  after: string;
  findings: Finding[];
}

export interface RunOptions {
  root: string;
  mode: "check" | "fix" | "aggressive";
  report: "text" | "json" | "markdown";
  quiet: boolean;
  ignore: string[];
}

export interface Rule {
  id: string;
  description: string;
  aggressive?: boolean;
  /** Inspect a file's content, return findings + optional new content. */
  apply(input: { path: string; content: string; opts: RunOptions }):
    | { findings: Finding[]; content?: string }
    | Promise<{ findings: Finding[]; content?: string }>;
}
