---
description: "Run all project build/type/lint checks in one pass. Detects tech stack automatically and runs the right verification commands in parallel."
---

# Project-wide Verification Check

Run all relevant build, type-check, and lint verification steps for the current project in a **single parallel batch**, then report a concise pass/fail summary.

## Instructions

1. **Detect the tech stack** by probing for config files in the project root (or `$ARGUMENTS` if given):
   - `package.json` → Node.js/JS/TS project
   - `tsconfig.json` → TypeScript (look for `tsc --noEmit`)
   - `vite.config.*` → Vite (look for `npx vite build`)
   - `Cargo.toml` → Rust (look for `cargo check`)
   - `requirements.txt` / `pyproject.toml` / `setup.py` → Python (look for `python -m py_compile` on changed files, or `ruff check` / `mypy` if configured)
   - `src-tauri/Cargo.toml` → Tauri app (check both frontend and Rust backend)

2. **Build the check commands** — for each detected toolchain, prepare ONE verification command:
   - **TypeScript**: `npx tsc --noEmit 2>&1`
   - **Vite**: `npx vite build 2>&1`
   - **Rust**: `cargo check 2>&1` (use `--manifest-path src-tauri/Cargo.toml` if Tauri subproject)
   - **Python**: `python -m py_compile <changed_files>` or `ruff check .` / `mypy .` if available
   - **ESLint**: `npx eslint . --max-warnings=0 2>&1` (only if `.eslintrc*` exists)

3. **Run all checks in parallel** using Bash tool calls. Set a reasonable timeout (120s for builds, 60s for type checks).

4. **Report results** in a compact table:
   ```
   | Check      | Status | Errors |
   |------------|--------|--------|
   | tsc        | ✅ OK  | 0      |
   | vite build | ❌ FAIL| 3      |
   | cargo check| ✅ OK  | 0      |
   ```

5. **If any check fails**: Show only the first 10 lines of error output, then stop. Do NOT automatically fix anything — just report.

6. **If all pass**: Report "All checks passed ✅" and list what was verified.

## Important

- Run checks in **parallel** — never sequentially when they are independent.
- Do NOT run checks after every individual edit. This command is meant to be run **once** after a batch of changes.
- If `$ARGUMENTS` is provided, use it as the working directory. Otherwise use the current project root.
- Do NOT attempt to fix errors automatically. Report only.
