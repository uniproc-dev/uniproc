use crate::write_if_changed;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::{Table, Value};

const ICON_MANIFEST_ENV: &str = "FORSL_ICONS_OFFLINE";
const SOURCE_KEYS: &[&str] = &["file", "iconify", "url", "glyph"];
const PLATFORM_KEYS: &[&str] = &["windows-ico"];
const ENTRY_OVERRIDE_KEYS: &[&str] = &["root"];

#[derive(Clone, Debug)]
pub struct IconManifest {
    manifest_path: PathBuf,
    workspace_root: PathBuf,
    entries: Vec<IconEntry>,
}

#[derive(Clone, Debug)]
pub struct IconEntry {
    key: String,
    source: IconSource,
    windows_ico: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub enum IconSource {
    File(PathBuf),
    Iconify(String),
    Url(String),
    Glyph(String),
}

#[derive(Clone, Debug)]
pub struct MaterializedIcon {
    pub key: String,
    pub property_name: String,
    pub backend: MaterializedIconBackend,
}

#[derive(Clone, Debug)]
pub enum MaterializedIconBackend {
    Image { path: PathBuf },
    Glyph { font_family: String, text: String },
}

#[derive(Clone, Debug, Default)]
struct IconManifestDefaults {
    root: Option<PathBuf>,
}

pub struct IconBuild {
    manifest_path: PathBuf,
    build_out_dir: PathBuf,
    materialized_root: PathBuf,
    slint_global_out: Option<PathBuf>,
    rust_registry_out: Option<PathBuf>,
    emit_shared_bundle: bool,
}

impl IconManifest {
    pub fn entries(&self) -> &[IconEntry] {
        &self.entries
    }

    pub fn entry(&self, key: &str) -> Option<&IconEntry> {
        self.entries.iter().find(|entry| entry.key == key)
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}

impl IconBuild {
    pub fn new(manifest_path: impl Into<PathBuf>) -> Self {
        let build_out_dir = out_dir();
        Self {
            manifest_path: manifest_path.into(),
            materialized_root: build_out_dir.clone(),
            build_out_dir,
            slint_global_out: None,
            rust_registry_out: None,
            emit_shared_bundle: false,
        }
    }

    pub fn auto() -> Self {
        Self::new(workspace_manifest_path())
    }

    pub fn out_dir(mut self, out_dir: impl Into<PathBuf>) -> Self {
        self.build_out_dir = out_dir.into();
        self
    }

    pub fn materialized_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.materialized_root = root.into();
        self
    }

    pub fn emit_slint_global(mut self, out_file: impl Into<PathBuf>) -> Self {
        self.slint_global_out = Some(out_file.into());
        self
    }

    pub fn emit_rust_registry(mut self, out_file: impl Into<PathBuf>) -> Self {
        self.rust_registry_out = Some(out_file.into());
        self
    }

    pub fn emit_shared_bundle(mut self) -> Self {
        self.emit_shared_bundle = true;
        self.materialized_root = shared_output_root();
        self
    }

    pub fn run(self) {
        let manifest = load_icon_manifest(&self.manifest_path);

        let needs_materialization = self.emit_shared_bundle
            || self.slint_global_out.is_some()
            || self.rust_registry_out.is_some();

        let materialized = if needs_materialization {
            Some(materialize_icons(&manifest, &self.materialized_root))
        } else {
            None
        };

        if self.emit_shared_bundle {
            let shared_slint_out = shared_output_root().join("shared").join("icons.slint");
            let icons = materialized
                .as_ref()
                .expect("Materialized icons must exist if emit_shared_bundle is true");

            generate_slint_icon_global_from_materialized(&shared_slint_out, icons);
            emit_shared_windows_artifacts(&manifest);
        }

        if let Some(out_file) = &self.slint_global_out {
            let icons = materialized
                .as_ref()
                .expect("Materialized icons missing for slint_global_out");
            generate_slint_icon_global_from_materialized(out_file, icons);
        }

        if let Some(out_file) = &self.rust_registry_out {
            let icons = materialized
                .as_ref()
                .expect("Materialized icons missing for rust_registry_out");
            generate_rust_icon_registry_from_materialized(&self.manifest_path, out_file, icons);
        }
    }
}

impl IconEntry {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn source(&self) -> &IconSource {
        &self.source
    }

    pub fn windows_ico(&self) -> Option<&Path> {
        self.windows_ico.as_deref()
    }
}

pub fn load_icon_manifest(manifest_path: &Path) -> IconManifest {
    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!("cargo:rerun-if-env-changed={ICON_MANIFEST_ENV}");

    let content = fs::read_to_string(manifest_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", manifest_path.display()));
    let table: Table = content
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", manifest_path.display()));

    let workspace_root = find_workspace_root(manifest_path).unwrap_or_else(|| {
        panic!(
            "Could not find workspace root for {}",
            manifest_path.display()
        )
    });
    let defaults = parse_defaults(&table, &workspace_root);

    let mut entries = Vec::new();
    collect_entries(Vec::new(), &table, &workspace_root, &defaults, &mut entries);
    entries.sort_by(|a, b| a.key.cmp(&b.key));

    IconManifest {
        manifest_path: manifest_path.to_path_buf(),
        workspace_root,
        entries,
    }
}

fn generate_slint_icon_global_from_materialized(out_file: &Path, icons: &[MaterializedIcon]) {
    let components = icons
        .iter()
        .map(|icon| {
            let component_name = slint_component_name(&icon.key);
            match &icon.backend {
                MaterializedIconBackend::Image { path } => {
                    let path = relative_or_absolute_icon_path(out_file, path);
                    let path = escape_slint_string(&path.replace('\\', "/"));
                    format!(
                        "export component {component_name} inherits Image {{\n    source: @image-url(\"{path}\");\n}}\n"
                    )
                }
                MaterializedIconBackend::Glyph { font_family, text } => {
                    let font_family = escape_slint_string(font_family);
                    let text = escape_slint_string(text);
                    format!(
                        "export component {component_name} inherits Text {{\n    text: \"{text}\";\n    font-family: \"{font_family}\";\n    horizontal-alignment: center;\n    vertical-alignment: center;\n    font-size: Math.min(self.width, self.height);\n    wrap: no-wrap;\n}}\n"
                    )
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let dynamic_component_cases = icons
        .iter()
        .map(|icon| {
            let component_name = slint_component_name(&icon.key);
            match &icon.backend {
                MaterializedIconBackend::Image { .. } => format!(
                    "    if (root.name == \"{key}\"): {component_name} {{\n        width: parent.width;\n        height: parent.height;\n        image-fit: contain;\n        colorize: root.colorize;\n    }}",
                    key = icon.key
                ),
                MaterializedIconBackend::Glyph { .. } => format!(
                    "    if (root.name == \"{key}\"): {component_name} {{\n        width: parent.width;\n        height: parent.height;\n        color: root.colorize;\n    }}",
                    key = icon.key
                ),
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let generated = format!(
        "// AUTO-GENERATED — do not edit manually\n{components}\nexport component Icon inherits Rectangle {{\n    in property <string> name;\n    in property <color> colorize: transparent;\n\n{dynamic_component_cases}\n}}"
    );
    write_if_changed(out_file, &generated);
}

pub fn generate_rust_icon_registry(manifest_path: &Path, out_file: &Path, build_out_dir: &Path) {
    let manifest = load_icon_manifest(manifest_path);
    let icons = materialize_icons(&manifest, build_out_dir);
    generate_rust_icon_registry_from_materialized(manifest_path, out_file, &icons);
}

fn generate_rust_icon_registry_from_materialized(
    manifest_path: &Path,
    out_file: &Path,
    icons: &[MaterializedIcon],
) {
    let key_consts = icons
        .iter()
        .map(|icon| {
            format!(
                "    pub const {}: IconKey = IconKey::new(\"{}\");",
                rust_const_name(&icon.key),
                icon.key
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let all_keys = icons
        .iter()
        .map(|icon| format!("keys::{}", rust_const_name(&icon.key)))
        .collect::<Vec<_>>()
        .join(", ");

    let key_to_name_arms = icons
        .iter()
        .map(|icon| {
            format!(
                "        keys::{} => Some(\"{}\"),",
                rust_const_name(&icon.key),
                icon.key
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let from_name_arms = icons
        .iter()
        .map(|icon| {
            format!(
                "        \"{}\" => Some(keys::{}),",
                icon.key,
                rust_const_name(&icon.key)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let family_entries = collect_families(icons);
    let unique_families = unique_family_names(&family_entries);
    let family_consts = unique_families
        .iter()
        .map(|family| {
            format!(
                "    pub const {}: IconFamily = IconFamily::new(\"{}\");",
                rust_const_name(family),
                family
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let variant_entries = collect_variants(icons);
    let variant_consts = variant_entries
        .iter()
        .map(|variant| {
            format!(
                "    pub const {}: IconVariant = IconVariant::new(\"{}\");",
                rust_const_name(variant),
                variant
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let from_family_variant_arms = family_entries
        .iter()
        .map(|entry| {
            let key = format!("keys::{}", rust_const_name(&entry.key));
            let family = format!("families::{}", rust_const_name(&entry.family));
            let variant = entry
                .variant
                .as_ref()
                .map(|variant| format!("Some(variants::{})", rust_const_name(variant)))
                .unwrap_or_else(|| "None".to_string());
            format!("        ({family}, {variant}) => Some({key}),")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let from_dynamic_family_variant_arms = family_entries
        .iter()
        .map(|entry| {
            let key = format!("keys::{}", rust_const_name(&entry.key));
            let variant = entry
                .variant
                .as_deref()
                .map(|variant| format!("Some(\"{variant}\")"))
                .unwrap_or_else(|| "None".to_string());
            format!("        (\"{}\", {variant}) => Some({key}),", entry.family)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let arms = icons
        .iter()
        .map(|icon| match &icon.backend {
            MaterializedIconBackend::Image { path } => {
                let asset_path = path.to_string_lossy().replace('\\', "\\\\");
                format!(
                    "        keys::{} => Some(include_bytes!(\"{asset_path}\") as &'static [u8]),",
                    rust_const_name(&icon.key)
                )
            }
            MaterializedIconBackend::Glyph { .. } => {
                format!("        keys::{} => None,", rust_const_name(&icon.key))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let generated = format!(
        r#"// AUTO-GENERATED from {}
use framework::icons::{{IconFamily, IconKey, IconRef, IconVariant}};
use slint::Image;

pub mod keys {{
    use super::IconKey;

{key_consts}
}}

pub mod families {{
    use super::IconFamily;

{family_consts}
}}

pub mod variants {{
    use super::IconVariant;

{variant_consts}
}}

pub const ALL_KEYS: &[IconKey] = &[{all_keys}];

pub fn name_for_key(key: IconKey) -> Option<&'static str> {{
    match key {{
{key_to_name_arms}
        _ => None,
    }}
}}

pub fn key_from_name(name: &str) -> Option<IconKey> {{
    match name {{
{from_name_arms}
        _ => None,
    }}
}}

pub fn key_from_family_variant(family: IconFamily, variant: Option<IconVariant>) -> Option<IconKey> {{
    match (family, variant) {{
{from_family_variant_arms}
        _ => None,
    }}
}}

pub fn key_from_dynamic_family_variant(family: &str, variant: Option<&str>) -> Option<IconKey> {{
    match (family, variant) {{
{from_dynamic_family_variant_arms}
        _ => None,
    }}
}}

pub fn key_from_ref(icon: IconRef<'_>) -> Option<IconKey> {{
    match icon {{
        IconRef::Key(key) => Some(key),
        IconRef::Name(name) => key_from_name(name),
        IconRef::FamilyVariant {{ family, variant }} => key_from_family_variant(family, variant),
        IconRef::DynamicFamilyVariant {{ family, variant }} => key_from_dynamic_family_variant(family, variant),
    }}
}}

pub fn bytes_for(key: IconKey) -> Option<&'static [u8]> {{
    match key {{
{arms}
        _ => None,
    }}
}}

pub struct Icons;

impl Icons {{
    pub fn get_key(key: IconKey) -> Image {{
        resolve(key)
    }}

    pub fn resolve<'a>(icon: impl Into<IconRef<'a>>) -> Image {{
        resolve(icon)
    }}
}}

pub fn resolve<'a>(icon: impl Into<IconRef<'a>>) -> Image {{
    let Some(key) = key_from_ref(icon.into()) else {{
        return Image::default();
    }};
    let Some(bytes) = bytes_for(key) else {{
        return Image::default();
    }};
    Image::load_from_svg_data(bytes).unwrap_or_default()
}}
"#,
        manifest_path.display()
    );

    write_if_changed(out_file, &generated);
}

pub fn materialize_icons(manifest: &IconManifest, build_out_dir: &Path) -> Vec<MaterializedIcon> {
    let icons_dir = build_out_dir.join("icons");
    let _ = fs::create_dir_all(&icons_dir);

    manifest
        .entries()
        .iter()
        .map(|entry| {
            let backend = match entry.source() {
                IconSource::File(path) => {
                    let output_path = icons_dir.join(format!("{}.svg", output_stem(entry.key())));
                    copy_if_changed(&canonicalize_existing(path), &output_path);
                    MaterializedIconBackend::Image { path: output_path }
                }
                IconSource::Iconify(id) => {
                    let output_path = icons_dir.join(format!("{}.svg", output_stem(entry.key())));
                    let url = iconify_url(id);
                    let cached = download_or_cache(&url);
                    copy_if_changed(&cached, &output_path);
                    MaterializedIconBackend::Image { path: output_path }
                }
                IconSource::Url(url) => {
                    let output_path = icons_dir.join(format!("{}.svg", output_stem(entry.key())));
                    let cached = download_or_cache(url);
                    copy_if_changed(&cached, &output_path);
                    MaterializedIconBackend::Image { path: output_path }
                }
                IconSource::Glyph(glyph) => {
                    let (font_family, text) = parse_glyph(glyph, entry.key());
                    MaterializedIconBackend::Glyph { font_family, text }
                }
            };

            MaterializedIcon {
                key: entry.key().to_string(),
                property_name: slint_property_name(entry.key()),
                backend,
            }
        })
        .collect()
}

fn collect_entries(
    path: Vec<String>,
    table: &Table,
    workspace_root: &Path,
    defaults: &IconManifestDefaults,
    acc: &mut Vec<IconEntry>,
) {
    let is_entry = table.keys().any(|key| {
        SOURCE_KEYS.contains(&key.as_str())
            || PLATFORM_KEYS.contains(&key.as_str())
            || ENTRY_OVERRIDE_KEYS.contains(&key.as_str())
    });

    if is_entry {
        if table.values().any(|value| matches!(value, Value::Table(_))) {
            panic!(
                "Icon manifest entry {:?} may not contain nested tables once source fields are present",
                path
            );
        }
        acc.push(parse_entry(path, table, workspace_root, defaults));
        return;
    }

    for (key, value) in table {
        if path.is_empty() && key == "defaults" {
            continue;
        }
        let Value::Table(sub_table) = value else {
            panic!(
                "Unexpected scalar at manifest group {:?}: key `{key}` must be a table",
                path
            );
        };
        let mut next_path = path.clone();
        next_path.push(key.to_string());
        collect_entries(next_path, sub_table, workspace_root, defaults, acc);
    }
}

fn parse_entry(
    path: Vec<String>,
    table: &Table,
    workspace_root: &Path,
    defaults: &IconManifestDefaults,
) -> IconEntry {
    let key = path.join(".");
    let root_override =
        string_field(table, "root").map(|value| resolve_workspace_path(workspace_root, value));
    let effective_root = root_override
        .as_deref()
        .or(defaults.root.as_deref())
        .unwrap_or(workspace_root);

    let file = string_field(table, "file").map(|value| resolve_entry_path(effective_root, value));
    let iconify = string_field(table, "iconify").map(str::to_string);
    let url = string_field(table, "url").map(str::to_string);
    let glyph = string_field(table, "glyph").map(str::to_string);

    let source_count = usize::from(file.is_some())
        + usize::from(iconify.is_some())
        + usize::from(url.is_some())
        + usize::from(glyph.is_some());
    if source_count != 1 {
        panic!("Icon manifest entry `{key}` must define exactly one of file/iconify/url/glyph");
    }

    let source = if let Some(path) = file {
        IconSource::File(path)
    } else if let Some(id) = iconify {
        IconSource::Iconify(id)
    } else if let Some(url) = url {
        IconSource::Url(url)
    } else {
        IconSource::Glyph(glyph.expect("glyph source should exist"))
    };

    let windows_ico =
        string_field(table, "windows-ico").map(|value| resolve_entry_path(effective_root, value));

    let allowed_fields = SOURCE_KEYS
        .iter()
        .chain(PLATFORM_KEYS.iter())
        .chain(ENTRY_OVERRIDE_KEYS.iter())
        .copied()
        .collect::<Vec<_>>();
    for field in table.keys() {
        if !allowed_fields.contains(&field.as_str()) {
            panic!("Unsupported field `{field}` in icon manifest entry `{key}`");
        }
    }

    IconEntry {
        key,
        source,
        windows_ico,
    }
}

fn parse_defaults(table: &Table, workspace_root: &Path) -> IconManifestDefaults {
    let Some(defaults) = table.get("defaults") else {
        return IconManifestDefaults::default();
    };
    let defaults = defaults
        .as_table()
        .unwrap_or_else(|| panic!("`[defaults]` must be a TOML table"));

    for key in defaults.keys() {
        if key != "root" {
            panic!("Unsupported field `{key}` in `[defaults]`");
        }
    }

    IconManifestDefaults {
        root: string_field(defaults, "root")
            .map(|value| resolve_workspace_path(workspace_root, value)),
    }
}

fn string_field<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table.get(key).map(|value| {
        value
            .as_str()
            .unwrap_or_else(|| panic!("Field `{key}` must be a string"))
    })
}

fn resolve_workspace_path(workspace_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn resolve_entry_path(root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn output_stem(key: &str) -> String {
    key.replace(['.', '_'], "-")
}

fn parse_glyph(glyph: &str, key: &str) -> (String, String) {
    let Some((font_family, codepoint)) = glyph.split_once(':') else {
        panic!(
            "Glyph manifest entry `{key}` must use `font-family:codepoint` format, got `{glyph}`"
        );
    };

    let font_family = font_family.trim();
    let codepoint = codepoint.trim();
    if font_family.is_empty() || codepoint.is_empty() {
        panic!(
            "Glyph manifest entry `{key}` must define both font family and codepoint, got `{glyph}`"
        );
    }

    let text = if codepoint.chars().count() == 1 {
        codepoint.to_string()
    } else {
        let normalized = codepoint
            .strip_prefix("U+")
            .or_else(|| codepoint.strip_prefix("u+"))
            .unwrap_or(codepoint);
        let scalar = u32::from_str_radix(normalized, 16).unwrap_or_else(|_| {
            panic!("Glyph manifest entry `{key}` has invalid codepoint `{codepoint}`")
        });
        let ch = char::from_u32(scalar).unwrap_or_else(|| {
            panic!("Glyph manifest entry `{key}` has non-scalar codepoint `{codepoint}`")
        });
        ch.to_string()
    };

    (font_family.to_string(), text)
}

fn slint_property_name(key: &str) -> String {
    output_stem(key)
}

fn slint_component_name(key: &str) -> String {
    format!("{}Icon", rust_variant_name(key))
}

fn rust_variant_name(key: &str) -> String {
    let mut result = String::new();
    for segment in key.split(['.', '-', '_']) {
        if segment.is_empty() {
            continue;
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }

    if result.is_empty() {
        "Unknown".to_string()
    } else if result.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("Icon{result}")
    } else {
        result
    }
}

fn rust_const_name(key: &str) -> String {
    key.replace(['.', '-'], "_").to_ascii_uppercase()
}

#[derive(Clone)]
struct FamilyEntry {
    family: String,
    variant: Option<String>,
    key: String,
}

fn collect_families(icons: &[MaterializedIcon]) -> Vec<FamilyEntry> {
    let mut entries = Vec::new();
    for icon in icons {
        if let Some((family, variant)) = split_family_variant(&icon.key) {
            entries.push(FamilyEntry {
                family,
                variant: Some(variant),
                key: icon.key.clone(),
            });
        }
    }
    entries.sort_by(|a, b| a.family.cmp(&b.family).then(a.key.cmp(&b.key)));
    entries
}

fn collect_variants(icons: &[MaterializedIcon]) -> Vec<String> {
    let mut variants = collect_families(icons)
        .into_iter()
        .filter_map(|entry| entry.variant)
        .collect::<Vec<_>>();
    variants.sort();
    variants.dedup();
    variants
}

fn unique_family_names(entries: &[FamilyEntry]) -> Vec<String> {
    let mut families = entries
        .iter()
        .map(|entry| entry.family.clone())
        .collect::<Vec<_>>();
    families.sort();
    families.dedup();
    families
}

fn split_family_variant(key: &str) -> Option<(String, String)> {
    const KNOWN_VARIANTS: &[&str] = &[
        "filled",
        "regular",
        "outlined",
        "outline",
        "rounded",
        "primary",
        "secondary",
        "small",
        "large",
    ];

    if let Some((prefix, suffix)) = key.rsplit_once('.') {
        if KNOWN_VARIANTS.contains(&suffix) {
            return Some((prefix.to_string(), suffix.to_string()));
        }
    }

    if let Some((prefix, suffix)) = key.rsplit_once('-') {
        if KNOWN_VARIANTS.contains(&suffix) {
            return Some((prefix.to_string(), suffix.to_string()));
        }
    }

    None
}

fn copy_if_changed(src: &Path, dest: &Path) {
    let src_bytes =
        fs::read(src).unwrap_or_else(|e| panic!("Failed to read {}: {e}", src.display()));
    let existing = fs::read(dest).unwrap_or_default();
    if existing != src_bytes {
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(dest, src_bytes)
            .unwrap_or_else(|e| panic!("Failed to write {}: {e}", dest.display()));
    }
}

fn canonicalize_existing(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|e| panic!("Failed to resolve {}: {e}", path.display()))
}

fn iconify_url(id: &str) -> String {
    let (set, name) = id
        .split_once(':')
        .unwrap_or_else(|| panic!("Iconify source must be `<set>:<name>`, got `{id}`"));
    format!("https://api.iconify.design/{set}/{name}.svg")
}

fn download_or_cache(url: &str) -> PathBuf {
    let cache_dir = cache_dir().join("remote");
    let _ = fs::create_dir_all(&cache_dir);

    let digest = sha256_hex(url);
    let cache_path = cache_dir.join(format!("{digest}.svg"));
    if cache_path.exists() {
        return cache_path;
    }

    if env::var_os(ICON_MANIFEST_ENV).is_some() {
        panic!("Icon `{url}` is missing from cache and {ICON_MANIFEST_ENV}=1 forbids downloads");
    }

    let response = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("Failed to download `{url}`: {e}"));
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .unwrap_or_else(|e| panic!("Failed to read `{url}`: {e}"));
    fs::write(&cache_path, bytes)
        .unwrap_or_else(|e| panic!("Failed to write cache file {}: {e}", cache_path.display()));
    cache_path
}

fn cache_dir() -> PathBuf {
    shared_output_root().join("cache")
}

pub fn compile_windows_resources_from_shared() {
    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let icon_path = shared_windows_icon_path();
    if !icon_path.exists() {
        panic!(
            "Shared Windows icon artifact is missing: {}. Ensure context icon entry ran first.",
            icon_path.display()
        );
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(
        icon_path
            .to_str()
            .unwrap_or_else(|| panic!("Non-UTF8 icon path: {}", icon_path.display())),
    );
    res.compile().unwrap();
}

pub fn shared_output_root() -> PathBuf {
    let out_dir = out_dir();
    for ancestor in out_dir.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("build") {
            let profile_dir = ancestor
                .parent()
                .unwrap_or_else(|| panic!("build directory should have a profile parent"));
            return profile_dir.join("forsl-icons");
        }
    }

    panic!(
        "Could not derive shared icon output root from {}",
        out_dir.display()
    );
}

pub fn shared_slint_include_paths() -> Vec<PathBuf> {
    let root = shared_output_root();
    vec![root.clone(), root.join("shared")]
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn escape_slint_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

trait ReadToEnd {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize>;
}

impl<T: io::Read> ReadToEnd for T {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        io::Read::read_to_end(self, buf)
    }
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"))
}

fn workspace_manifest_path() -> PathBuf {
    find_workspace_root_from_cwd()
        .unwrap_or_else(|| {
            panic!(
                "Could not find workspace root from {}",
                current_dir().display()
            )
        })
        .join("icons.toml")
}

fn find_workspace_root(manifest_path: &Path) -> Option<PathBuf> {
    let start = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    find_workspace_root_from(start)
}

fn find_workspace_root_from_cwd() -> Option<PathBuf> {
    find_workspace_root_from(&current_dir())
}

fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = canonicalize_or_self(start);
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml).ok()?;
            if content.contains("[workspace]") {
                return Some(current);
            }
        }

        current = current.parent()?.to_path_buf();
    }
}

fn current_dir() -> PathBuf {
    env::current_dir().expect("Current directory should be available")
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn emit_shared_windows_artifacts(manifest: &IconManifest) {
    if let Some(icon_path) = manifest
        .entry("sys.application.icon")
        .and_then(|entry| entry.windows_ico())
    {
        let out_path = shared_windows_icon_path();
        copy_if_changed(&canonicalize_existing(icon_path), &out_path);
    }
}

fn shared_windows_icon_path() -> PathBuf {
    shared_output_root()
        .join("platform")
        .join("windows")
        .join("application.ico")
}

fn relative_or_absolute_icon_path(base_file: &Path, target: &Path) -> String {
    if let Some(base_dir) = base_file.parent() {
        if let Ok(relative) = pathdiff::diff_paths(target, base_dir).ok_or(()) {
            return relative.to_string_lossy().to_string();
        }
    }

    target.to_string_lossy().to_string()
}
