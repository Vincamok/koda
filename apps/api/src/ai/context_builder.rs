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
