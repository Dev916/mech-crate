# Recipe Update Feature — Design Spec

**Date:** 2026-04-17
**Status:** Draft — ready for review
**Author:** Mike + Claude (brainstorming session)

## Problem

Apps scaffolded via `mx add <recipe>` are snapshots. Once a recipe evolves (bug fixes in Dockerfiles, new compose patterns, updated env conventions, improved boilerplate), there's no way to pull those improvements into existing apps. Users either manually cherry-pick changes or live with drift.

We need `mx update` — a command that pulls the latest recipe updates into an already-scaffolded app with safe merge semantics.

## Goals

- Pull latest recipe changes (docker/infra AND app boilerplate) into an existing scaffolded app.
- Preserve user modifications — never silently overwrite.
- Show what will change before applying (dry-run by default).
- Handle conflicts predictably using git-style merge semantics.
- Scope: any file the recipe originally scaffolded. Ignore user-added files.

## Non-goals

- Automatic migrations of user code (e.g., rewriting imports, renaming symbols).
- Tracking external dependencies (npm/cargo/composer upgrades).
- Rolling back an update (user can use git).
- Cross-recipe migrations (e.g., astro → nuxt).

## Approach

### Baseline via manifest + 3-way merge

At scaffold time, `mx add` writes a manifest to `.mx/manifest.json` in the target app. The manifest records exactly what the recipe produced, so on update we can reconstruct the "original" side of a 3-way merge:

- **base** = recipe@original (the version at scaffold time)
- **ours** = current file on disk (may have user edits)
- **theirs** = recipe@latest (with same placeholders re-applied)

Standard 3-way merge rules:
- file unchanged by user + changed by recipe → take recipe (auto)
- file changed by user + unchanged by recipe → keep user (auto)
- file changed by both, non-overlapping regions → auto-merge
- file changed by both, overlapping → **conflict** (see UX below)
- file deleted by recipe → flag + prompt
- file added by recipe → add
- file not in manifest (user-added) → ignore

### Manifest shape

`.mx/manifest.json`:

```json
{
  "recipe": "rust-api",
  "recipe_sha": "abc123def456...",
  "scaffolded_at": "2026-04-17T12:34:56Z",
  "mx_version": "0.4.2",
  "placeholders": {
    "APP_NAME": "ai_share",
    "SERVICE_UPPER": "AI_SHARE",
    "DOMAIN": "ai-share.local"
  },
  "files": {
    "docker/compose/app.yml": {
      "sha256": "e3b0c44...",
      "source": "rust-api"
    },
    "docker/compose/redis.yml": {
      "sha256": "a7f2c1...",
      "source": "common"
    },
    "docker/.config/.env.redis": {
      "sha256": "d4c9b8...",
      "source": "common"
    }
  }
}
```

Fields:
- `recipe` — recipe name as invoked
- `recipe_sha` — git commit SHA of the templates tree at scaffold time (precise baseline for 3-way merge)
- `placeholders` — stored so we can re-render recipe@latest with identical values before diffing
- `files[path].sha256` — hash of the file as scaffolded (pre-user-edit), used to detect "user modified?"
- `files[path].source` — `recipe-name` or `common` (for shared templates pulled via `common://`)

### Version tracking

Recipes are currently only versioned via git. We'll record the git SHA of `templates/recipes/<recipe>/` (and `templates/recipes/common/`) at scaffold time. This gives us a precise "original" without requiring manual version bumps in `recipe.json`.

Future enhancement: optional `version` field in `recipe.json` for human-readable versions, but SHA remains the source of truth.

### Command UX

```
mx update [--dry-run] [--only <path>...] [--interactive]
```

**Default behavior** (no flags):
1. Load manifest, detect recipe + SHA at scaffold time.
2. Check out recipe@scaffold-sha and recipe@HEAD into temp dirs.
3. Re-render both with stored placeholders.
4. Compute 3-way merge for each tracked file.
5. Print summary:
   ```
   Recipe: rust-api  (a7c2f30 → e15fa04, 12 commits)

   Changes:
     M  docker/compose/app.yml          (auto-merged)
     M  docker/dockerfiles/app.Dockerfile (auto-merged)
     A  docker/compose/redis.yml        (new)
     D  docker/compose/old-thing.yml    (recipe removed)
     C  Cargo.toml                      (conflict — overlapping edits)

   3 untracked files in docker/ (ignored)

   Proceed? [y/N]
   ```
6. On confirm: apply. Conflicts are written with git-style markers. Deletions prompt per-file.

**`--dry-run`**: print summary only, don't apply.

**`--only <path>...`**: restrict to specific files/dirs. Example: `mx update --only docker/` to only sync infra.

**`--interactive`**: for conflicts, launch per-file picker (`[k]eep mine / [t]ake theirs / [e]dit / [s]kip`) instead of writing markers.

### Conflict resolution

Default: git-style markers written into the file.

```
<<<<<<< yours
FROM rust:1.75-alpine AS builder
=======
FROM rust:1.80-alpine AS builder
>>>>>>> recipe (e15fa04)
```

Rationale:
- Works in non-interactive contexts (CI, scripts).
- Devs already know how to resolve these.
- `--interactive` available for users who prefer a picker.

### Deletion handling

Recipe removed a file that's still in the app:
- If user hasn't modified it (matches manifest hash) → prompt: `delete? [y/N]`
- If user has modified it → always prompt, default to keep: `recipe removed X, but you've modified it — delete? [y/N]` (default N)

Never silently delete.

### After update

- Rewrite `.mx/manifest.json` with new `recipe_sha`, updated per-file hashes (based on what's now on disk post-merge), and any new files.
- Files with unresolved conflict markers are hashed as-is; user is expected to resolve and commit.

## Edge cases

**User moved/renamed a tracked file**
- Manifest still points to old path. Detection: file missing on disk.
- Behavior: treat as "user deleted" — recipe changes to that file are ignored, new recipe content (if path still exists) is treated as an add. Log a warning.

**Placeholders evolved**
- If recipe@latest uses a new placeholder not in stored manifest, fail with clear error: `recipe now requires placeholder X — run \`mx update --set X=<value>\``.
- Add `--set KEY=VALUE` flag for this.

**`common://` templates changed independently**
- Shared templates have their own SHA tracking via the `source: "common"` marker + the common tree's SHA.
- Store `common_sha` alongside `recipe_sha` in manifest.

**Binary files**
- Already handled by installer (byte-copy fallback). For updates: if user hash differs from manifest hash AND recipe hash differs, treat as conflict — save recipe version as `<file>.new` and tell user to compare manually. Don't try to merge binary.

**No `.mx/manifest.json` exists (pre-manifest scaffolds)**
- `mx update` errors with guidance: `no manifest found — run \`mx update init --recipe <name>\` to adopt this app, which will create a manifest based on current state`.
- `mx update init` attempts best-effort: hashes current files, sets `recipe_sha` to `HEAD` of templates, warns that baseline detection will be approximate.

## Implementation scope (high-level)

1. **Manifest writer** — hook into `installer.rs` after scaffold completes. Collect file paths + hashes + source + placeholders + recipe SHA.
2. **Git SHA resolver** — small helper to get the SHA of a subdir in the templates repo at scaffold time and at update time.
3. **3-way merge engine** — use an existing Rust crate (`diffy` or similar) rather than rolling our own.
4. **`mx update` command** — new subcommand in `mx-cli`. Load manifest, run merge engine, print summary, apply.
5. **`mx update init`** — adoption path for pre-manifest apps.
6. **Tests** — integration tests covering: clean update (no conflicts), conflicts, deletions, additions, placeholder mismatches, binary files, missing manifest.

## Open questions for review

- Name: `mx update` vs `mx sync` vs `mx recipe apply`. Current lean: `mx update`.
- Do we want a `mx update --list` to show available updates without computing the full diff? (cheap `git log` on the recipe tree)
- Should `mx update` be per-recipe or support multi-recipe apps (apps that used `mx add` twice)? Current assumption: one manifest, one recipe per app.

## Next step

On approval of this spec, invoke `superpowers:writing-plans` to produce an implementation plan.
