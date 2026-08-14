//! Strict recipe.json validator.
//!
//! [`Recipe::parse`](super::Recipe) is deliberately permissive: `serde` ignores unknown
//! fields, so a typo like `post_install.npm_install` or `transform: "kebab"` parses
//! cleanly and is then silently dropped on the floor. This module is the strict
//! counterpart — it walks the raw JSON with per-object allowlists mirroring the
//! structs in `parser.rs` exactly and reports every deviation as a [`Finding`]
//! instead of failing fast, so a whole recipe can be linted in one pass.
//!
//! It is pure by default; the only filesystem access is the optional
//! `common://` template-source existence check, enabled by passing a recipes root
//! to [`validate_recipe_json_with_root`].

use std::path::Path;

use serde_json::{Map, Value};

/// A single validation problem, addressed by a JSONPath-ish location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Location of the problem, e.g. `$.post_install.npm_install`.
    pub path: String,
    /// Human-readable description of what is wrong.
    pub message: String,
}

// ── Allowed keys — these MUST mirror the structs in parser.rs exactly ──────────

const RECIPE_KEYS: &[&str] = &[
    "name",
    "title",
    "description",
    "version",
    "features",
    "services",
    "options",
    "placeholders",
    "init_app",
    "directories",
    "templates",
    "post_install",
    "next_steps",
    "notes",
];
const SERVICE_KEYS: &[&str] = &["name", "description"];
const OPTION_KEYS: &[&str] = &["flag", "default", "description", "required", "choices"];
const PLACEHOLDER_KEYS: &[&str] = &["source", "transform"];
const INIT_APP_KEYS: &[&str] = &["cwd", "target_dir", "skip_if_exists", "command"];
const TEMPLATE_KEYS: &[&str] = &["from", "to", "condition"];
const POST_INSTALL_KEYS: &[&str] = &[
    "create_files",
    "rename",
    "chmod",
    "gitkeep",
    "run",
    "gitignore",
];
const CREATE_FILE_KEYS: &[&str] = &["path", "content"];
const RENAME_KEYS: &[&str] = &["from", "to"];
const CHMOD_KEYS: &[&str] = &["path", "mode"];
const RUN_KEYS: &[&str] = &["command", "cwd", "description"];

/// Transforms understood by `Recipe::build_placeholders`.
const TRANSFORMS: &[&str] = &["slug", "upper", "rust_crate", "ssr_bool"];

/// Validate a parsed `recipe.json` document without touching the filesystem.
pub fn validate_recipe_json(raw: &Value) -> Vec<Finding> {
    validate_recipe_json_with_root(raw, None)
}

/// Validate a parsed `recipe.json` document.
///
/// When `recipes_root` is supplied (the directory holding `common/` and the
/// per-recipe directories, i.e. `templates/recipes`), `templates[].from` entries
/// using the `common://` namespace are additionally checked for existence.
pub fn validate_recipe_json_with_root(raw: &Value, recipes_root: Option<&Path>) -> Vec<Finding> {
    let mut v = Validator {
        root: recipes_root,
        findings: Vec::new(),
    };
    v.recipe(raw);
    v.findings
}

struct Validator<'a> {
    root: Option<&'a Path>,
    findings: Vec<Finding>,
}

impl Validator<'_> {
    fn report(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.findings.push(Finding {
            path: path.into(),
            message: message.into(),
        });
    }

    // ── primitives ───────────────────────────────────────────────────────────

    fn as_object<'j>(&mut self, path: &str, value: &'j Value) -> Option<&'j Map<String, Value>> {
        match value.as_object() {
            Some(m) => Some(m),
            None => {
                self.report(path, "expected an object");
                None
            }
        }
    }

    fn as_array<'j>(&mut self, path: &str, value: &'j Value) -> Option<&'j Vec<Value>> {
        match value.as_array() {
            Some(a) => Some(a),
            None => {
                self.report(path, "expected an array");
                None
            }
        }
    }

    fn expect_string(&mut self, path: &str, value: &Value) {
        if !value.is_string() {
            self.report(path, "expected a string");
        }
    }

    fn expect_bool(&mut self, path: &str, value: &Value) {
        if !value.is_boolean() {
            self.report(path, "expected a boolean");
        }
    }

    fn expect_string_array(&mut self, path: &str, value: &Value) {
        if let Some(items) = self.as_array(path, value) {
            for (i, item) in items.iter().enumerate() {
                if !item.is_string() {
                    self.report(format!("{path}[{i}]"), "expected a string");
                }
            }
        }
    }

    /// Report every key that is not in `allowed` — this is what makes drift loud.
    fn unknown_keys(&mut self, path: &str, map: &Map<String, Value>, allowed: &[&str]) {
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                self.report(
                    format!("{path}.{key}"),
                    format!(
                        "unknown field `{key}` (allowed: {}) — it is silently ignored at parse time",
                        allowed.join(", ")
                    ),
                );
            }
        }
    }

    fn require(&mut self, path: &str, map: &Map<String, Value>, keys: &[&str]) {
        for key in keys {
            if !map.contains_key(*key) {
                self.report(
                    format!("{path}.{key}"),
                    format!("required field `{key}` is missing"),
                );
            }
        }
    }

    // ── structure ────────────────────────────────────────────────────────────

    fn recipe(&mut self, raw: &Value) {
        let Some(map) = self.as_object("$", raw) else {
            return;
        };
        self.unknown_keys("$", map, RECIPE_KEYS);
        self.require("$", map, &["name"]);

        for key in ["name", "title", "description", "version"] {
            if let Some(v) = map.get(key) {
                self.expect_string(&format!("$.{key}"), v);
            }
        }
        for key in ["features", "directories", "next_steps", "notes"] {
            if let Some(v) = map.get(key) {
                self.expect_string_array(&format!("$.{key}"), v);
            }
        }

        if let Some(v) = map.get("services") {
            self.each_object("$.services", v, |s, path, m| {
                s.unknown_keys(path, m, SERVICE_KEYS);
                s.require(path, m, &["name"]);
                for key in ["name", "description"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
            });
        }

        if let Some(v) = map.get("options") {
            self.each_map_entry("$.options", v, |s, path, m| {
                s.unknown_keys(path, m, OPTION_KEYS);
                for key in ["flag", "default", "description"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
                if let Some(x) = m.get("required") {
                    s.expect_bool(&format!("{path}.required"), x);
                }
                if let Some(x) = m.get("choices") {
                    s.expect_string_array(&format!("{path}.choices"), x);
                }
            });
        }

        if let Some(v) = map.get("placeholders") {
            self.each_map_entry("$.placeholders", v, |s, path, m| {
                s.unknown_keys(path, m, PLACEHOLDER_KEYS);
                s.require(path, m, &["source"]);
                if let Some(x) = m.get("source") {
                    s.expect_string(&format!("{path}.source"), x);
                }
                if let Some(x) = m.get("transform") {
                    let tpath = format!("{path}.transform");
                    match x.as_str() {
                        Some(t) if TRANSFORMS.contains(&t) => {}
                        Some(t) => s.report(
                            tpath,
                            format!(
                                "unknown transform `{t}` (allowed: {}) — it falls through to the untransformed value",
                                TRANSFORMS.join(", ")
                            ),
                        ),
                        None => s.report(tpath, "expected a string"),
                    }
                }
            });
        }

        if let Some(v) = map.get("init_app") {
            if let Some(m) = self.as_object("$.init_app", v) {
                let m = m.clone();
                self.unknown_keys("$.init_app", &m, INIT_APP_KEYS);
                self.require("$.init_app", &m, &["command"]);
                for key in ["cwd", "target_dir", "command"] {
                    if let Some(x) = m.get(key) {
                        self.expect_string(&format!("$.init_app.{key}"), x);
                    }
                }
                if let Some(x) = m.get("skip_if_exists") {
                    self.expect_bool("$.init_app.skip_if_exists", x);
                }
            }
        }

        if let Some(v) = map.get("templates") {
            self.each_object("$.templates", v, |s, path, m| {
                s.unknown_keys(path, m, TEMPLATE_KEYS);
                s.require(path, m, &["from", "to"]);
                for key in ["from", "to", "condition"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
                if let Some(from) = m.get("from").and_then(Value::as_str) {
                    s.check_template_source(&format!("{path}.from"), from);
                }
            });
        }

        if let Some(v) = map.get("post_install") {
            self.post_install(v);
        }
    }

    fn post_install(&mut self, value: &Value) {
        let Some(map) = self.as_object("$.post_install", value) else {
            return;
        };
        let map = map.clone();
        self.unknown_keys("$.post_install", &map, POST_INSTALL_KEYS);

        for key in ["gitkeep", "gitignore"] {
            if let Some(v) = map.get(key) {
                self.expect_string_array(&format!("$.post_install.{key}"), v);
            }
        }

        if let Some(v) = map.get("create_files") {
            self.each_object("$.post_install.create_files", v, |s, path, m| {
                s.unknown_keys(path, m, CREATE_FILE_KEYS);
                s.require(path, m, &["path", "content"]);
                for key in ["path", "content"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
            });
        }
        if let Some(v) = map.get("rename") {
            self.each_object("$.post_install.rename", v, |s, path, m| {
                s.unknown_keys(path, m, RENAME_KEYS);
                s.require(path, m, &["from", "to"]);
                for key in ["from", "to"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
            });
        }
        if let Some(v) = map.get("chmod") {
            self.each_object("$.post_install.chmod", v, |s, path, m| {
                s.unknown_keys(path, m, CHMOD_KEYS);
                s.require(path, m, &["path"]);
                for key in ["path", "mode"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
            });
        }
        if let Some(v) = map.get("run") {
            self.each_object("$.post_install.run", v, |s, path, m| {
                s.unknown_keys(path, m, RUN_KEYS);
                s.require(path, m, &["command"]);
                for key in ["command", "cwd", "description"] {
                    if let Some(x) = m.get(key) {
                        s.expect_string(&format!("{path}.{key}"), x);
                    }
                }
            });
        }
    }

    /// `common://…` sources resolve to `<recipes_root>/common/<rest>` (see
    /// `RecipeInstaller::resolve_template_source`). Everything else is relative to
    /// the recipe directory and is not checked here.
    fn check_template_source(&mut self, path: &str, from: &str) {
        let (Some(root), Some(rest)) = (self.root, from.strip_prefix("common://")) else {
            return;
        };
        let resolved = root.join("common").join(rest);
        if !resolved.exists() {
            self.report(
                path,
                format!(
                    "template source `{from}` resolves to {} which does not exist",
                    resolved.display()
                ),
            );
        }
    }

    // ── iteration helpers ────────────────────────────────────────────────────

    /// Walk an array of objects, e.g. `templates`, applying `f` to each element.
    fn each_object<F>(&mut self, path: &str, value: &Value, f: F)
    where
        F: Fn(&mut Self, &str, &Map<String, Value>),
    {
        let Some(items) = self.as_array(path, value) else {
            return;
        };
        let items = items.clone();
        for (i, item) in items.iter().enumerate() {
            let item_path = format!("{path}[{i}]");
            if let Some(m) = self.as_object(&item_path, item) {
                let m = m.clone();
                f(self, &item_path, &m);
            }
        }
    }

    /// Walk a map of named objects, e.g. `options` / `placeholders`.
    fn each_map_entry<F>(&mut self, path: &str, value: &Value, f: F)
    where
        F: Fn(&mut Self, &str, &Map<String, Value>),
    {
        let Some(map) = self.as_object(path, value) else {
            return;
        };
        let map = map.clone();
        for (name, item) in map.iter() {
            let item_path = format!("{path}.{name}");
            if let Some(m) = self.as_object(&item_path, item) {
                let m = m.clone();
                f(self, &item_path, &m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn messages(findings: &[Finding]) -> String {
        findings
            .iter()
            .map(|f| format!("{} :: {}", f.path, f.message))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn minimal() -> serde_json::Value {
        json!({
            "name": "demo",
            "title": "Demo",
            "description": "A demo recipe",
            "version": "1.0",
            "features": ["one"],
            "services": [{ "name": "<name>", "description": "app" }],
            "options": {
                "domain": { "flag": "--domain", "default": "d.localhost", "description": "domain" }
            },
            "placeholders": {
                "SERVICE_NAME": { "source": "name" },
                "SERVICE_SLUG": { "source": "name", "transform": "slug" }
            },
            "init_app": {
                "cwd": "apps",
                "target_dir": "apps/x",
                "skip_if_exists": true,
                "command": "echo hi"
            },
            "directories": ["apps/x"],
            "templates": [{ "from": "README.md", "to": "apps/x/README.md" }],
            "post_install": {
                "create_files": [{ "path": "a", "content": "b" }],
                "rename": [{ "from": "a", "to": "b" }],
                "chmod": [{ "path": "a", "mode": "+x" }],
                "gitkeep": ["apps/x/tmp"],
                "run": [{ "command": "true", "cwd": ".", "description": "noop" }],
                "gitignore": ["/tmp"]
            },
            "next_steps": ["cd apps/x"],
            "notes": ["a note"]
        })
    }

    #[test]
    fn clean_minimal_recipe_has_no_findings() {
        let findings = validate_recipe_json(&minimal());
        assert!(
            findings.is_empty(),
            "expected clean, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn unknown_post_install_key_is_reported() {
        let mut raw = minimal();
        raw["post_install"]["npm_install"] = json!(true);
        let findings = validate_recipe_json(&raw);
        assert!(
            findings.iter().any(
                |f| f.path == "$.post_install.npm_install" && f.message.contains("npm_install")
            ),
            "expected npm_install finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn unknown_top_level_key_is_reported() {
        let mut raw = minimal();
        raw["bogus"] = json!(1);
        let findings = validate_recipe_json(&raw);
        assert!(
            findings.iter().any(|f| f.path == "$.bogus"),
            "expected top-level finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn unknown_placeholder_transform_is_reported() {
        let mut raw = minimal();
        raw["placeholders"]["SERVICE_SLUG"]["transform"] = json!("kebab");
        let findings = validate_recipe_json(&raw);
        assert!(
            findings.iter().any(|f| {
                f.path == "$.placeholders.SERVICE_SLUG.transform" && f.message.contains("kebab")
            }),
            "expected kebab finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn every_known_transform_is_accepted() {
        for t in ["slug", "upper", "rust_crate", "ssr_bool"] {
            let mut raw = minimal();
            raw["placeholders"]["SERVICE_SLUG"]["transform"] = json!(t);
            let findings = validate_recipe_json(&raw);
            assert!(findings.is_empty(), "{t}: {}", messages(&findings));
        }
    }

    #[test]
    fn missing_required_key_is_reported() {
        let mut raw = minimal();
        raw.as_object_mut().unwrap().remove("name");
        let findings = validate_recipe_json(&raw);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("required") && f.path == "$.name"),
            "expected missing-name finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn wrong_type_is_reported() {
        let mut raw = minimal();
        raw["templates"] = json!("not-an-array");
        let findings = validate_recipe_json(&raw);
        assert!(
            findings.iter().any(|f| f.path == "$.templates"),
            "expected type finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn dangling_common_template_source_is_reported() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("common/docker")).unwrap();

        let mut raw = minimal();
        raw["templates"] = serde_json::json!([{ "from": "common://nope", "to": "x" }]);
        let findings = validate_recipe_json_with_root(&raw, Some(root.path()));
        assert!(
            findings
                .iter()
                .any(|f| f.path == "$.templates[0].from" && f.message.contains("common://nope")),
            "expected dangling finding, got: {}",
            messages(&findings)
        );
    }

    #[test]
    fn existing_common_template_source_is_accepted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("common/docker")).unwrap();
        std::fs::write(root.path().join("common/docker/base.yml"), "x").unwrap();

        let mut raw = minimal();
        raw["templates"] =
            serde_json::json!([{ "from": "common://docker/base.yml", "to": "docker/base.yml" }]);
        let findings = validate_recipe_json_with_root(&raw, Some(root.path()));
        assert!(findings.is_empty(), "{}", messages(&findings));
    }

    #[test]
    fn common_source_is_not_checked_without_a_root() {
        let mut raw = minimal();
        raw["templates"] = serde_json::json!([{ "from": "common://nope", "to": "x" }]);
        let findings = validate_recipe_json(&raw);
        assert!(findings.is_empty(), "{}", messages(&findings));
    }

    #[test]
    fn non_object_root_is_reported() {
        let findings = validate_recipe_json(&serde_json::json!([1, 2, 3]));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "$");
    }
}
