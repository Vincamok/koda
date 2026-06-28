use super::provider::AiContext;

/// Builds the 6-layer LLM context for a workspace AI request.
pub struct AiContextBuilder {
    layers: Vec<Option<String>>,
    locale: String,
}

impl AiContextBuilder {
    pub fn new() -> Self {
        Self {
            layers: vec![None; 6],
            locale: "fr".into(),
        }
    }

    /// Layer 1 — platform prompt (super_admin editable)
    pub fn platform_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.layers[0] = Some(prompt.into());
        self
    }

    /// Layer 2 — org prompt (org admin editable)
    pub fn org_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.layers[1] = Some(prompt.into());
        self
    }

    /// Layer 3+4 — language + framework packs (built-in, auto-detected)
    pub fn lang_packs(mut self, packs: Vec<String>) -> Self {
        if !packs.is_empty() {
            self.layers[2] = Some(packs.join("\n\n"));
        }
        self
    }

    pub fn framework_packs(mut self, packs: Vec<String>) -> Self {
        if !packs.is_empty() {
            self.layers[3] = Some(packs.join("\n\n"));
        }
        self
    }

    /// Layer 5 — workspace KODA.md
    pub fn koda_md(mut self, content: impl Into<String>) -> Self {
        let s = content.into();
        if !s.is_empty() {
            self.layers[4] = Some(s);
        }
        self
    }

    /// Layer 6 — personal ai/instructions.md + locale injection
    pub fn personal_instructions(mut self, content: impl Into<String>) -> Self {
        let mut layer = content.into();
        // Inject language preference
        let lang_instruction = match self.locale.as_str() {
            "fr" => "Always respond in French (français).",
            "en" => "Always respond in English.",
            "es" => "Always respond in Spanish (español).",
            "de" => "Always respond in German (Deutsch).",
            _ => "",
        };
        if !lang_instruction.is_empty() {
            if layer.is_empty() {
                layer = lang_instruction.to_string();
            } else {
                layer = format!("{lang_instruction}\n\n{layer}");
            }
        }
        self.layers[5] = Some(layer);
        self
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    pub fn build(self, messages: Vec<super::provider::ChatMessage>, tools: Vec<serde_json::Value>) -> AiContext {
        let system_layers: Vec<String> = self
            .layers
            .into_iter()
            .flatten()
            .collect();

        AiContext { system_layers, messages, tools }
    }
}

impl Default for AiContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in language pack content.
pub fn builtin_lang_pack(lang: &str) -> Option<&'static str> {
    match lang {
        "rust" => Some(include_str!("../lang_packs/rust.md")),
        "typescript" => Some(include_str!("../lang_packs/typescript.md")),
        "python" => Some(include_str!("../lang_packs/python.md")),
        "go" => Some(include_str!("../lang_packs/go.md")),
        "sql" => Some(include_str!("../lang_packs/sql.md")),
        _ => None,
    }
}

/// Built-in framework pack content.
pub fn builtin_framework_pack(framework: &str) -> Option<&'static str> {
    match framework {
        "axum" => Some(include_str!("../lang_packs/axum.md")),
        "react" => Some(include_str!("../lang_packs/react.md")),
        "nextjs" => Some(include_str!("../lang_packs/nextjs.md")),
        "sqlx" => Some(include_str!("../lang_packs/sqlx.md")),
        _ => None,
    }
}

/// Detects active language packs from manifest file names present in a repo.
pub fn detect_packs(manifest_files: &[&str], dep_names: &[&str]) -> (Vec<String>, Vec<String>) {
    let mut lang_packs = vec![];
    let mut framework_packs = vec![];

    let has = |name: &str| manifest_files.iter().any(|f| f.contains(name));
    let has_dep = |dep: &str| dep_names.iter().any(|d| d.contains(dep));

    if has("Cargo.toml") {
        lang_packs.push("rust".into());
        if has_dep("axum") { framework_packs.push("axum".into()); }
        if has_dep("sqlx") { framework_packs.push("sqlx".into()); }
    }
    if has("package.json") || has("tsconfig") {
        lang_packs.push("typescript".into());
        if has_dep("react") { framework_packs.push("react".into()); }
        if has("next.config") || has_dep("next") { framework_packs.push("nextjs".into()); }
    }
    if has("requirements.txt") || has("pyproject.toml") || has("setup.py") {
        lang_packs.push("python".into());
    }
    if has("go.mod") {
        lang_packs.push("go".into());
    }

    (lang_packs, framework_packs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_produces_ordered_layers() {
        let ctx = AiContextBuilder::new()
            .platform_prompt("platform")
            .org_prompt("org")
            .lang_packs(vec!["use rust".into()])
            .koda_md("# KODA")
            .locale("en")
            .personal_instructions("my instructions")
            .build(vec![], vec![]);

        // Layers 1, 2, 3, 5, 6 set; layer 4 (framework) skipped
        assert_eq!(ctx.system_layers.len(), 5);
        assert_eq!(ctx.system_layers[0], "platform");
        assert_eq!(ctx.system_layers[1], "org");
        assert_eq!(ctx.system_layers[2], "use rust");
        assert_eq!(ctx.system_layers[3], "# KODA");
        assert!(ctx.system_layers[4].contains("Always respond in English"));
        assert!(ctx.system_layers[4].contains("my instructions"));
    }

    #[test]
    fn locale_fr_injects_french_instruction() {
        let ctx = AiContextBuilder::new()
            .locale("fr")
            .personal_instructions("")
            .build(vec![], vec![]);

        assert_eq!(ctx.system_layers.len(), 1);
        assert!(ctx.system_layers[0].contains("français"));
    }

    #[test]
    fn detect_packs_rust_axum() {
        let manifests = &["Cargo.toml"];
        let deps = &["axum", "sqlx", "tokio"];
        let (langs, frameworks) = detect_packs(manifests, deps);
        assert!(langs.contains(&"rust".to_string()));
        assert!(frameworks.contains(&"axum".to_string()));
        assert!(frameworks.contains(&"sqlx".to_string()));
    }

    #[test]
    fn detect_packs_nextjs() {
        let manifests = &["package.json", "next.config.ts"];
        let deps = &["next", "react", "typescript"];
        let (langs, frameworks) = detect_packs(manifests, deps);
        assert!(langs.contains(&"typescript".to_string()));
        assert!(frameworks.contains(&"nextjs".to_string()));
        assert!(frameworks.contains(&"react".to_string()));
    }

    #[test]
    fn detect_packs_empty_input() {
        let (langs, frameworks) = detect_packs(&[], &[]);
        assert!(langs.is_empty());
        assert!(frameworks.is_empty());
    }

    #[test]
    fn builtin_lang_packs_exist() {
        assert!(builtin_lang_pack("rust").is_some());
        assert!(builtin_lang_pack("typescript").is_some());
        assert!(builtin_lang_pack("python").is_some());
        assert!(builtin_lang_pack("go").is_some());
        assert!(builtin_lang_pack("sql").is_some());
        assert!(builtin_lang_pack("unknown").is_none());
    }

    #[test]
    fn builtin_framework_packs_exist() {
        assert!(builtin_framework_pack("axum").is_some());
        assert!(builtin_framework_pack("react").is_some());
        assert!(builtin_framework_pack("nextjs").is_some());
        assert!(builtin_framework_pack("sqlx").is_some());
        assert!(builtin_framework_pack("unknown").is_none());
    }
}
