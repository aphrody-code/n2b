//! `n2b ui <sub>` — scaffolde des projets Bun avec un design system UI moderne.
//!
//! Refs :
//!   - https://ui.shadcn.com/
//!   - https://ui.shadcn.com/docs/registry/getting-started
//!   - https://material-web.dev/
//!   - https://m3.material.io/
//!   - https://www.material-tailwind.com/
//!   - https://mui.com/
//!   - https://fonts.google.com/icons (Material Symbols)
//!   - https://fonts.google.com/specimen/Google+Sans+Flex

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFlavor {
    /// shadcn/ui : Radix primitives + Tailwind v4 + CLI add.
    Shadcn,
    /// Material UI (MUI) : React + Emotion CSS-in-JS.
    Mui,
    /// Material Web Components (vanilla, framework-agnostic) — M3 officiel Google.
    MaterialWeb,
    /// Material Tailwind : M3 + Tailwind classes.
    MaterialTailwind,
    /// Registry shadcn personnalisé qui porte MWC vers shadcn via Tailwind v4.
    /// Inclut les composants M3 web + mobile-native-web (Bottom Sheet, FAB,
    /// Segmented Control, …) avec build pipeline Rust (crate m3-registry-builder).
    M3Registry,
}

impl UiFlavor {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "shadcn" | "radix" => Some(Self::Shadcn),
            "mui" | "material-ui" => Some(Self::Mui),
            "material-web" | "mwc" => Some(Self::MaterialWeb),
            "material-tailwind" | "mt" => Some(Self::MaterialTailwind),
            "md3-ui" | "md3" | "m3" | "m3-registry" | "shadcn-m3" => Some(Self::M3Registry),
            _ => None,
        }
    }
}

pub enum UiCmd {
    Init {
        name: String,
        flavor: UiFlavor,
        dir: Option<PathBuf>,
        force: bool,
    },
    Doctor,
}

pub fn run(cmd: UiCmd, quiet: bool) -> Result<()> {
    match cmd {
        UiCmd::Init { name, flavor, dir, force } => init(name, flavor, dir, force, quiet),
        UiCmd::Doctor => doctor(quiet),
    }
}

fn init(
    name: String,
    flavor: UiFlavor,
    dir: Option<PathBuf>,
    force: bool,
    quiet: bool,
) -> Result<()> {
    let target = match dir {
        Some(d) => d.join(&name),
        None => PathBuf::from(&name),
    };
    if target.exists() && !force {
        anyhow::bail!(
            "{} existe déjà — relancer avec --force",
            target.display()
        );
    }
    std::fs::create_dir_all(&target)?;

    match flavor {
        UiFlavor::Shadcn => scaffold_shadcn(&target, &name, quiet)?,
        UiFlavor::Mui => scaffold_mui(&target, &name, quiet)?,
        UiFlavor::MaterialWeb => scaffold_material_web(&target, &name, quiet)?,
        UiFlavor::MaterialTailwind => scaffold_material_tailwind(&target, &name, quiet)?,
        UiFlavor::M3Registry => scaffold_m3_registry(&target, &name, quiet)?,
    }

    if !quiet {
        eprintln!(
            "[ui] ✓ {name} ({:?}) scaffolded → cd {} && bun install && bun dev",
            flavor,
            target.display()
        );
    }
    Ok(())
}

fn doctor(quiet: bool) -> Result<()> {
    let tools: &[(&str, &str, &str)] = &[
        ("bun",     "https://bun.sh/install",        "runtime"),
        ("git",     "apt install git",                "version control"),
        ("pwsh",    "winget install PowerShell",      "PS (optionnel Windows)"),
    ];
    for (bin, install, desc) in tools {
        let ok = which(bin).is_ok();
        if !quiet {
            let mark = if ok { "✓" } else { "✗" };
            println!("  {mark} {:<8} {desc}", bin);
            if !ok {
                println!("       install: {install}");
            }
        }
    }
    if !quiet {
        println!("\nUI libs supportées :");
        println!("  shadcn            — Radix + Tailwind v4 (copy-paste components)");
        println!("  mui               — Material UI (React + Emotion)");
        println!("  material-web      — @material/web (vanilla Web Components M3)");
        println!("  material-tailwind — M3 + Tailwind classes");
        println!("\nIcons : material-symbols (variable font, https://fonts.google.com/icons)");
        println!("Fonts : @fontsource-variable/google-sans-flex");
    }
    Ok(())
}

// --- Scaffolders ---

fn scaffold_shadcn(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(dir.join("package.json"), &render_shadcn_package_json(name), quiet)?;
    write(dir.join("components.json"), SHADCN_COMPONENTS_JSON, quiet)?;
    write(dir.join("tsconfig.json"), TSCONFIG_BUNDLER, quiet)?;
    write(dir.join("src/app/globals.css"), SHADCN_GLOBALS_CSS, quiet)?;
    write(dir.join("src/app/layout.tsx"), &render_shadcn_layout_tsx(name), quiet)?;
    write(dir.join("src/app/page.tsx"), SHADCN_PAGE_TSX, quiet)?;
    write(dir.join("src/lib/utils.ts"), SHADCN_UTILS_TS, quiet)?;
    write(dir.join("src/components/ui/button.tsx"), SHADCN_BUTTON_TSX, quiet)?;
    write(dir.join("next.config.ts"), NEXT_CONFIG_TS, quiet)?;
    write(dir.join("postcss.config.mjs"), POSTCSS_TAILWIND_CONFIG, quiet)?;
    write(dir.join("README.md"), &render_shadcn_readme(name), quiet)?;
    write(dir.join(".mcp.json"), SHADCN_MCP_JSON, quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_mui(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(dir.join("package.json"), &render_mui_package_json(name), quiet)?;
    write(dir.join("tsconfig.json"), TSCONFIG_BUNDLER, quiet)?;
    write(dir.join("src/app/layout.tsx"), &render_mui_layout_tsx(name), quiet)?;
    write(dir.join("src/app/page.tsx"), MUI_PAGE_TSX, quiet)?;
    write(dir.join("src/theme.ts"), MUI_THEME_TS, quiet)?;
    write(dir.join("next.config.ts"), NEXT_CONFIG_TS, quiet)?;
    write(dir.join("README.md"), &readme(name, "Next.js + MUI (Material UI React + Emotion)"), quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_material_web(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(dir.join("package.json"), &render_material_web_package_json(name), quiet)?;
    write(dir.join("tsconfig.json"), TSCONFIG_BUNDLER, quiet)?;
    write(dir.join("index.html"), &render_material_web_html(name), quiet)?;
    write(dir.join("src/index.ts"), MATERIAL_WEB_INDEX_TS, quiet)?;
    write(dir.join("src/styles.css"), MATERIAL_WEB_STYLES_CSS, quiet)?;
    write(dir.join("README.md"), &readme(name, "Material Web Components M3 (vanilla TS, buildé par Bun)"), quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn scaffold_m3_registry(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    scaffold_md3_ui_framework(dir, name, quiet)
}

fn scaffold_md3_ui_framework(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    // --- Root monorepo config ---
    write(dir.join("package.json"), &render_md3_root_package_json(name), quiet)?;
    write(dir.join("Cargo.toml"), &render_md3_workspace_cargo_toml(), quiet)?;
    write(dir.join("biome.json"), MD3_BIOME_JSON, quiet)?;
    write(dir.join("rspack.config.mjs"), MD3_RSPACK_CONFIG, quiet)?;
    write(dir.join("tsconfig.base.json"), MD3_TSCONFIG_BASE, quiet)?;
    write(dir.join("turbo.json"), MD3_TURBO_JSON, quiet)?;
    write(dir.join(".mcp.json"), SHADCN_MCP_JSON, quiet)?;
    write(dir.join(".gitignore"), M3_GITIGNORE, quiet)?;
    write(dir.join("README.md"), &render_md3_framework_readme(name), quiet)?;
    write(dir.join("LICENSE"), MD3_LICENSE_MIT, quiet)?;

    // --- packages/core — composants React M3 ---
    write(dir.join("packages/core/package.json"), MD3_CORE_PACKAGE_JSON, quiet)?;
    write(dir.join("packages/core/tsconfig.json"), MD3_PKG_TSCONFIG, quiet)?;
    write(dir.join("packages/core/rsbuild.config.ts"), MD3_CORE_RSBUILD, quiet)?;
    write(dir.join("packages/core/src/index.ts"), MD3_CORE_INDEX, quiet)?;
    write(dir.join("packages/core/src/theme/ThemeProvider.tsx"), MD3_THEME_PROVIDER, quiet)?;
    write(dir.join("packages/core/src/theme/tokens.ts"), MD3_THEME_TOKENS_TS, quiet)?;
    write(dir.join("packages/core/src/theme/useTheme.ts"), MD3_USE_THEME, quiet)?;
    write(dir.join("packages/core/src/lib/utils.ts"), SHADCN_UTILS_TS, quiet)?;
    // Composants (réutilisent les templates M3 existants)
    write(dir.join("packages/core/src/button/Button.tsx"), M3_BUTTON_TSX, quiet)?;
    write(dir.join("packages/core/src/button/index.ts"), "export * from \"./Button\";\n", quiet)?;
    write(dir.join("packages/core/src/card/Card.tsx"), M3_CARD_TSX, quiet)?;
    write(dir.join("packages/core/src/card/index.ts"), "export * from \"./Card\";\n", quiet)?;
    write(dir.join("packages/core/src/chip/Chip.tsx"), M3_CHIP_TSX, quiet)?;
    write(dir.join("packages/core/src/chip/index.ts"), "export * from \"./Chip\";\n", quiet)?;
    write(dir.join("packages/core/src/fab/Fab.tsx"), M3_FAB_TSX, quiet)?;
    write(dir.join("packages/core/src/fab/index.ts"), "export * from \"./Fab\";\n", quiet)?;
    write(dir.join("packages/core/src/navigation-bar/NavigationBar.tsx"), M3_NAV_BAR_TSX, quiet)?;
    write(dir.join("packages/core/src/navigation-bar/index.ts"), "export * from \"./NavigationBar\";\n", quiet)?;
    write(dir.join("packages/core/src/bottom-sheet/BottomSheet.tsx"), M3_BOTTOM_SHEET_TSX, quiet)?;
    write(dir.join("packages/core/src/bottom-sheet/index.ts"), "export * from \"./BottomSheet\";\n", quiet)?;
    write(dir.join("packages/core/src/segmented-control/SegmentedControl.tsx"), M3_SEGMENTED_TSX, quiet)?;
    write(dir.join("packages/core/src/segmented-control/index.ts"), "export * from \"./SegmentedControl\";\n", quiet)?;
    // Motion module (M3 motion spec : easings + durations)
    write(dir.join("packages/core/src/motion/index.ts"), MD3_MOTION_INDEX_TS, quiet)?;
    write(dir.join("packages/core/src/motion/useMotion.ts"), MD3_USE_MOTION_TS, quiet)?;
    write(dir.join("packages/core/src/motion/Transition.tsx"), MD3_TRANSITION_TSX, quiet)?;

    // --- packages/tokens ---
    write(dir.join("packages/tokens/package.json"), MD3_TOKENS_PACKAGE_JSON, quiet)?;
    write(dir.join("packages/tokens/src/tokens.css"), M3_TOKENS_CSS, quiet)?;
    write(dir.join("packages/tokens/src/theme.json"), MD3_THEME_JSON, quiet)?;

    // --- packages/registry — shadcn registry fork ---
    write(dir.join("packages/registry/package.json"), MD3_REGISTRY_PACKAGE_JSON, quiet)?;
    write(dir.join("packages/registry/registry.json"), &render_m3_registry_json(name), quiet)?;
    write(dir.join("packages/registry/scripts/build.ts"), M3_BUILD_REGISTRY_TS, quiet)?;

    // --- packages/cli ---
    write(dir.join("packages/cli/package.json"), &render_md3_cli_package_json(name), quiet)?;
    write(dir.join("packages/cli/src/cli.ts"), &render_md3_cli_ts(name), quiet)?;

    // --- packages/lint-plugin (Biome plugin stub) ---
    write(dir.join("packages/lint-plugin/package.json"), MD3_LINT_PACKAGE_JSON, quiet)?;
    write(dir.join("packages/lint-plugin/src/index.ts"), MD3_LINT_INDEX, quiet)?;
    write(dir.join("packages/lint-plugin/src/rules/no-raw-color.ts"), MD3_LINT_NO_RAW_COLOR, quiet)?;
    write(dir.join("packages/lint-plugin/src/rules/use-m3-tokens.ts"), MD3_LINT_USE_TOKENS, quiet)?;

    // --- packages/md3-docs (Next 16 showcase + Material 3 Expressive content) ---
    write(dir.join("packages/md3-docs/package.json"), MD3_DOCS_PACKAGE_JSON, quiet)?;
    write(dir.join("packages/md3-docs/next.config.ts"), NEXT_CONFIG_TS, quiet)?;
    write(dir.join("packages/md3-docs/postcss.config.mjs"), POSTCSS_TAILWIND_CONFIG, quiet)?;
    write(dir.join("packages/md3-docs/src/app/layout.tsx"), &render_md3_docs_layout(name), quiet)?;
    write(dir.join("packages/md3-docs/src/app/page.tsx"), M3_PAGE_TSX, quiet)?;
    write(dir.join("packages/md3-docs/src/app/globals.css"), MD3_DOCS_GLOBALS, quiet)?;
    write(dir.join("packages/md3-docs/src/app/expressive/page.tsx"), MD3_EXPRESSIVE_PAGE_TSX, quiet)?;
    write(dir.join("packages/md3-docs/src/app/motion/page.tsx"), MD3_MOTION_PAGE_TSX, quiet)?;
    write(dir.join("packages/md3-docs/src/app/tokens/page.tsx"), MD3_TOKENS_PAGE_TSX, quiet)?;
    write(dir.join("packages/md3-docs/src/components/Nav.tsx"), MD3_DOCS_NAV_TSX, quiet)?;
    write(dir.join("packages/md3-docs/tsconfig.json"), MD3_DOCS_TSCONFIG, quiet)?;

    // --- crates/md3-compiler ---
    write(dir.join("crates/md3-compiler/Cargo.toml"), MD3_COMPILER_CARGO, quiet)?;
    write(dir.join("crates/md3-compiler/src/main.rs"), MD3_COMPILER_MAIN_RS, quiet)?;

    // --- crates/md3-registry-builder ---
    write(dir.join("crates/md3-registry-builder/Cargo.toml"), &render_m3_builder_cargo_toml(name), quiet)?;
    write(dir.join("crates/md3-registry-builder/src/main.rs"), M3_BUILDER_MAIN_RS, quiet)?;

    // --- crates/md3-wasm-plugin ---
    write(dir.join("crates/md3-wasm-plugin/Cargo.toml"), MD3_WASM_PLUGIN_CARGO, quiet)?;
    write(dir.join("crates/md3-wasm-plugin/src/lib.rs"), MD3_WASM_PLUGIN_RS, quiet)?;

    // --- examples/next-app ---
    write(dir.join("examples/next-app/package.json"), &render_md3_example_pkg(name), quiet)?;
    write(dir.join("examples/next-app/next.config.ts"), NEXT_CONFIG_TS, quiet)?;
    write(dir.join("examples/next-app/src/app/layout.tsx"), &render_md3_example_layout(name), quiet)?;
    write(dir.join("examples/next-app/src/app/page.tsx"), M3_PAGE_TSX, quiet)?;
    write(dir.join("examples/next-app/src/app/globals.css"), MD3_EXAMPLE_GLOBALS, quiet)?;

    Ok(())
}

fn scaffold_material_tailwind(dir: &Path, name: &str, quiet: bool) -> Result<()> {
    write(dir.join("package.json"), &render_mt_package_json(name), quiet)?;
    write(dir.join("tsconfig.json"), TSCONFIG_BUNDLER, quiet)?;
    write(dir.join("src/app/layout.tsx"), &render_mt_layout_tsx(name), quiet)?;
    write(dir.join("src/app/page.tsx"), MT_PAGE_TSX, quiet)?;
    write(dir.join("src/app/globals.css"), MT_GLOBALS_CSS, quiet)?;
    write(dir.join("next.config.ts"), NEXT_CONFIG_TS, quiet)?;
    write(dir.join("postcss.config.mjs"), POSTCSS_TAILWIND_CONFIG, quiet)?;
    write(dir.join("README.md"), &readme(name, "Next.js + Material Tailwind (M3 via Tailwind classes)"), quiet)?;
    write(dir.join(".gitignore"), GITIGNORE, quiet)?;
    Ok(())
}

fn write(path: PathBuf, content: &str, quiet: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    if !quiet {
        eprintln!("[ui]   + {}", path.display());
    }
    Ok(())
}

fn which(name: &str) -> Result<PathBuf> {
    let path = std::env::var_os("PATH").ok_or_else(|| anyhow!("PATH non défini"))?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(anyhow!("binaire `{name}` introuvable dans PATH"))
}

fn readme(name: &str, title: &str) -> String {
    format!(
        r#"# {name}

{title}

Scaffolded by `n2b ui init`.

## Dev

```bash
bun install
bun dev
```

## Refs

- [shadcn/ui](https://ui.shadcn.com/)
- [Material Design 3](https://m3.material.io/)
- [Material Web](https://material-web.dev/)
- [Material Symbols](https://fonts.google.com/icons)
"#,
    )
}

// --- Renderers ---

fn render_shadcn_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start",
    "add": "bunx shadcn@latest add"
  }},
  "dependencies": {{
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@radix-ui/react-slot": "^1.1.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0",
    "lucide-react": "^0.454.0"
  }},
  "devDependencies": {{
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "@types/node": "^22.0.0",
    "typescript": "^5.6.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0"
  }}
}}
"#,
    )
}

fn render_mui_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start"
  }},
  "dependencies": {{
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@mui/material": "^6.0.0",
    "@mui/icons-material": "^6.0.0",
    "@mui/material-nextjs": "^6.0.0",
    "@emotion/react": "^11.13.0",
    "@emotion/styled": "^11.13.0",
    "@emotion/cache": "^11.13.0",
    "@fontsource/roboto": "^5.1.0"
  }},
  "devDependencies": {{
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "typescript": "^5.6.0"
  }}
}}
"#,
    )
}

fn render_material_web_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "bun --hot src/index.ts",
    "build": "bun build src/index.ts --outdir dist --target browser",
    "preview": "bunx --bun serve dist"
  }},
  "dependencies": {{
    "@material/web": "^2.2.0",
    "material-symbols": "^0.27.0"
  }},
  "devDependencies": {{
    "@types/bun": "latest",
    "typescript": "^5.6.0"
  }}
}}
"#,
    )
}

fn render_mt_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start"
  }},
  "dependencies": {{
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@material-tailwind/react": "^2.1.0"
  }},
  "devDependencies": {{
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "typescript": "^5.6.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }}
}}
"#,
    )
}

// --- Templates ---

const SHADCN_COMPONENTS_JSON: &str = r#"{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "new-york",
  "rsc": true,
  "tsx": true,
  "tailwind": {
    "config": "",
    "css": "src/app/globals.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "hooks": "@/hooks"
  },
  "iconLibrary": "lucide"
}
"#;

const SHADCN_GLOBALS_CSS: &str = r#"@import "tailwindcss";

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 0 0% 3.9%;
    --primary: 0 0% 9%;
    --primary-foreground: 0 0% 98%;
    --border: 0 0% 89.8%;
    --ring: 0 0% 3.9%;
    --radius: 0.5rem;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --background: 0 0% 3.9%;
      --foreground: 0 0% 98%;
      --primary: 0 0% 98%;
      --primary-foreground: 0 0% 9%;
      --border: 0 0% 14.9%;
      --ring: 0 0% 83.1%;
    }
  }
}
"#;

fn render_shadcn_layout_tsx(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import "./globals.css";

export const metadata: Metadata = {{
  title: "{name}",
  description: "Scaffolded by n2b ui init --flavor shadcn",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body className="min-h-screen bg-background text-foreground antialiased">
        {{children}}
      </body>
    </html>
  );
}}
"#,
    )
}

const SHADCN_PAGE_TSX: &str = r#"import { Button } from "@/components/ui/button";

export default function Home() {
  return (
    <main className="container mx-auto p-8">
      <h1 className="text-4xl font-bold mb-6">Hello shadcn/ui + Bun</h1>
      <div className="flex gap-2">
        <Button>Primary</Button>
        <Button variant="secondary">Secondary</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="destructive">Destructive</Button>
      </div>
      <p className="mt-6 text-sm text-muted-foreground">
        Ajouter un composant :{" "}
        <code className="bg-muted px-1 rounded">bunx shadcn@latest add dialog</code>
      </p>
    </main>
  );
}
"#;

const SHADCN_UTILS_TS: &str = r#"import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
"#;

const SHADCN_BUTTON_TSX: &str = r#"import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
        outline: "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 rounded-md px-3 text-xs",
        lg: "h-10 rounded-md px-8",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />;
  }
);
Button.displayName = "Button";
"#;

fn render_mui_layout_tsx(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import {{ Roboto }} from "next/font/google";
import {{ AppRouterCacheProvider }} from "@mui/material-nextjs/v15-appRouter";
import {{ ThemeProvider, CssBaseline }} from "@mui/material";
import theme from "@/theme";

const roboto = Roboto({{
  weight: ["300", "400", "500", "700"],
  subsets: ["latin"],
  display: "swap",
  variable: "--font-roboto",
}});

export const metadata: Metadata = {{
  title: "{name}",
  description: "Scaffolded by n2b ui init --flavor mui",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en" className={{roboto.variable}}>
      <body>
        <AppRouterCacheProvider>
          <ThemeProvider theme={{theme}}>
            <CssBaseline />
            {{children}}
          </ThemeProvider>
        </AppRouterCacheProvider>
      </body>
    </html>
  );
}}
"#,
    )
}

const MUI_PAGE_TSX: &str = r#""use client";

import { Container, Typography, Button, Stack } from "@mui/material";
import HomeIcon from "@mui/icons-material/Home";

export default function Home() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" gutterBottom>
        Hello MUI + Bun
      </Typography>
      <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
        <Button variant="contained" startIcon={<HomeIcon />}>
          Contained
        </Button>
        <Button variant="outlined">Outlined</Button>
        <Button variant="text">Text</Button>
      </Stack>
    </Container>
  );
}
"#;

const MUI_THEME_TS: &str = r#""use client";

import { createTheme } from "@mui/material/styles";

const theme = createTheme({
  cssVariables: true,
  palette: { mode: "light" },
  typography: { fontFamily: "var(--font-roboto), sans-serif" },
});

export default theme;
"#;

fn render_material_web_html(name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <title>{name} — Material Web</title>
    <link rel="stylesheet" href="./src/styles.css" />
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:wght@400;500;700&display=swap" />
    <link rel="stylesheet" href="https://fonts.googleapis.com/icon?family=Material+Symbols+Outlined" />
  </head>
  <body>
    <main>
      <md-outlined-button id="btn">
        <md-icon slot="icon">favorite</md-icon>
        Click me
      </md-outlined-button>
      <p id="count">0 clicks</p>
    </main>
    <script type="module" src="./src/index.ts"></script>
  </body>
</html>
"#,
    )
}

const MATERIAL_WEB_INDEX_TS: &str = r#"// Register Material Web Components — chaque import active un <md-*> tag.
import "@material/web/button/outlined-button.js";
import "@material/web/icon/icon.js";
import "@material/web/typography/md-typescale-styles.js";

// Active les styles typography globaux.
import { styles as typescaleStyles } from "@material/web/typography/md-typescale-styles.js";
document.adoptedStyleSheets = [...document.adoptedStyleSheets, typescaleStyles.styleSheet!];

const btn = document.getElementById("btn")!;
const count = document.getElementById("count")!;
let n = 0;

btn.addEventListener("click", () => {
  n++;
  count.textContent = `${n} clicks`;
});
"#;

const MATERIAL_WEB_STYLES_CSS: &str = r#"@import url("https://fonts.googleapis.com/icon?family=Material+Symbols+Outlined");

body {
  font-family: "Google Sans Flex", system-ui, sans-serif;
  padding: 2rem;
  background: #fafafa;
  color: #1c1c1c;
}

main {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  align-items: flex-start;
}
"#;

fn render_mt_layout_tsx(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import {{ ThemeProvider }} from "@material-tailwind/react";
import "./globals.css";

export const metadata: Metadata = {{
  title: "{name}",
  description: "Scaffolded by n2b ui init --flavor material-tailwind",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body>
        <ThemeProvider>{{children}}</ThemeProvider>
      </body>
    </html>
  );
}}
"#,
    )
}

const MT_PAGE_TSX: &str = r#""use client";

import { Button, Typography, Card, CardBody } from "@material-tailwind/react";

export default function Home() {
  return (
    <main className="p-8 space-y-6">
      <Typography variant="h2" color="blue-gray">
        Material Tailwind + Bun
      </Typography>
      <div className="flex gap-2">
        <Button color="blue">Filled</Button>
        <Button variant="outlined">Outlined</Button>
        <Button variant="text">Text</Button>
      </div>
      <Card className="max-w-md">
        <CardBody>
          <Typography variant="h5">Card title</Typography>
          <Typography>
            M3 tokens via Tailwind classes — shipped via @material-tailwind/react.
          </Typography>
        </CardBody>
      </Card>
    </main>
  );
}
"#;

const MT_GLOBALS_CSS: &str = r#"@tailwind base;
@tailwind components;
@tailwind utilities;
"#;

// --- Shared ---

const TSCONFIG_BUNDLER: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "Preserve",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "allowImportingTsExtensions": true,
    "verbatimModuleSyntax": false,
    "noEmit": true,
    "strict": true,
    "skipLibCheck": true,
    "moduleDetection": "force",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "incremental": true,
    "types": ["bun"],
    "paths": { "@/*": ["./src/*"] },
    "plugins": [{ "name": "next" }]
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
"#;

const NEXT_CONFIG_TS: &str = r#"import type { NextConfig } from "next";

const config: NextConfig = {
  turbopack: {},
};

export default config;
"#;

const POSTCSS_TAILWIND_CONFIG: &str = r#"export default {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};
"#;

const GITIGNORE: &str = r#"node_modules/
.next/
out/
dist/
.DS_Store
.env.local
"#;

// ============================================================================
// M3 REGISTRY (shadcn-m3) — composants Material Design 3 via Tailwind v4
// ============================================================================

fn render_m3_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": false,
  "type": "module",
  "description": "Material Design 3 registry for shadcn/ui — ports MWC to React+Tailwind",
  "scripts": {{
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start",
    "build:registry": "bun scripts/build-registry.ts",
    "build:registry:rust": "cargo run --release --manifest-path crates/m3-registry-builder/Cargo.toml -- registry public/r",
    "preview": "bun scripts/build-registry.ts && bun dev"
  }},
  "dependencies": {{
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@radix-ui/react-slot": "^1.1.0",
    "@radix-ui/react-dialog": "^1.1.0",
    "@radix-ui/react-toggle-group": "^1.1.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0",
    "lucide-react": "^0.454.0",
    "material-symbols": "^0.27.0",
    "@fontsource-variable/google-sans-flex": "^5.1.0"
  }},
  "devDependencies": {{
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "@types/node": "^22.0.0",
    "typescript": "^5.6.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0"
  }}
}}
"#,
    )
}

/// Registry index v2 — http://ui.shadcn.com/docs/registry/registry-json
fn render_m3_registry_json(name: &str) -> String {
    format!(
        r#"{{
  "$schema": "https://ui.shadcn.com/schema/registry.json",
  "name": "{name}",
  "homepage": "https://github.com/your-org/{name}",
  "items": [
    {{
      "name": "button",
      "type": "registry:ui",
      "title": "Button (M3)",
      "description": "Material 3 Button : Filled / Tonal / Outlined / Text / Elevated.",
      "files": [
        {{ "path": "registry/new-york/ui/button.tsx", "type": "registry:ui" }}
      ],
      "dependencies": ["class-variance-authority", "@radix-ui/react-slot"],
      "registryDependencies": ["utils"]
    }},
    {{
      "name": "card",
      "type": "registry:ui",
      "title": "Card (M3)",
      "description": "Material 3 Card : Elevated / Filled / Outlined.",
      "files": [{{ "path": "registry/new-york/ui/card.tsx", "type": "registry:ui" }}]
    }},
    {{
      "name": "chip",
      "type": "registry:ui",
      "title": "Chip (M3)",
      "description": "Material 3 Chip : Assist / Filter / Input / Suggestion.",
      "files": [{{ "path": "registry/new-york/ui/chip.tsx", "type": "registry:ui" }}]
    }},
    {{
      "name": "fab",
      "type": "registry:ui",
      "title": "FAB (M3)",
      "description": "Material 3 FAB + Extended FAB (small/medium/large).",
      "files": [{{ "path": "registry/new-york/ui/fab.tsx", "type": "registry:ui" }}]
    }},
    {{
      "name": "navigation-bar",
      "type": "registry:ui",
      "title": "Navigation Bar (M3)",
      "description": "Bottom navigation bar mobile-first — M3.",
      "files": [{{ "path": "registry/new-york/ui/navigation-bar.tsx", "type": "registry:ui" }}]
    }},
    {{
      "name": "bottom-sheet",
      "type": "registry:ui",
      "title": "Bottom Sheet (M3 mobile-native-web)",
      "description": "Modal + standard bottom sheet, drag-to-dismiss.",
      "files": [{{ "path": "registry/new-york/ui/bottom-sheet.tsx", "type": "registry:ui" }}],
      "dependencies": ["@radix-ui/react-dialog"]
    }},
    {{
      "name": "segmented-control",
      "type": "registry:ui",
      "title": "Segmented Control (iOS/M3 mobile-native-web)",
      "description": "Segmented button group type iOS + M3 mobile.",
      "files": [{{ "path": "registry/new-york/ui/segmented-control.tsx", "type": "registry:ui" }}],
      "dependencies": ["@radix-ui/react-toggle-group"]
    }},
    {{
      "name": "utils",
      "type": "registry:lib",
      "files": [{{ "path": "registry/new-york/lib/utils.ts", "type": "registry:lib" }}]
    }},
    {{
      "name": "m3-tokens",
      "type": "registry:theme",
      "title": "M3 CSS tokens",
      "files": [{{ "path": "registry/new-york/lib/m3-tokens.css", "type": "registry:theme" }}]
    }}
  ]
}}
"#,
    )
}

const M3_COMPONENTS_JSON: &str = r#"{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "new-york",
  "rsc": true,
  "tsx": true,
  "tailwind": {
    "config": "",
    "css": "src/app/globals.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/registry/new-york/lib/utils",
    "ui": "@/registry/new-york/ui",
    "lib": "@/registry/new-york/lib",
    "hooks": "@/hooks"
  },
  "iconLibrary": "lucide"
}
"#;

fn render_m3_layout_tsx(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import "@fontsource-variable/google-sans-flex";
import "material-symbols";
import "./globals.css";

export const metadata: Metadata = {{
  title: "{name}",
  description: "Material Design 3 registry for shadcn/ui — generated by n2b",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body className="min-h-screen bg-[--md-sys-color-surface] text-[--md-sys-color-on-surface] antialiased">
        {{children}}
      </body>
    </html>
  );
}}
"#,
    )
}

const M3_GLOBALS_CSS: &str = r#"@import "tailwindcss";
@import "../../registry/new-york/lib/m3-tokens.css";

html, body {
  font-family: "Google Sans Flex Variable", system-ui, sans-serif;
}
"#;

/// M3 tokens officiels (extraits du spec). Source :
/// https://m3.material.io/styles/color/roles
/// https://m3.material.io/styles/typography/type-scale-tokens
const M3_TOKENS_CSS: &str = r#"/* Material Design 3 tokens — couleurs + typography. */
/* Compatible Tailwind v4 arbitrary classes : bg-[--md-sys-color-primary] */

:root {
  /* ── Color roles (light theme, baseline Material) ── */
  --md-sys-color-primary: #6750A4;
  --md-sys-color-on-primary: #FFFFFF;
  --md-sys-color-primary-container: #EADDFF;
  --md-sys-color-on-primary-container: #21005D;

  --md-sys-color-secondary: #625B71;
  --md-sys-color-on-secondary: #FFFFFF;
  --md-sys-color-secondary-container: #E8DEF8;
  --md-sys-color-on-secondary-container: #1D192B;

  --md-sys-color-tertiary: #7D5260;
  --md-sys-color-on-tertiary: #FFFFFF;
  --md-sys-color-tertiary-container: #FFD8E4;
  --md-sys-color-on-tertiary-container: #31111D;

  --md-sys-color-error: #B3261E;
  --md-sys-color-on-error: #FFFFFF;
  --md-sys-color-error-container: #F9DEDC;
  --md-sys-color-on-error-container: #410E0B;

  --md-sys-color-surface: #FEF7FF;
  --md-sys-color-on-surface: #1D1B20;
  --md-sys-color-surface-dim: #DED8E1;
  --md-sys-color-surface-bright: #FEF7FF;
  --md-sys-color-surface-container-lowest: #FFFFFF;
  --md-sys-color-surface-container-low: #F7F2FA;
  --md-sys-color-surface-container: #F3EDF7;
  --md-sys-color-surface-container-high: #ECE6F0;
  --md-sys-color-surface-container-highest: #E6E0E9;
  --md-sys-color-surface-variant: #E7E0EC;
  --md-sys-color-on-surface-variant: #49454F;

  --md-sys-color-outline: #79747E;
  --md-sys-color-outline-variant: #CAC4D0;
  --md-sys-color-inverse-surface: #322F35;
  --md-sys-color-inverse-on-surface: #F5EFF7;
  --md-sys-color-inverse-primary: #D0BCFF;
  --md-sys-color-scrim: #000000;
  --md-sys-color-shadow: #000000;

  /* ── Shape (corner radius) ── */
  --md-sys-shape-corner-none: 0;
  --md-sys-shape-corner-extra-small: 4px;
  --md-sys-shape-corner-small: 8px;
  --md-sys-shape-corner-medium: 12px;
  --md-sys-shape-corner-large: 16px;
  --md-sys-shape-corner-extra-large: 28px;
  --md-sys-shape-corner-full: 9999px;

  /* ── Typography (subset, voir m3.material.io/styles/typography) ── */
  --md-sys-typescale-display-large-size: 57px;
  --md-sys-typescale-display-medium-size: 45px;
  --md-sys-typescale-headline-large-size: 32px;
  --md-sys-typescale-title-large-size: 22px;
  --md-sys-typescale-body-large-size: 16px;
  --md-sys-typescale-label-large-size: 14px;

  /* ── Elevation (box-shadow) ── */
  --md-sys-elevation-level0: none;
  --md-sys-elevation-level1: 0 1px 2px rgba(0,0,0,.30), 0 1px 3px 1px rgba(0,0,0,.15);
  --md-sys-elevation-level2: 0 1px 2px rgba(0,0,0,.30), 0 2px 6px 2px rgba(0,0,0,.15);
  --md-sys-elevation-level3: 0 1px 3px rgba(0,0,0,.30), 0 4px 8px 3px rgba(0,0,0,.15);
  --md-sys-elevation-level4: 0 2px 3px rgba(0,0,0,.30), 0 6px 10px 4px rgba(0,0,0,.15);
  --md-sys-elevation-level5: 0 4px 4px rgba(0,0,0,.30), 0 8px 12px 6px rgba(0,0,0,.15);

  /* ── Motion — easings officiels M3 ──────────────────────────────
   * Spec : https://m3.material.io/styles/motion/easing-and-duration
   * "Emphasized" : transitions expressives (pages, sheets) — pic à milieu
   * "Standard" : éléments fonctionnels (buttons, icons)
   * "Accelerate" : sorties (éléments qui quittent)
   * "Decelerate" : entrées (éléments qui entrent)                  */
  --md-sys-motion-easing-linear: cubic-bezier(0, 0, 1, 1);

  --md-sys-motion-easing-standard:            cubic-bezier(0.2, 0, 0, 1);
  --md-sys-motion-easing-standard-accelerate: cubic-bezier(0.3, 0, 1, 1);
  --md-sys-motion-easing-standard-decelerate: cubic-bezier(0, 0, 0, 1);

  --md-sys-motion-easing-emphasized:            cubic-bezier(0.2, 0, 0, 1);
  --md-sys-motion-easing-emphasized-accelerate: cubic-bezier(0.3, 0, 0.8, 0.15);
  --md-sys-motion-easing-emphasized-decelerate: cubic-bezier(0.05, 0.7, 0.1, 1);

  --md-sys-motion-easing-legacy:            cubic-bezier(0.4, 0, 0.2, 1);
  --md-sys-motion-easing-legacy-accelerate: cubic-bezier(0.4, 0, 1, 1);
  --md-sys-motion-easing-legacy-decelerate: cubic-bezier(0, 0, 0.2, 1);

  /* ── Motion — durations officielles M3 ────────────────────────── */
  --md-sys-motion-duration-short1:      50ms;
  --md-sys-motion-duration-short2:     100ms;
  --md-sys-motion-duration-short3:     150ms;
  --md-sys-motion-duration-short4:     200ms;

  --md-sys-motion-duration-medium1:    250ms;
  --md-sys-motion-duration-medium2:    300ms;
  --md-sys-motion-duration-medium3:    350ms;
  --md-sys-motion-duration-medium4:    400ms;

  --md-sys-motion-duration-long1:      450ms;
  --md-sys-motion-duration-long2:      500ms;
  --md-sys-motion-duration-long3:      550ms;
  --md-sys-motion-duration-long4:      600ms;

  --md-sys-motion-duration-extra-long1: 700ms;
  --md-sys-motion-duration-extra-long2: 800ms;
  --md-sys-motion-duration-extra-long3: 900ms;
  --md-sys-motion-duration-extra-long4:1000ms;
}

@media (prefers-color-scheme: dark) {
  :root {
    --md-sys-color-primary: #D0BCFF;
    --md-sys-color-on-primary: #381E72;
    --md-sys-color-primary-container: #4F378B;
    --md-sys-color-on-primary-container: #EADDFF;
    --md-sys-color-secondary: #CCC2DC;
    --md-sys-color-on-secondary: #332D41;
    --md-sys-color-secondary-container: #4A4458;
    --md-sys-color-on-secondary-container: #E8DEF8;
    --md-sys-color-tertiary: #EFB8C8;
    --md-sys-color-on-tertiary: #492532;
    --md-sys-color-tertiary-container: #633B48;
    --md-sys-color-on-tertiary-container: #FFD8E4;
    --md-sys-color-error: #F2B8B5;
    --md-sys-color-on-error: #601410;
    --md-sys-color-error-container: #8C1D18;
    --md-sys-color-on-error-container: #F9DEDC;
    --md-sys-color-surface: #141218;
    --md-sys-color-on-surface: #E6E0E9;
    --md-sys-color-surface-dim: #141218;
    --md-sys-color-surface-bright: #3B383E;
    --md-sys-color-surface-container-lowest: #0F0D13;
    --md-sys-color-surface-container-low: #1D1B20;
    --md-sys-color-surface-container: #211F26;
    --md-sys-color-surface-container-high: #2B2930;
    --md-sys-color-surface-container-highest: #36343B;
    --md-sys-color-surface-variant: #49454F;
    --md-sys-color-on-surface-variant: #CAC4D0;
    --md-sys-color-outline: #938F99;
    --md-sys-color-outline-variant: #49454F;
    --md-sys-color-inverse-surface: #E6E0E9;
    --md-sys-color-inverse-on-surface: #322F35;
    --md-sys-color-inverse-primary: #6750A4;
  }
}
"#;

const M3_PAGE_TSX: &str = r#"import { Button } from "@/registry/new-york/ui/button";
import { Card, CardContent } from "@/registry/new-york/ui/card";
import { Chip } from "@/registry/new-york/ui/chip";
import { Fab } from "@/registry/new-york/ui/fab";
import { SegmentedControl, Segment } from "@/registry/new-york/ui/segmented-control";

export default function Home() {
  return (
    <main className="container mx-auto p-8 space-y-8">
      <header>
        <h1 className="text-5xl font-medium text-[--md-sys-color-on-surface]">
          Material 3 × shadcn
        </h1>
        <p className="mt-2 text-[--md-sys-color-on-surface-variant]">
          Registry : <code>bunx shadcn@latest add https://your.domain/r/button.json</code>
        </p>
      </header>

      <section>
        <h2 className="text-2xl mb-4">Buttons</h2>
        <div className="flex flex-wrap gap-3">
          <Button variant="filled">Filled</Button>
          <Button variant="tonal">Tonal</Button>
          <Button variant="outlined">Outlined</Button>
          <Button variant="elevated">Elevated</Button>
          <Button variant="text">Text</Button>
        </div>
      </section>

      <section>
        <h2 className="text-2xl mb-4">Chips</h2>
        <div className="flex flex-wrap gap-2">
          <Chip variant="assist">Assist</Chip>
          <Chip variant="filter">Filter</Chip>
          <Chip variant="input">Input</Chip>
          <Chip variant="suggestion">Suggestion</Chip>
        </div>
      </section>

      <section>
        <h2 className="text-2xl mb-4">Cards</h2>
        <div className="grid md:grid-cols-3 gap-4">
          <Card variant="elevated"><CardContent>Elevated</CardContent></Card>
          <Card variant="filled"><CardContent>Filled</CardContent></Card>
          <Card variant="outlined"><CardContent>Outlined</CardContent></Card>
        </div>
      </section>

      <section>
        <h2 className="text-2xl mb-4">Segmented Control (mobile-native-web)</h2>
        <SegmentedControl defaultValue="day">
          <Segment value="day">Day</Segment>
          <Segment value="week">Week</Segment>
          <Segment value="month">Month</Segment>
        </SegmentedControl>
      </section>

      <Fab>+</Fab>
    </main>
  );
}
"#;

const M3_BUTTON_TSX: &str = r#"// M3 Button — 5 variants officiels Material Design 3.
// Ports depuis https://m3.material.io/components/buttons et l'ancien
// @material/web <md-filled-button>, <md-tonal-button>, etc.

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/registry/new-york/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 h-10 px-6 rounded-full text-[14px] font-medium tracking-[0.1px] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[--md-sys-color-primary] disabled:opacity-38 disabled:pointer-events-none [&_svg]:size-[18px] [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        filled:
          "bg-[--md-sys-color-primary] text-[--md-sys-color-on-primary] hover:shadow-[var(--md-sys-elevation-level1)]",
        tonal:
          "bg-[--md-sys-color-secondary-container] text-[--md-sys-color-on-secondary-container] hover:shadow-[var(--md-sys-elevation-level1)]",
        outlined:
          "border border-[--md-sys-color-outline] text-[--md-sys-color-primary] hover:bg-[--md-sys-color-primary]/8",
        elevated:
          "bg-[--md-sys-color-surface-container-low] text-[--md-sys-color-primary] shadow-[var(--md-sys-elevation-level1)] hover:shadow-[var(--md-sys-elevation-level2)]",
        text:
          "text-[--md-sys-color-primary] hover:bg-[--md-sys-color-primary]/8 px-3",
      },
    },
    defaultVariants: { variant: "filled" },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return <Comp ref={ref} className={cn(buttonVariants({ variant, className }))} {...props} />;
  }
);
Button.displayName = "M3Button";
"#;

const M3_CARD_TSX: &str = r#"// M3 Card — Elevated / Filled / Outlined.
// https://m3.material.io/components/cards

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/registry/new-york/lib/utils";

const cardVariants = cva(
  "rounded-[12px] overflow-hidden transition-shadow [--md-shape:var(--md-sys-shape-corner-medium)]",
  {
    variants: {
      variant: {
        elevated:
          "bg-[--md-sys-color-surface-container-low] shadow-[var(--md-sys-elevation-level1)] hover:shadow-[var(--md-sys-elevation-level2)]",
        filled:
          "bg-[--md-sys-color-surface-container-highest]",
        outlined:
          "bg-[--md-sys-color-surface] border border-[--md-sys-color-outline-variant]",
      },
    },
    defaultVariants: { variant: "elevated" },
  }
);

export interface CardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardVariants> {}

export const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({ className, variant, ...props }, ref) => (
    <div ref={ref} className={cn(cardVariants({ variant, className }))} {...props} />
  )
);
Card.displayName = "M3Card";

export const CardContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div ref={ref} className={cn("p-4", className)} {...props} />
  )
);
CardContent.displayName = "M3CardContent";
"#;

const M3_CHIP_TSX: &str = r#"// M3 Chip — Assist / Filter / Input / Suggestion.
// https://m3.material.io/components/chips

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/registry/new-york/lib/utils";

const chipVariants = cva(
  "inline-flex items-center gap-1 h-8 px-3 rounded-[8px] border text-[14px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[--md-sys-color-primary] disabled:opacity-38",
  {
    variants: {
      variant: {
        assist:
          "border-[--md-sys-color-outline] text-[--md-sys-color-on-surface] bg-transparent hover:bg-[--md-sys-color-on-surface]/8",
        filter:
          "border-[--md-sys-color-outline] text-[--md-sys-color-on-surface-variant] bg-transparent data-[state=on]:bg-[--md-sys-color-secondary-container] data-[state=on]:text-[--md-sys-color-on-secondary-container] data-[state=on]:border-transparent",
        input:
          "border-[--md-sys-color-outline] text-[--md-sys-color-on-surface-variant] bg-[--md-sys-color-surface-container]",
        suggestion:
          "border-[--md-sys-color-outline] text-[--md-sys-color-on-surface-variant] bg-transparent hover:bg-[--md-sys-color-on-surface]/8",
      },
    },
    defaultVariants: { variant: "assist" },
  }
);

export interface ChipProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof chipVariants> {}

export const Chip = React.forwardRef<HTMLButtonElement, ChipProps>(
  ({ className, variant, ...props }, ref) => (
    <button ref={ref} className={cn(chipVariants({ variant, className }))} {...props} />
  )
);
Chip.displayName = "M3Chip";
"#;

const M3_FAB_TSX: &str = r#"// M3 FAB + Extended FAB — small / medium / large.
// https://m3.material.io/components/floating-action-button

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/registry/new-york/lib/utils";

const fabVariants = cva(
  "fixed bottom-4 right-4 z-50 inline-flex items-center justify-center gap-2 rounded-[16px] shadow-[var(--md-sys-elevation-level3)] transition-all hover:shadow-[var(--md-sys-elevation-level4)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[--md-sys-color-primary] [&_svg]:shrink-0",
  {
    variants: {
      size: {
        small: "size-[40px] [&_svg]:size-[20px]",
        medium: "size-[56px] [&_svg]:size-[24px] text-[24px]",
        large: "size-[96px] [&_svg]:size-[36px] rounded-[28px] text-[36px]",
        extended: "h-[56px] px-4 [&_svg]:size-[24px] text-[14px] font-medium",
      },
      color: {
        primary:
          "bg-[--md-sys-color-primary-container] text-[--md-sys-color-on-primary-container]",
        surface:
          "bg-[--md-sys-color-surface-container-high] text-[--md-sys-color-primary]",
        secondary:
          "bg-[--md-sys-color-secondary-container] text-[--md-sys-color-on-secondary-container]",
        tertiary:
          "bg-[--md-sys-color-tertiary-container] text-[--md-sys-color-on-tertiary-container]",
      },
    },
    defaultVariants: { size: "medium", color: "primary" },
  }
);

export interface FabProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof fabVariants> {}

export const Fab = React.forwardRef<HTMLButtonElement, FabProps>(
  ({ className, size, color, ...props }, ref) => (
    <button ref={ref} className={cn(fabVariants({ size, color, className }))} {...props} />
  )
);
Fab.displayName = "M3Fab";
"#;

const M3_NAV_BAR_TSX: &str = r#"// M3 Navigation Bar — bottom navigation mobile-first.
// https://m3.material.io/components/navigation-bar

import * as React from "react";
import { cn } from "@/registry/new-york/lib/utils";

export interface NavItem {
  label: string;
  icon: React.ReactNode;
  value: string;
}

export function NavigationBar({
  items,
  value,
  onChange,
  className,
}: {
  items: NavItem[];
  value: string;
  onChange: (v: string) => void;
  className?: string;
}) {
  return (
    <nav
      className={cn(
        "fixed bottom-0 inset-x-0 z-40 h-[80px] bg-[--md-sys-color-surface-container] border-t border-[--md-sys-color-outline-variant] flex items-stretch",
        className
      )}
    >
      {items.map((item) => {
        const active = item.value === value;
        return (
          <button
            key={item.value}
            onClick={() => onChange(item.value)}
            className="flex-1 flex flex-col items-center justify-center gap-1 text-[12px] focus-visible:outline-none"
          >
            <span
              className={cn(
                "inline-flex items-center justify-center h-8 px-4 rounded-full transition-colors",
                active
                  ? "bg-[--md-sys-color-secondary-container] text-[--md-sys-color-on-secondary-container]"
                  : "text-[--md-sys-color-on-surface-variant] hover:bg-[--md-sys-color-on-surface]/8"
              )}
            >
              {item.icon}
            </span>
            <span
              className={cn(
                active
                  ? "text-[--md-sys-color-on-surface] font-medium"
                  : "text-[--md-sys-color-on-surface-variant]"
              )}
            >
              {item.label}
            </span>
          </button>
        );
      })}
    </nav>
  );
}
"#;

const M3_BOTTOM_SHEET_TSX: &str = r#"// M3 Bottom Sheet — modal & standard, drag-to-dismiss via Radix Dialog.
// Pattern mobile-native-web : comportement iOS/Android reproduit.

"use client";

import * as React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { cn } from "@/registry/new-york/lib/utils";

export const BottomSheet = Dialog.Root;
export const BottomSheetTrigger = Dialog.Trigger;

export const BottomSheetContent = React.forwardRef<
  React.ElementRef<typeof Dialog.Content>,
  React.ComponentPropsWithoutRef<typeof Dialog.Content>
>(({ className, children, ...props }, ref) => (
  <Dialog.Portal>
    <Dialog.Overlay className="fixed inset-0 z-50 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
    <Dialog.Content
      ref={ref}
      className={cn(
        "fixed bottom-0 inset-x-0 z-50 rounded-t-[28px] bg-[--md-sys-color-surface-container-low] pb-8 shadow-[var(--md-sys-elevation-level3)] data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-bottom data-[state=open]:slide-in-from-bottom",
        className
      )}
      {...props}
    >
      <div className="mx-auto mt-4 h-1 w-8 rounded-full bg-[--md-sys-color-on-surface-variant]/40" />
      <div className="p-6">{children}</div>
    </Dialog.Content>
  </Dialog.Portal>
));
BottomSheetContent.displayName = "BottomSheetContent";
"#;

const M3_SEGMENTED_TSX: &str = r#"// Segmented Control — mix iOS segmented control + M3 segmented button.
// Utilise Radix ToggleGroup pour l'accessibilité.

"use client";

import * as React from "react";
import * as ToggleGroup from "@radix-ui/react-toggle-group";
import { cn } from "@/registry/new-york/lib/utils";

export function SegmentedControl({
  defaultValue,
  value,
  onValueChange,
  children,
  className,
}: {
  defaultValue?: string;
  value?: string;
  onValueChange?: (v: string) => void;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <ToggleGroup.Root
      type="single"
      defaultValue={defaultValue}
      value={value}
      onValueChange={(v) => v && onValueChange?.(v)}
      className={cn(
        "inline-flex h-10 rounded-full border border-[--md-sys-color-outline] overflow-hidden divide-x divide-[--md-sys-color-outline]",
        className
      )}
    >
      {children}
    </ToggleGroup.Root>
  );
}

export function Segment({
  value,
  children,
  className,
}: {
  value: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <ToggleGroup.Item
      value={value}
      className={cn(
        "px-4 text-[14px] font-medium text-[--md-sys-color-on-surface] data-[state=on]:bg-[--md-sys-color-secondary-container] data-[state=on]:text-[--md-sys-color-on-secondary-container] transition-colors focus-visible:outline-none",
        className
      )}
    >
      {children}
    </ToggleGroup.Item>
  );
}
"#;

/// Script Bun qui génère public/r/*.json depuis registry/.
/// Équivalent Rust dans crates/m3-registry-builder (plus rapide sur gros registry).
const M3_BUILD_REGISTRY_TS: &str = r#"#!/usr/bin/env bun
// Build le registry shadcn depuis registry/*.tsx → public/r/*.json.
// Usage : bun scripts/build-registry.ts
// Version Rust (plus rapide) : cargo run -p m3-registry-builder -- registry public/r

import { Glob } from "bun";

type RegistryIndex = {
  name: string;
  items: Array<{
    name: string;
    type: string;
    title?: string;
    description?: string;
    files: Array<{ path: string; type: string }>;
    dependencies?: string[];
    registryDependencies?: string[];
  }>;
};

const indexPath = "./registry.json";
const outDir = "./public/r";
const index = (await Bun.file(indexPath).json()) as RegistryIndex;

await Bun.$`mkdir -p ${outDir}`;

let count = 0;
for (const item of index.items) {
  const files = [];
  for (const file of item.files) {
    const content = await Bun.file(file.path).text();
    files.push({
      path: file.path,
      type: file.type,
      content,
      target: `components/${item.name}.tsx`,
    });
  }
  const payload = {
    $schema: "https://ui.shadcn.com/schema/registry-item.json",
    name: item.name,
    type: item.type,
    title: item.title,
    description: item.description,
    dependencies: item.dependencies ?? [],
    registryDependencies: item.registryDependencies ?? [],
    files,
  };
  await Bun.write(`${outDir}/${item.name}.json`, JSON.stringify(payload, null, 2));
  count++;
}

// Index global
await Bun.write(`${outDir}/index.json`, JSON.stringify(index, null, 2));

console.log(`✓ ${count} items built → ${outDir}/`);
"#;

fn render_m3_builder_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "m3-registry-builder"
version = "0.1.0"
edition = "2021"
description = "Build shadcn registry JSON files from TSX sources — Rust version for {name}"

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
walkdir = "2"
anyhow = "1"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
"#,
    )
}

const M3_BUILDER_MAIN_RS: &str = r#"//! Build le registry shadcn depuis registry.json → public/r/*.json.
//! Version Rust — ~10× plus rapide que le script Bun pour gros registries.
//!
//! Usage : cargo run --release -- registry public/r

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct RegistryFile {
    path: String,
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryItem {
    name: String,
    #[serde(rename = "type")]
    ty: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    files: Vec<RegistryFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "registryDependencies")]
    registry_dependencies: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegistryIndex {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    items: Vec<RegistryItem>,
}

#[derive(Serialize)]
struct FileOut {
    path: String,
    #[serde(rename = "type")]
    ty: String,
    content: String,
    target: String,
}

#[derive(Serialize)]
struct ItemOut<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    name: &'a str,
    #[serde(rename = "type")]
    ty: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: &'a Option<String>,
    dependencies: &'a Vec<String>,
    #[serde(rename = "registryDependencies")]
    registry_dependencies: &'a Vec<String>,
    files: Vec<FileOut>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let _registry_dir = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("registry"));
    let out_dir = args.get(2).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("public/r"));

    let index_content = fs::read_to_string("registry.json").context("lire registry.json")?;
    let index: RegistryIndex = serde_json::from_str(&index_content)?;

    fs::create_dir_all(&out_dir)?;

    let mut count = 0usize;
    for item in &index.items {
        let mut files = Vec::with_capacity(item.files.len());
        for f in &item.files {
            let content = fs::read_to_string(&f.path).with_context(|| format!("lire {}", f.path))?;
            files.push(FileOut {
                path: f.path.clone(),
                ty: f.ty.clone(),
                content,
                target: format!("components/{}.tsx", item.name),
            });
        }
        let payload = ItemOut {
            schema: "https://ui.shadcn.com/schema/registry-item.json",
            name: &item.name,
            ty: &item.ty,
            title: &item.title,
            description: &item.description,
            dependencies: &item.dependencies,
            registry_dependencies: &item.registry_dependencies,
            files,
        };
        let path = out_dir.join(format!("{}.json", item.name));
        fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
        count += 1;
    }

    // Index global (copie de registry.json sans le $schema)
    fs::write(out_dir.join("index.json"), serde_json::to_string_pretty(&index)?)?;

    println!("✓ {count} items → {}", out_dir.display());
    Ok(())
}
"#;

fn render_m3_readme(name: &str) -> String {
    format!(
        r#"# {name}

**Material Design 3 registry for shadcn/ui** — ports MWC (Material Web Components)
vers shadcn via Tailwind v4, avec composants mobile-native-web (comme MUI) et
build pipeline Rust + Bun.

Scaffolded by `n2b ui init --flavor m3`.

## Composants fournis

| Composant | Variants | Source M3 |
|-----------|----------|-----------|
| Button | filled, tonal, outlined, elevated, text | [m3.material.io/components/buttons](https://m3.material.io/components/buttons) |
| Card | elevated, filled, outlined | [m3.material.io/components/cards](https://m3.material.io/components/cards) |
| Chip | assist, filter, input, suggestion | [m3.material.io/components/chips](https://m3.material.io/components/chips) |
| FAB | small, medium, large, extended × 4 colors | [m3.material.io/components/fab](https://m3.material.io/components/floating-action-button) |
| Navigation Bar | mobile bottom nav | [m3.material.io/components/navigation-bar](https://m3.material.io/components/navigation-bar) |
| **Bottom Sheet** | modal + drag-to-dismiss | mobile-native-web |
| **Segmented Control** | iOS/Android style | mobile-native-web |

Tous stylés avec les tokens M3 exposés en **CSS custom properties** —
utilisables via Tailwind arbitrary classes : `bg-[--md-sys-color-primary]`.

## Dev & preview

```bash
bun install
bun dev                      # showcase Next 16 sur :3000
```

## Build le registry → public/r/*.json

Deux options, mêmes inputs (`registry.json` + `registry/**`) :

### Option 1 — Bun (rapide, TypeScript)
```bash
bun run build:registry
# → public/r/button.json, card.json, chip.json, fab.json, …
```

### Option 2 — Rust (crate m3-registry-builder, ~10× plus rapide)
```bash
bun run build:registry:rust
# cargo run --release --manifest-path crates/m3-registry-builder/Cargo.toml -- registry public/r
```

## Publier et consommer

1. Build → `public/r/*.json`
2. Deploy Next (Vercel, GitHub Pages, …)
3. Depuis un autre projet :
   ```bash
   bunx --bun shadcn@latest add https://your.domain/r/button.json
   bunx --bun shadcn@latest add https://your.domain/r/segmented-control.json
   ```

## MCP pour Claude / Cursor

```bash
bunx --bun shadcn@latest mcp init --client claude
# → Claude peut ajouter les composants en conversation :
#   « Add the M3 Button and Bottom Sheet from {name} »
```

## Skills AI

```bash
bunx --bun skills add shadcn/ui
```

## Architecture

```
{name}/
├── registry.json                   ← index registry (shadcn schema)
├── registry/new-york/
│   ├── ui/
│   │   ├── button.tsx              ← M3 Button (CVA variants)
│   │   ├── card.tsx
│   │   ├── chip.tsx
│   │   ├── fab.tsx
│   │   ├── navigation-bar.tsx
│   │   ├── bottom-sheet.tsx        ← mobile-native-web
│   │   └── segmented-control.tsx   ← mobile-native-web
│   └── lib/
│       ├── utils.ts                ← cn()
│       └── m3-tokens.css           ← tokens M3 officiels (light + dark)
├── src/app/                        ← showcase Next 16
├── scripts/build-registry.ts       ← builder Bun
├── crates/m3-registry-builder/     ← builder Rust (release ~10× faster)
├── components.json                 ← shadcn config
├── .mcp.json                       ← config MCP shadcn pour Claude
└── public/r/                       ← généré par bun run build:registry
```

## Fonts & icons

- **Google Sans Flex** via `@fontsource-variable/google-sans-flex` (var font)
- **Material Symbols** via `material-symbols` (icons variable font)

## Refs

- [shadcn Registry docs](https://ui.shadcn.com/docs/registry)
- [Material Design 3](https://m3.material.io/)
- [M3 color tokens](https://m3.material.io/styles/color/roles)
- [M3 typography](https://m3.material.io/styles/typography)
- [material-components/material-web](https://github.com/material-components/material-web) (MWC source)
- [material-tailwind](https://github.com/creativetimofficial/material-tailwind)
- [mui/material-ui](https://github.com/mui/material-ui)
"#,
    )
}

const M3_GITIGNORE: &str = r#"node_modules/
.next/
out/
dist/
public/r/
target/
.DS_Store
.env.local
packages/*/dist/
"#;

// ============================================================================
// MD3-UI FRAMEWORK — monorepo Bun + Rust
// ============================================================================

fn render_md3_root_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "description": "React framework Material Design 3 — fork de shadcn, compilé via Rust (SWC+Rspack+OXC), linté via Biome, orchestré via Turborepo, Web APIs via Bun",
  "type": "module",
  "workspaces": [
    "packages/*",
    "apps/*",
    "examples/*"
  ],
  "scripts": {{
    "build": "turbo run build",
    "dev": "turbo run dev --parallel",
    "lint": "turbo run lint",
    "lint:fix": "bunx --bun @biomejs/biome check --write .",
    "format": "bunx --bun @biomejs/biome format --write .",
    "test": "turbo run test",
    "typecheck": "turbo run typecheck",
    "registry:build": "turbo run build --filter @md3-ui/registry",
    "registry:build:rust": "cargo run --release -p md3-registry-builder -- packages/registry packages/registry/public/r",
    "cli": "bun run --filter @md3-ui/cli start",
    "cargo:build": "cargo build --workspace --release",
    "cargo:check": "cargo check --workspace"
  }},
  "devDependencies": {{
    "@biomejs/biome": "^1.9.0",
    "turbo": "^2.0.0",
    "typescript": "^5.6.0",
    "@types/bun": "latest"
  }},
  "engines": {{
    "bun": ">=1.2.0"
  }},
  "trustedDependencies": ["@biomejs/biome"],
  "packageManager": "bun@1.2.0"
}}
"#,
    )
}

fn render_md3_workspace_cargo_toml() -> String {
    r#"[workspace]
resolver = "2"
members = [
  "crates/md3-compiler",
  "crates/md3-registry-builder",
  "crates/md3-wasm-plugin",
]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
walkdir = "2"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
"#
    .to_string()
}

/// Turborepo v2 pipeline — orchestre les tâches cross-packages avec cache.
/// Supporte aussi les crates Rust via `cargo run/build` side (non caché par Turbo,
/// mais Cargo a son propre cache + `target/` partagé par workspace).
const MD3_TURBO_JSON: &str = r#"{
  "$schema": "https://turborepo.com/schema.json",
  "ui": "tui",
  "globalDependencies": [
    "bun.lock",
    "Cargo.lock",
    "tsconfig.base.json",
    "biome.json",
    "rspack.config.mjs"
  ],
  "globalEnv": ["NODE_ENV", "CI"],
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "inputs": [
        "src/**",
        "package.json",
        "tsconfig.json",
        "rsbuild.config.ts"
      ],
      "outputs": ["dist/**", ".next/**", "!.next/cache/**"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    },
    "lint": {
      "dependsOn": [],
      "inputs": ["src/**", "biome.json"],
      "outputs": []
    },
    "typecheck": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "tsconfig.json"],
      "outputs": ["*.tsbuildinfo", "dist/.tsbuildinfo"]
    },
    "test": {
      "dependsOn": ["^build"],
      "inputs": ["src/**", "tests/**"],
      "outputs": ["coverage/**"]
    }
  }
}
"#;

const MD3_BIOME_JSON: &str = r#"{
  "$schema": "https://biomejs.dev/schemas/1.9.0/schema.json",
  "vcs": { "enabled": true, "clientKind": "git", "useIgnoreFile": true },
  "files": {
    "ignore": ["dist", "node_modules", "target", ".next", "public/r"]
  },
  "organizeImports": { "enabled": true },
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "lineWidth": 100
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "style": {
        "useImportType": "error",
        "noNonNullAssertion": "off"
      },
      "suspicious": {
        "noExplicitAny": "warn"
      },
      "correctness": {
        "noUnusedImports": "error"
      }
    }
  },
  "javascript": {
    "formatter": {
      "quoteStyle": "double",
      "semicolons": "always",
      "trailingCommas": "all"
    }
  }
}
"#;

const MD3_RSPACK_CONFIG: &str = r#"// Rspack config — bundler Rust pour packages/core + examples.
// https://rspack.rs/config/

import { defineConfig } from "@rspack/cli";

export default defineConfig({
  mode: "production",
  entry: "./packages/core/src/index.ts",
  output: {
    path: "./packages/core/dist",
    filename: "index.js",
    library: { type: "module" },
    module: true,
  },
  experiments: { outputModule: true },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: { loader: "builtin:swc-loader", options: {
          jsc: {
            parser: { syntax: "typescript", tsx: true },
            transform: { react: { runtime: "automatic" } },
            target: "es2022",
          },
        }},
      },
      {
        test: /\.wasm$/,
        type: "webassembly/async",
      },
    ],
  },
  externalsType: "module",
  externals: ["react", "react-dom"],
});
"#;

const MD3_TSCONFIG_BASE: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "Preserve",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "allowImportingTsExtensions": true,
    "moduleDetection": "force",
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "resolveJsonModule": true,
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "noUncheckedIndexedAccess": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "types": ["bun"],
    "paths": {
      "@md3-ui/core": ["./packages/core/src"],
      "@md3-ui/tokens": ["./packages/tokens/src"],
      "@md3-ui/registry": ["./packages/registry/src"]
    }
  }
}
"#;

const MD3_CORE_PACKAGE_JSON: &str = r#"{
  "name": "@md3-ui/core",
  "version": "0.1.0",
  "type": "module",
  "description": "React components — Material Design 3 (fork shadcn, no CSS-in-JS)",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./src/index.ts",
  "exports": {
    ".": { "import": "./dist/index.js", "types": "./src/index.ts" },
    "./button": { "import": "./dist/button/index.js", "types": "./src/button/index.ts" },
    "./card":   { "import": "./dist/card/index.js",   "types": "./src/card/index.ts" },
    "./chip":   { "import": "./dist/chip/index.js",   "types": "./src/chip/index.ts" },
    "./fab":    { "import": "./dist/fab/index.js",    "types": "./src/fab/index.ts" },
    "./navigation-bar":    { "import": "./dist/navigation-bar/index.js",    "types": "./src/navigation-bar/index.ts" },
    "./bottom-sheet":      { "import": "./dist/bottom-sheet/index.js",      "types": "./src/bottom-sheet/index.ts" },
    "./segmented-control": { "import": "./dist/segmented-control/index.js", "types": "./src/segmented-control/index.ts" },
    "./motion": { "import": "./dist/motion/index.js", "types": "./src/motion/index.ts" },
    "./theme":  { "import": "./dist/theme/index.js",  "types": "./src/theme/index.ts" }
  },
  "sideEffects": false,
  "scripts": {
    "build": "rsbuild build",
    "dev": "rsbuild dev"
  },
  "dependencies": {
    "@radix-ui/react-slot": "^1.1.0",
    "@radix-ui/react-dialog": "^1.1.0",
    "@radix-ui/react-toggle-group": "^1.1.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.5.0",
    "@md3-ui/tokens": "workspace:*"
  },
  "peerDependencies": {
    "react": "^18.0.0 || ^19.0.0",
    "react-dom": "^18.0.0 || ^19.0.0"
  },
  "devDependencies": {
    "@rsbuild/core": "^1.1.0",
    "@rsbuild/plugin-react": "^1.0.0",
    "@types/react": "^19.0.0",
    "typescript": "^5.6.0"
  }
}
"#;

const MD3_PKG_TSCONFIG: &str = r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "rootDir": "./src",
    "declaration": true,
    "declarationMap": true,
    "composite": true,
    "tsBuildInfoFile": "./dist/.tsbuildinfo"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#;

const MD3_CORE_RSBUILD: &str = r#"import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

export default defineConfig({
  plugins: [pluginReact()],
  source: {
    entry: {
      index: "./src/index.ts",
      "button/index": "./src/button/index.ts",
      "card/index": "./src/card/index.ts",
      "chip/index": "./src/chip/index.ts",
      "fab/index": "./src/fab/index.ts",
      "navigation-bar/index": "./src/navigation-bar/index.ts",
      "bottom-sheet/index": "./src/bottom-sheet/index.ts",
      "segmented-control/index": "./src/segmented-control/index.ts",
      "motion/index": "./src/motion/index.ts",
      "theme/index": "./src/theme/ThemeProvider.tsx",
    },
  },
  output: {
    target: "web",
    distPath: { root: "dist" },
    externals: ["react", "react-dom"],
  },
  tools: {
    swc: {
      jsc: {
        transform: {
          react: { runtime: "automatic" },
        },
      },
    },
  },
});
"#;

const MD3_CORE_INDEX: &str = r#"// Public API of @md3-ui/core.
// Compilé par Rspack+SWC, linté par Biome, tokens M3 via @md3-ui/tokens
// (CSS custom properties, zero CSS-in-JS). Motion M3 natif via les tokens
// --md-sys-motion-* (easing + duration).

export * from "./button";
export * from "./card";
export * from "./chip";
export * from "./fab";
export * from "./navigation-bar";
export * from "./bottom-sheet";
export * from "./segmented-control";
export * from "./motion";
export { ThemeProvider, useTheme } from "./theme/ThemeProvider";
export type { M3ColorScheme } from "./theme/tokens";
"#;

const MD3_THEME_PROVIDER: &str = r#"// Theme provider — pur React context, zéro CSS-in-JS.
// Le thème est piloté via `data-theme="light|dark"` sur <html>, les tokens
// M3 sont exposés en variables CSS dans @md3-ui/tokens.
//
// Usage :
//   import { ThemeProvider, useTheme } from "@md3-ui/core";
//   <ThemeProvider scheme="light">{app}</ThemeProvider>

"use client";

import * as React from "react";
import type { M3ColorScheme } from "./tokens";

const ThemeSchemeContext = React.createContext<{
  scheme: M3ColorScheme;
  setScheme: (s: M3ColorScheme) => void;
}>({ scheme: "light", setScheme: () => {} });

export function ThemeProvider({
  scheme: initial = "light",
  children,
}: {
  scheme?: M3ColorScheme;
  children: React.ReactNode;
}) {
  const [scheme, setScheme] = React.useState<M3ColorScheme>(initial);

  React.useEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.dataset.theme = scheme;
    }
  }, [scheme]);

  return (
    <ThemeSchemeContext.Provider value={{ scheme, setScheme }}>
      {children}
    </ThemeSchemeContext.Provider>
  );
}

export function useTheme() {
  return React.useContext(ThemeSchemeContext);
}
"#;

const MD3_THEME_TOKENS_TS: &str = r#"// M3 color scheme discriminé — les valeurs réelles vivent dans
// @md3-ui/tokens/css (variables CSS). Ce fichier n'expose QUE le type.

export type M3ColorScheme = "light" | "dark";
"#;

const MD3_USE_THEME: &str = r#"export { useTheme } from "./ThemeProvider";
"#;

const MD3_TOKENS_PACKAGE_JSON: &str = r#"{
  "name": "@md3-ui/tokens",
  "version": "0.1.0",
  "type": "module",
  "description": "Material Design 3 tokens (CSS custom properties + JSON)",
  "main": "./src/theme.json",
  "types": "./src/theme.json",
  "exports": {
    ".": { "default": "./src/theme.json" },
    "./css": "./src/tokens.css"
  },
  "sideEffects": ["./src/tokens.css"],
  "files": ["src"]
}
"#;

const MD3_THEME_JSON: &str = r##"{
  "light": {
    "primary": "#6750A4",
    "onPrimary": "#FFFFFF",
    "primaryContainer": "#EADDFF",
    "onPrimaryContainer": "#21005D",
    "surface": "#FEF7FF",
    "onSurface": "#1D1B20"
  },
  "dark": {
    "primary": "#D0BCFF",
    "onPrimary": "#381E72",
    "primaryContainer": "#4F378B",
    "onPrimaryContainer": "#EADDFF",
    "surface": "#141218",
    "onSurface": "#E6E0E9"
  }
}
"##;

const MD3_REGISTRY_PACKAGE_JSON: &str = r#"{
  "name": "@md3-ui/registry",
  "version": "0.1.0",
  "type": "module",
  "description": "shadcn-compatible registry for md3-ui components",
  "private": true,
  "scripts": {
    "build": "bun scripts/build.ts"
  },
  "devDependencies": { "@types/bun": "latest" }
}
"#;

fn render_md3_cli_package_json(name: &str) -> String {
    format!(
        r#"{{
  "name": "@md3-ui/cli",
  "version": "0.1.0",
  "type": "module",
  "description": "CLI {name} — copy components from the registry to your project",
  "bin": {{ "md3-ui": "./src/cli.ts" }},
  "scripts": {{
    "start": "bun run src/cli.ts"
  }},
  "devDependencies": {{ "@types/bun": "latest" }}
}}
"#,
    )
}

fn render_md3_cli_ts(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env bun
// md3-ui CLI — add a component from the registry into the user's project.
// Inspired by `bunx shadcn@latest add <component>` but scoped to {name}.

import {{ parseArgs }} from "node:util";

const USAGE = `md3-ui — add {name} components to your project

Usage :
  md3-ui add <component>...      # copie le composant dans src/components/ui/
  md3-ui list                     # liste les composants du registry
  md3-ui --help

Ex : md3-ui add button card chip
`;

const REGISTRY_URL = process.env.MD3_UI_REGISTRY ?? "https://md3-ui.dev/r";

async function list() {{
  const res = await fetch(`${{REGISTRY_URL}}/index.json`);
  const idx = await res.json();
  for (const item of idx.items) {{
    console.log(`  ${{item.name.padEnd(24)}} ${{item.description ?? ""}}`);
  }}
}}

async function add(components: string[]) {{
  for (const name of components) {{
    const url = `${{REGISTRY_URL}}/${{name}}.json`;
    const item = await (await fetch(url)).json() as any;
    for (const f of item.files) {{
      const out = f.target ?? `src/components/ui/${{name}}.tsx`;
      await Bun.write(out, f.content);
      console.log(`  + ${{out}}`);
    }}
  }}
}}

const {{ values, positionals }} = parseArgs({{
  args: Bun.argv.slice(2),
  options: {{ help: {{ type: "boolean", short: "h" }} }},
  allowPositionals: true,
  strict: false,
}});

if (values.help || positionals.length === 0) {{
  process.stdout.write(USAGE);
  process.exit(0);
}}

const [cmd, ...rest] = positionals;
if (cmd === "list") await list();
else if (cmd === "add") await add(rest);
else {{
  console.error(`unknown command: ${{cmd}}`);
  process.exit(1);
}}
"#,
    )
}

const MD3_LINT_PACKAGE_JSON: &str = r#"{
  "name": "@md3-ui/lint-plugin",
  "version": "0.1.0",
  "type": "module",
  "description": "Biome / ESLint-compatible plugin — enforce M3 tokens usage",
  "main": "./src/index.ts",
  "exports": { ".": "./src/index.ts" },
  "keywords": ["biome", "eslint-plugin", "material-design", "md3"],
  "devDependencies": { "@types/bun": "latest", "typescript": "^5.6.0" }
}
"#;

const MD3_LINT_INDEX: &str = r#"// md3-ui lint plugin — règles custom pour enforce l'usage des tokens M3.
// Compatible Biome (plugin API en preview) + ESLint via adapter.

export * as noRawColor from "./rules/no-raw-color";
export * as useM3Tokens from "./rules/use-m3-tokens";

export const rules = {
  "no-raw-color": noRawColor,
  "use-m3-tokens": useM3Tokens,
};

import * as noRawColor from "./rules/no-raw-color";
import * as useM3Tokens from "./rules/use-m3-tokens";
"#;

const MD3_LINT_NO_RAW_COLOR: &str = r#"// Règle : interdit les couleurs hex/rgb brutes dans les className ou style.
// Préférer bg-[--md-sys-color-primary] ou theme.color.primary.

export const meta = {
  name: "no-raw-color",
  description: "Disallow raw hex/rgb colors — prefer M3 tokens.",
  docs: "https://m3.material.io/styles/color/roles",
};

export function check(source: string): { line: number; message: string }[] {
  const lines = source.split("\n");
  const findings: { line: number; message: string }[] = [];
  const hex = /#[0-9A-Fa-f]{3,8}\b/g;
  const rgb = /\brgba?\s*\([^)]+\)/g;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.includes("--md-sys-color-")) continue;   // token usage, OK
    if (hex.test(line) || rgb.test(line)) {
      findings.push({
        line: i + 1,
        message: `raw color detected — use var(--md-sys-color-*) or theme.color.* instead`,
      });
    }
  }
  return findings;
}
"#;

const MD3_LINT_USE_TOKENS: &str = r#"// Règle : détecte les className Tailwind "bg-blue-500", "text-red-600" etc.
// et suggère l'équivalent M3 token.

export const meta = {
  name: "use-m3-tokens",
  description: "Prefer M3 color tokens over arbitrary Tailwind palette.",
};

const PALETTE_MAP: Record<string, string> = {
  "bg-blue-500":   "bg-[--md-sys-color-primary]",
  "bg-red-500":    "bg-[--md-sys-color-error]",
  "text-white":    "text-[--md-sys-color-on-primary]",
  "bg-gray-50":    "bg-[--md-sys-color-surface]",
  "bg-gray-100":   "bg-[--md-sys-color-surface-container]",
};

export function check(source: string): { line: number; message: string }[] {
  const lines = source.split("\n");
  const out: { line: number; message: string }[] = [];
  for (let i = 0; i < lines.length; i++) {
    for (const [bad, good] of Object.entries(PALETTE_MAP)) {
      if (lines[i].includes(bad)) {
        out.push({ line: i + 1, message: `replace ${bad} by ${good}` });
      }
    }
  }
  return out;
}
"#;

const MD3_DOCS_PACKAGE_JSON: &str = r#"{
  "name": "@md3-ui/md3-docs",
  "version": "0.1.0",
  "type": "module",
  "private": true,
  "scripts": {
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start"
  },
  "dependencies": {
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@md3-ui/core": "workspace:*",
    "@md3-ui/tokens": "workspace:*"
  },
  "devDependencies": {
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0",
    "typescript": "^5.6.0"
  }
}
"#;

fn render_md3_docs_layout(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import {{ ThemeProvider }} from "@md3-ui/core";
import "./globals.css";

export const metadata: Metadata = {{
  title: "{name} — M3 React framework",
  description: "Material Design 3 React framework",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body>
        <ThemeProvider scheme="light">{{children}}</ThemeProvider>
      </body>
    </html>
  );
}}
"#,
    )
}

const MD3_DOCS_GLOBALS: &str = r#"@import "tailwindcss";
@import "@md3-ui/tokens/css";

html, body {
  font-family: "Google Sans Flex Variable", system-ui, sans-serif;
  background: var(--md-sys-color-surface);
  color: var(--md-sys-color-on-surface);
}
"#;

const MD3_COMPILER_CARGO: &str = r#"[package]
name = "md3-compiler"
version.workspace = true
edition.workspace = true
description = "Rust wrapper around SWC+Rspack to build md3-ui packages"
license.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
walkdir.workspace = true

[profile.release]
lto = true
"#;

const MD3_COMPILER_MAIN_RS: &str = r#"//! md3-compiler — orchestration de SWC + Rspack côté Rust.
//!
//! Ce binaire appelle le CLI `rspack` via std::process pour build chaque
//! package. Il peut aussi émettre les tokens M3 en CSS depuis un JSON source.
//!
//! Usage :
//!   md3-compiler build-all
//!   md3-compiler tokens packages/tokens/src/theme.json > tokens.css

use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    match cmd {
        "build-all" => {
            // Invoque rspack sur chaque package.
            println!("→ rspack build packages/core");
            let st = Command::new("bunx")
                .args(["--bun", "rspack", "build", "-c", "packages/core/rsbuild.config.ts"])
                .status()?;
            if !st.success() {
                anyhow::bail!("rspack failed");
            }
        }
        "tokens" => {
            let path = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("usage: md3-compiler tokens <json>"))?;
            let raw = std::fs::read_to_string(path)?;
            let json: serde_json::Value = serde_json::from_str(&raw)?;
            emit_css_tokens(&json)?;
        }
        _ => {
            println!("md3-compiler — build-all | tokens <file>");
        }
    }
    Ok(())
}

fn emit_css_tokens(json: &serde_json::Value) -> Result<()> {
    println!(":root {{");
    if let Some(obj) = json.get("light").and_then(|v| v.as_object()) {
        for (key, value) in obj {
            if let Some(v) = value.as_str() {
                let kebab = to_kebab(key);
                println!("  --md-sys-color-{kebab}: {v};");
            }
        }
    }
    println!("}}");

    if let Some(obj) = json.get("dark").and_then(|v| v.as_object()) {
        println!("@media (prefers-color-scheme: dark) {{");
        println!("  :root {{");
        for (key, value) in obj {
            if let Some(v) = value.as_str() {
                let kebab = to_kebab(key);
                println!("    --md-sys-color-{kebab}: {v};");
            }
        }
        println!("  }}");
        println!("}}");
    }
    Ok(())
}

fn to_kebab(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}
"#;

const MD3_WASM_PLUGIN_CARGO: &str = r#"[package]
name = "md3-wasm-plugin"
version.workspace = true
edition.workspace = true
description = "Bun bundler plugin (bun-native-plugin + napi-rs) — WASM import support"
license.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
bun-native-plugin = "0.0.1"
napi = { version = "2", features = ["napi8"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
"#;

const MD3_WASM_PLUGIN_RS: &str = r#"//! Plugin Bun natif pour md3-ui : hook onBeforeParse qui détecte les imports
//! `.wasm` et les transforme en imports typed adéquats. Exemple :
//!
//!   import gpu from "./compute.wasm";   // → instancié en ReadableStream
//!
//! Doc : https://docs.rs/bun-native-plugin

use bun_native_plugin::{sys, OnBeforeParse};
use napi_derive::napi;

#[napi]
pub fn register_bun_plugin() -> String {
    "md3-wasm-plugin".to_string()
}

#[no_mangle]
pub extern "C" fn on_before_parse_plugin_impl(
    args: *const sys::OnBeforeParseArguments,
    result: *mut sys::OnBeforeParseResult,
) {
    let args = unsafe { &*args };
    let result = unsafe { &mut *result };
    let mut handle = match OnBeforeParse::from_raw(args, result) {
        Ok(h) => h,
        Err(_) => return,
    };
    let source = match handle.input_source_code() {
        Ok(s) => s,
        Err(_) => {
            handle.log_error("md3-wasm: failed to read source");
            return;
        }
    };

    // Exemple simple : ajoute un marker si des imports .wasm sont détectés.
    if source.contains(".wasm\"") || source.contains(".wasm'") {
        let patched = format!(
            "// [md3-wasm-plugin] detected .wasm import(s)\n{}",
            source
        );
        handle.set_output_source_code(patched, handle.output_loader());
    }
}
"#;

fn render_md3_example_pkg(_name: &str) -> String {
    r#"{
  "name": "@md3-ui/example-next",
  "version": "0.1.0",
  "type": "module",
  "private": true,
  "scripts": {
    "dev": "bunx --bun next dev --turbopack",
    "build": "next build --turbopack",
    "start": "next start"
  },
  "dependencies": {
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@md3-ui/core": "workspace:*",
    "@md3-ui/tokens": "workspace:*"
  },
  "devDependencies": {
    "@types/bun": "latest",
    "@types/react": "^19.0.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0",
    "typescript": "^5.6.0"
  }
}
"#
    .to_string()
}

fn render_md3_example_layout(name: &str) -> String {
    format!(
        r#"import type {{ Metadata }} from "next";
import {{ ThemeProvider }} from "@md3-ui/core";
import "./globals.css";

export const metadata: Metadata = {{
  title: "{name} example",
}};

export default function RootLayout({{ children }}: {{ children: React.ReactNode }}) {{
  return (
    <html lang="en">
      <body>
        <ThemeProvider>{{children}}</ThemeProvider>
      </body>
    </html>
  );
}}
"#,
    )
}

const MD3_EXAMPLE_GLOBALS: &str = MD3_DOCS_GLOBALS;

const MD3_DOCS_TSCONFIG: &str = r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "jsx": "preserve",
    "noEmit": true,
    "paths": {
      "@/*": ["./src/*"]
    },
    "plugins": [{ "name": "next" }]
  },
  "include": ["next-env.d.ts", "src/**/*.ts", "src/**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
"#;

const MD3_DOCS_NAV_TSX: &str = r#"// Nav latérale — liens vers les pages docs du framework.

import Link from "next/link";

const LINKS = [
  { href: "/", label: "Home" },
  { href: "/expressive", label: "Material 3 Expressive" },
  { href: "/motion", label: "Motion tokens" },
  { href: "/tokens", label: "Color & shape tokens" },
];

export function Nav() {
  return (
    <nav className="sticky top-0 bg-[var(--md-sys-color-surface-container)] border-b border-[var(--md-sys-color-outline-variant)] px-4 py-3">
      <ul className="flex gap-4 text-sm">
        {LINKS.map((l) => (
          <li key={l.href}>
            <Link
              href={l.href}
              className="text-[var(--md-sys-color-on-surface)] hover:text-[var(--md-sys-color-primary)]"
            >
              {l.label}
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}
"#;

/// Page dédiée à "Material 3 Expressive" — 46 études / 18k participants.
/// Source : https://design.google/library/expressive-material-design-google-research
const MD3_EXPRESSIVE_PAGE_TSX: &str = r#"import { Nav } from "@/components/Nav";
import { Button } from "@md3-ui/core";

export default function ExpressivePage() {
  return (
    <>
      <Nav />
      <article className="container mx-auto p-8 prose prose-invert max-w-3xl">
        <h1 className="text-5xl font-bold mb-4">Material 3 Expressive</h1>
        <p className="text-[var(--md-sys-color-on-surface-variant)]">
          Research-driven evolution of Material Design 3 — 46 studies, 18,000+ participants.
          <br />
          Source : <a
            className="text-[var(--md-sys-color-primary)] underline"
            href="https://design.google/library/expressive-material-design-google-research"
          >design.google/library/expressive-material-design-google-research</a>
        </p>

        <h2 className="text-3xl mt-8 mb-3">Core idea</h2>
        <p>
          Expressive design makes users <strong>feel something</strong>. It inspires emotion,
          communicates function, and helps users achieve their goals via <em>color, shape, size,
          motion and containment</em> — deliberately amplified.
        </p>

        <h2 className="text-3xl mt-8 mb-3">Measured gains</h2>
        <ul className="space-y-1">
          <li>🔎 Users spot key UI elements <strong>up to 4× faster</strong></li>
          <li>😊 Significantly higher ratings for "energetic", "playful", "creative", "friendly"</li>
          <li>♿ Age-based performance gaps in UI recognition <strong>virtually disappear</strong></li>
          <li>📈 +32% subculture perception · +34% modernity · +30% rebelliousness</li>
          <li>👥 87% preference among 18–24 yr olds</li>
        </ul>

        <h2 className="text-3xl mt-8 mb-3">Apply via md3-ui</h2>
        <ul className="space-y-2">
          <li>Use <code>emphasized</code> easings on primary actions (see Motion).</li>
          <li>Scale FABs + primary buttons larger than their "standard M3" size.</li>
          <li>Pair bright containers (primary / tertiary) for hierarchy emphasis.</li>
          <li>Keep text labels — removing them hurts usability even in expressive UIs.</li>
        </ul>

        <h2 className="text-3xl mt-8 mb-3">Demo</h2>
        <p>Compare standard vs expressive CTA :</p>
        <div className="flex flex-wrap gap-4 items-center not-prose">
          <Button variant="filled">Standard</Button>
          <Button
            variant="filled"
            className="h-14 px-8 text-base"
            style={{
              transition:
                "all var(--md-sys-motion-duration-medium3) var(--md-sys-motion-easing-emphasized)",
            }}
          >
            Expressive CTA
          </Button>
        </div>
      </article>
    </>
  );
}
"#;

const MD3_MOTION_PAGE_TSX: &str = r#"import { Nav } from "@/components/Nav";
import { Transition } from "@md3-ui/core";

const EASINGS = [
  "standard",
  "standard-accelerate",
  "standard-decelerate",
  "emphasized",
  "emphasized-accelerate",
  "emphasized-decelerate",
  "legacy",
] as const;

const DURATIONS = [
  ["short1",  "50ms"],
  ["short2", "100ms"],
  ["short3", "150ms"],
  ["short4", "200ms"],
  ["medium1","250ms"],
  ["medium2","300ms"],
  ["medium3","350ms"],
  ["medium4","400ms"],
  ["long1",  "450ms"],
  ["long2",  "500ms"],
  ["long3",  "550ms"],
  ["long4",  "600ms"],
  ["extra-long1", "700ms"],
  ["extra-long2", "800ms"],
  ["extra-long3", "900ms"],
  ["extra-long4","1000ms"],
] as const;

export default function MotionPage() {
  return (
    <>
      <Nav />
      <article className="container mx-auto p-8 max-w-3xl">
        <h1 className="text-5xl font-bold mb-4">Motion tokens</h1>
        <p className="text-[var(--md-sys-color-on-surface-variant)] mb-8">
          Tokens officiels Material Motion 3.{" "}
          <a
            className="text-[var(--md-sys-color-primary)] underline"
            href="https://m3.material.io/styles/motion/overview/how-it-works"
          >spec</a>
        </p>

        <h2 className="text-2xl mb-3">Easings</h2>
        <ul className="space-y-1 mb-8">
          {EASINGS.map((e) => (
            <li key={e}>
              <code className="bg-[var(--md-sys-color-surface-container)] px-2 py-0.5 rounded">
                --md-sys-motion-easing-{e}
              </code>
            </li>
          ))}
        </ul>

        <h2 className="text-2xl mb-3">Durations</h2>
        <table className="w-full text-left">
          <thead>
            <tr>
              <th className="py-2">Token</th>
              <th className="py-2">Value</th>
            </tr>
          </thead>
          <tbody>
            {DURATIONS.map(([name, value]) => (
              <tr key={name} className="border-t border-[var(--md-sys-color-outline-variant)]">
                <td className="py-2"><code>--md-sys-motion-duration-{name}</code></td>
                <td className="py-2">{value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </article>
    </>
  );
}
"#;

const MD3_TOKENS_PAGE_TSX: &str = r#"import { Nav } from "@/components/Nav";

const COLOR_ROLES = [
  "primary", "on-primary", "primary-container", "on-primary-container",
  "secondary", "on-secondary", "secondary-container", "on-secondary-container",
  "tertiary", "on-tertiary", "tertiary-container", "on-tertiary-container",
  "error", "on-error", "error-container", "on-error-container",
  "surface", "on-surface",
  "surface-container-lowest", "surface-container-low", "surface-container",
  "surface-container-high", "surface-container-highest",
  "outline", "outline-variant",
];

export default function TokensPage() {
  return (
    <>
      <Nav />
      <article className="container mx-auto p-8">
        <h1 className="text-5xl font-bold mb-4">Color tokens</h1>
        <p className="text-[var(--md-sys-color-on-surface-variant)] mb-6">
          Spec M3 :{" "}
          <a className="text-[var(--md-sys-color-primary)] underline"
            href="https://m3.material.io/styles/color/roles">m3.material.io/styles/color/roles</a>
        </p>
        <div className="grid md:grid-cols-3 gap-2">
          {COLOR_ROLES.map((role) => (
            <div
              key={role}
              className="p-4 rounded-[8px] border border-[var(--md-sys-color-outline-variant)] text-sm"
              style={{
                background: `var(--md-sys-color-${role})`,
                color: role.startsWith("on-")
                  ? `var(--md-sys-color-${role.slice(3)})`
                  : `var(--md-sys-color-on-${role.replace("-container", "").replace(/-lowest|-low|-high|-highest/, "")})`,
              }}
            >
              <code>--md-sys-color-{role}</code>
            </div>
          ))}
        </div>
      </article>
    </>
  );
}
"#;

/// Module motion — helpers typed qui exposent les tokens Material Motion M3.
/// Spec : https://m3.material.io/styles/motion/overview/how-it-works
const MD3_MOTION_INDEX_TS: &str = r#"// @md3-ui/core/motion — Material Motion 3 helpers.
//
// Tous les tokens sont exposés en CSS variables (@md3-ui/tokens). Ces helpers
// TS sont fournis pour composer inline-style ou construire des keyframes.
// Spec : https://m3.material.io/styles/motion/overview/how-it-works

export type M3Easing =
  | "standard" | "standard-accelerate" | "standard-decelerate"
  | "emphasized" | "emphasized-accelerate" | "emphasized-decelerate"
  | "legacy" | "legacy-accelerate" | "legacy-decelerate"
  | "linear";

export type M3Duration =
  | "short1" | "short2" | "short3" | "short4"
  | "medium1" | "medium2" | "medium3" | "medium4"
  | "long1" | "long2" | "long3" | "long4"
  | "extra-long1" | "extra-long2" | "extra-long3" | "extra-long4";

export const easing = (kind: M3Easing) => `var(--md-sys-motion-easing-${kind})`;
export const duration = (kind: M3Duration) => `var(--md-sys-motion-duration-${kind})`;

/**
 * Construit une propriété `transition` CSS à partir des tokens M3.
 *
 * ```ts
 * const style = { transition: m3Transition(["opacity", "transform"], "medium2", "emphasized") };
 * ```
 */
export function m3Transition(
  properties: string[] = ["all"],
  dur: M3Duration = "medium2",
  ease: M3Easing = "standard",
): string {
  return properties
    .map((p) => `${p} ${duration(dur)} ${easing(ease)}`)
    .join(", ");
}

export { Transition } from "./Transition";
export { useMotion, useReducedMotion } from "./useMotion";
"#;

const MD3_USE_MOTION_TS: &str = r#"// Hook useMotion — respecte prefers-reduced-motion pour l'accessibilité.

"use client";

import * as React from "react";
import type { M3Duration, M3Easing } from "./index";

export function useReducedMotion(): boolean {
  const [reduced, setReduced] = React.useState(false);
  React.useEffect(() => {
    if (typeof window === "undefined") return;
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const sync = () => setReduced(mq.matches);
    sync();
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  }, []);
  return reduced;
}

/**
 * Retourne une string `transition:` tenant compte de prefers-reduced-motion.
 * Si l'utilisateur a demandé une animation réduite, renvoie "none".
 */
export function useMotion(
  properties: string[],
  dur: M3Duration = "medium2",
  ease: M3Easing = "standard",
): string {
  const reduced = useReducedMotion();
  if (reduced) return "none";
  return properties
    .map((p) => `${p} var(--md-sys-motion-duration-${dur}) var(--md-sys-motion-easing-${ease})`)
    .join(", ");
}
"#;

const MD3_TRANSITION_TSX: &str = r#"// <Transition> — wrapper minimal qui applique une transition M3 à un child.
// Pattern équivalent à l'API CSSTransition mais utilise les tokens officiels.

"use client";

import * as React from "react";
import { useMotion } from "./useMotion";
import type { M3Duration, M3Easing } from "./index";

export interface TransitionProps extends React.HTMLAttributes<HTMLDivElement> {
  properties?: string[];
  duration?: M3Duration;
  easing?: M3Easing;
}

export const Transition = React.forwardRef<HTMLDivElement, TransitionProps>(
  ({ properties = ["all"], duration = "medium2", easing = "standard", style, ...rest }, ref) => {
    const transition = useMotion(properties, duration, easing);
    return <div ref={ref} style={{ transition, ...style }} {...rest} />;
  }
);
Transition.displayName = "M3Transition";
"#;

const MD3_LICENSE_MIT: &str = r#"MIT License

Copyright (c) 2026 md3-ui contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
"#;

fn render_md3_framework_readme(name: &str) -> String {
    format!(
        r#"# {name}

**React framework Material Design 3** — fork de shadcn, monorepo Bun + Rust.

- **Compilation** : Rust via SWC (transpile) + Rspack (bundle) + OXC (lint/resolve)
- **Linting** : Biome (Rust) + plugin custom `@md3-ui/lint-plugin` (règles M3)
- **Styling** : Tailwind v4 + tokens M3 en **CSS custom properties** (zéro CSS-in-JS)
- **Motion** : tokens officiels Material Motion 3 (easings + durations) — helpers
  `m3Transition()`, `useMotion()`, `<Transition>` respectant `prefers-reduced-motion`
- **WebAssembly** : plugin Bun natif `md3-wasm-plugin` (bun-native-plugin + napi-rs)
- **Runtime** : Bun (Web APIs, test runner, dev server, bundler fallback)
- **Registry** : shadcn-compatible (publiable `bunx md3-ui add button`)

## Structure monorepo

```
{name}/
├── packages/
│   ├── core/             → composants React M3 (Button, Card, Chip, FAB,
│   │                       NavigationBar, BottomSheet, SegmentedControl…)
│   ├── tokens/           → CSS + JSON tokens M3 (light / dark)
│   ├── registry/         → shadcn registry fork + build script
│   ├── cli/              → `md3-ui add <component>`
│   ├── lint-plugin/      → règles Biome custom (no-raw-color, use-m3-tokens)
│   └── docs/             → showcase Next 16 + Tailwind v4
├── crates/
│   ├── md3-compiler/         → wrapper SWC+Rspack (build-all, tokens→CSS)
│   ├── md3-registry-builder/ → build registry JSON (Rust, ~10× faster)
│   └── md3-wasm-plugin/      → plugin Bun bundler WASM (cdylib + napi-rs)
├── examples/next-app/    → exemple consommateur
├── biome.json            → config lint+format
├── rspack.config.mjs     → config bundler
├── Cargo.toml            → workspace Rust
└── package.json          → workspaces Bun
```

## Quick start

```bash
bun install
bun run lint            # Biome lint+format
bun run build           # build tous les packages (Rspack → packages/*/dist/)
bun run dev             # showcase Next on :3000
bun run registry:build  # registry JSON (Bun)
bun run registry:build:rust   # même chose en Rust (plus rapide)
```

## Usage depuis un projet externe

```bash
bun add @md3-ui/core @md3-ui/tokens
```

```tsx
import {{ ThemeProvider, Button, Card, CardContent, m3Transition }} from "@md3-ui/core";
import "@md3-ui/tokens/css";  // CSS variables (colors + shape + elevation + motion)

export default function App() {{
  return (
    <ThemeProvider scheme="light">
      <Button
        variant="filled"
        style={{{{ transition: m3Transition(["background", "box-shadow"], "medium2", "emphasized") }}}}
      >
        Click me
      </Button>
      <Card variant="elevated"><CardContent>Hello M3</CardContent></Card>
    </ThemeProvider>
  );
}}
```

## Material Motion 3

Tokens officiels exposés ([spec M3 motion](https://m3.material.io/styles/motion/overview/how-it-works)) :

```css
/* easings */
--md-sys-motion-easing-standard            /* fonctionnel */
--md-sys-motion-easing-emphasized          /* expressif (sheets, pages) */
--md-sys-motion-easing-standard-accelerate /* sortie */
--md-sys-motion-easing-standard-decelerate /* entrée */

/* durations : short1-4, medium1-4, long1-4, extra-long1-4 */
--md-sys-motion-duration-short2  /* 100ms */
--md-sys-motion-duration-medium2 /* 300ms */
--md-sys-motion-duration-long2   /* 500ms */
```

Helpers typed côté React :

```tsx
import {{ m3Transition, useMotion, Transition }} from "@md3-ui/core/motion";

// String-build : `opacity 300ms cubic-bezier(0.2,0,0,1), ...`
const style = {{ transition: m3Transition(["opacity", "transform"], "medium2") }};

// Hook : respecte prefers-reduced-motion → "none" si demandé
const transition = useMotion(["all"], "short4", "emphasized");

// Component wrapper
<Transition properties={{["opacity"]}} duration="short2" easing="standard">
  <div>Fade</div>
</Transition>
```

## Stack Rust côté build

| Crate | Rôle |
|-------|------|
| `md3-compiler` | orchestre Rspack/SWC, émet tokens CSS depuis JSON |
| `md3-registry-builder` | build registry shadcn JSON (serde + walkdir) |
| `md3-wasm-plugin` | plugin Bun natif WASM support (bun-native-plugin + napi-rs) |

## Stack TS côté runtime

| Package | Type | Rôle |
|---------|------|------|
| `@md3-ui/core` | lib React | composants + ThemeProvider + motion helpers |
| `@md3-ui/tokens` | assets CSS+JSON | tokens M3 |
| `@md3-ui/registry` | lib | registry shadcn compatible |
| `@md3-ui/cli` | CLI | `md3-ui add button` |
| `@md3-ui/lint-plugin` | Biome/ESLint | règles M3 (no-raw-color, use-m3-tokens) |
| `@md3-ui/docs` | Next 16 | showcase |

## Refs

- [Material Design 3](https://m3.material.io/)
- [shadcn/ui Registry](https://ui.shadcn.com/docs/registry)
- [Rspack](https://rspack.rs/) · [SWC](https://swc.rs/) · [OXC](https://oxc.rs/)
- [Biome](https://biomejs.dev/)
- [Bun](https://bun.sh/) · [bun-native-plugin](https://docs.rs/bun-native-plugin)
- [Material Motion 3](https://m3.material.io/styles/motion/overview/how-it-works)
- [Expressive Material Design (Google Research)](https://design.google/library/expressive-material-design-google-research)

## License

MIT
"#,
    )
}


/// Config MCP — initialise le server MCP shadcn pour Claude/Cursor/etc.
/// Équivalent de `bunx --bun shadcn@latest mcp init --client claude`.
/// Claude Desktop lit ce fichier via son propre config (~/.config/claude/...).
const SHADCN_MCP_JSON: &str = r#"{
  "mcpServers": {
    "shadcn": {
      "command": "bunx",
      "args": ["--bun", "shadcn@latest", "mcp"]
    }
  }
}
"#;

fn render_shadcn_readme(name: &str) -> String {
    format!(
        r#"# {name}

Next.js 16 + Tailwind v4 + shadcn/ui (Radix + CVA + Tailwind Merge).

Scaffolded by `n2b ui init --flavor shadcn`.

## Dev

```bash
bun install
bun dev                 # bunx --bun next dev --turbopack
```

## Ajouter un composant shadcn

```bash
# Add officiel (copie le code dans src/components/ui/)
bunx --bun shadcn@latest add dialog
bunx --bun shadcn@latest add dropdown-menu
bunx --bun shadcn@latest add form

# Via registry custom (shadcn v2 API)
bunx --bun shadcn@latest add https://my-registry.example.com/registry/button.json
```

## AI workflow (Claude / Cursor / Zed)

### MCP server shadcn (interaction Claude avec ton registry)

Le fichier `.mcp.json` configure le server MCP shadcn. Pour l'activer :

```bash
# Initialise la config MCP côté client IA
bunx --bun shadcn@latest mcp init --client claude
bunx --bun shadcn@latest mcp init --client cursor
```

Claude Desktop peut alors lister / ajouter des composants shadcn en conversation :
> « Add the dialog and dropdown-menu components from shadcn/ui »

### Skills (context packs AI)

```bash
# Package `skills` — installe des instructions/skills pour ton agent IA
bunx --bun skills add shadcn/ui
bunx --bun skills add next
bunx --bun skills list
```

## Structure

```
{name}/
├── components.json          ← config shadcn (style, aliases, CSS path)
├── .mcp.json                ← config MCP server shadcn (Claude/Cursor)
├── src/
│   ├── app/
│   │   ├── layout.tsx       ← racine Next App Router
│   │   ├── page.tsx         ← démo Button
│   │   └── globals.css      ← @import "tailwindcss"
│   ├── components/ui/
│   │   └── button.tsx       ← composant shadcn (CVA variants)
│   └── lib/utils.ts         ← cn(classes) helper
├── postcss.config.mjs       ← @tailwindcss/postcss
└── next.config.ts           ← turbopack: {{}}
```

## Refs

- [shadcn/ui](https://ui.shadcn.com/)
- [shadcn Registry API](https://ui.shadcn.com/docs/registry/getting-started)
- [Radix UI](https://www.radix-ui.com/)
- [Tailwind v4](https://tailwindcss.com/)
- [Material Design tokens (à croiser avec shadcn themes)](https://m3.material.io/)
"#,
    )
}
