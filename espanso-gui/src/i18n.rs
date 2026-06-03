/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// Loaded translations for one language.
#[derive(Debug, Clone, Deserialize)]
pub struct Translations {
    #[serde(rename = "_nav")]
    pub nav: Option<NavTranslations>,
    #[serde(rename = "_match_manager")]
    pub match_manager: Option<MatchManagerTranslations>,
    #[serde(rename = "_package_manager")]
    pub package_manager: Option<PackageManagerTranslations>,
    #[serde(rename = "_settings")]
    pub settings: Option<SettingsTranslations>,
    #[serde(rename = "_trigger_tester")]
    pub trigger_tester: Option<TriggerTesterTranslations>,
    #[serde(rename = "_stats_dashboard")]
    pub stats_dashboard: Option<StatsDashboardTranslations>,
    #[serde(rename = "_common")]
    pub common: Option<CommonTranslations>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavTranslations {
    pub match_manager: String,
    pub package_manager: String,
    pub settings: String,
    pub trigger_tester: String,
    pub stats_dashboard: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatchManagerTranslations {
    pub title: String,
    pub search_placeholder: String,
    pub add_button: String,
    pub import_button: String,
    pub export_button: String,
    pub delete_selected: String,
    pub select_all: String,
    pub type_filter_all: String,
    pub type_filter_text: String,
    pub type_filter_image: String,
    pub type_filter_markdown: String,
    pub type_filter_regex: String,
    pub type_filter_script: String,
    pub total_count: String,
    pub no_matches: String,
    pub no_matches_hint: String,
    pub confirm_delete: String,
    pub confirm_delete_multi: String,
    pub confirm_delete_hint: String,
    pub moved_to_trash: String,
    pub editor_title_new: String,
    pub editor_title_edit: String,
    pub editor_type: String,
    pub editor_trigger: String,
    pub editor_replace: String,
    pub editor_advanced: String,
    pub editor_word_boundary: String,
    pub editor_propagate_case: String,
    pub editor_inject_mode: String,
    pub editor_inject_mode_auto: String,
    pub editor_inject_mode_keys: String,
    pub editor_inject_mode_clipboard: String,
    pub editor_label: String,
    pub editor_search_terms: String,
    pub editor_cancel: String,
    pub editor_save: String,
    pub editor_trigger_conflict: String,
    pub editor_save_error: String,
    pub editor_save_success: String,
    pub image_preview_title: String,
    pub image_dimensions: String,
    pub image_size_kb: String,
    pub image_format: String,
    pub image_location: String,
    pub image_open_location: String,
    pub image_lost: String,
    pub image_lost_hint: String,
    pub image_drop_hint: String,
    pub image_copy_from_clipboard: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageManagerTranslations {
    pub title: String,
    pub tab_installed: String,
    pub tab_hub: String,
    pub tab_updates: String,
    pub search_placeholder: String,
    pub install_button: String,
    pub uninstall_button: String,
    pub update_button: String,
    pub update_all_button: String,
    pub version_label: String,
    pub installed_label: String,
    pub update_available: String,
    pub detail_title: String,
    pub detail_author: String,
    pub detail_stars: String,
    pub detail_version: String,
    pub detail_preview: String,
    pub detail_total_matches: String,
    pub detail_cancel: String,
    pub detail_install: String,
    pub installing: String,
    pub uninstalling: String,
    pub updating: String,
    pub install_done: String,
    pub uninstall_done: String,
    pub update_done: String,
    pub install_error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTranslations {
    pub title: String,
    pub tab_general: String,
    pub tab_injection: String,
    pub tab_shortcuts: String,
    pub tab_advanced: String,
    pub save_button: String,
    pub reset_button: String,
    pub save_success: String,
    pub save_error: String,
    pub enable: String,
    pub enable_desc: String,
    pub show_icon: String,
    pub show_icon_desc: String,
    pub show_notifications: String,
    pub show_notifications_desc: String,
    pub auto_restart: String,
    pub auto_restart_desc: String,
    pub stats_enabled: String,
    pub stats_enabled_desc: String,
    pub backend: String,
    pub backend_desc: String,
    pub backend_auto: String,
    pub backend_inject: String,
    pub backend_clipboard: String,
    pub clipboard_threshold: String,
    pub clipboard_threshold_desc: String,
    pub inject_delay: String,
    pub inject_delay_desc: String,
    pub key_delay: String,
    pub key_delay_desc: String,
    pub pre_paste_delay: String,
    pub pre_paste_delay_desc: String,
    pub paste_shortcut: String,
    pub paste_shortcut_desc: String,
    pub paste_shortcut_event_delay: String,
    pub paste_shortcut_event_delay_desc: String,
    pub preserve_clipboard: String,
    pub preserve_clipboard_desc: String,
    pub restore_clipboard_delay: String,
    pub restore_clipboard_delay_desc: String,
    pub toggle_key: String,
    pub toggle_key_desc: String,
    pub search_trigger: String,
    pub search_trigger_desc: String,
    pub search_shortcut: String,
    pub search_shortcut_desc: String,
    pub word_separators: String,
    pub word_separators_desc: String,
    pub backspace_limit: String,
    pub backspace_limit_desc: String,
    pub undo_backspace: String,
    pub undo_backspace_desc: String,
    pub apply_patch: String,
    pub apply_patch_desc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerTesterTranslations {
    pub title: String,
    pub input_label: String,
    pub input_placeholder: String,
    pub app_context_label: String,
    pub app_context_title: String,
    pub app_context_class: String,
    pub app_context_exec: String,
    pub test_button: String,
    pub match_result: String,
    pub matched_rule: String,
    pub match_type: String,
    pub rendered_output: String,
    pub no_match: String,
    pub variable_trace: String,
    pub var_name: String,
    pub var_type: String,
    pub var_params: String,
    pub var_output: String,
    pub elapsed: String,
    pub script_warning: String,
    pub worker_not_connected: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatsDashboardTranslations {
    pub title: String,
    pub period_7d: String,
    pub period_30d: String,
    pub period_all: String,
    pub total_expansions: String,
    pub active_matches: String,
    pub unused_matches: String,
    pub top_triggers: String,
    pub trigger: String,
    pub count: String,
    pub last_used: String,
    pub never_used: String,
    pub stats_disabled: String,
    pub enable_stats: String,
    pub no_data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonTranslations {
    pub cancel: String,
    pub confirm: String,
    pub close: String,
    pub save: String,
    pub delete: String,
    pub edit: String,
    pub search: String,
    pub loading: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub yes: String,
    pub no: String,
    pub ok: String,
    pub worker_connected: String,
    pub worker_disconnected: String,
    pub worker_restart_hint: String,
    pub theme_system: String,
    pub theme_light: String,
    pub theme_dark: String,
    pub import_success: String,
    pub import_partial: String,
    pub import_error: String,
    pub export_success: String,
    pub backup_success: String,
    pub version: String,
}

/// The currently loaded language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    ChineseSimplified,
    Japanese,
    Korean,
    German,
    French,
}

/// Global translations store, initialized on first access.
static TRANSLATIONS: OnceLock<HashMap<Language, Translations>> = OnceLock::new();

impl Language {
    /// Detect the system language.
    pub fn detect_system() -> Self {
        let locale = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_default();

        if locale.starts_with("zh_CN") || locale.starts_with("zh-Hans") {
            Language::ChineseSimplified
        } else if locale.starts_with("ja") {
            Language::Japanese
        } else if locale.starts_with("ko") {
            Language::Korean
        } else if locale.starts_with("de") {
            Language::German
        } else if locale.starts_with("fr") {
            Language::French
        } else {
            Language::English
        }
    }

    pub fn all() -> [Language; 6] {
        [
            Language::English,
            Language::ChineseSimplified,
            Language::Japanese,
            Language::Korean,
            Language::German,
            Language::French,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::ChineseSimplified => "简体中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
            Language::German => "Deutsch",
            Language::French => "Français",
        }
    }
}

/// Initialize translations by loading all YAML files.
fn init_translations() -> HashMap<Language, Translations> {
    let mut map = HashMap::new();

    let en: Translations =
        serde_norway::from_str(include_str!("../locales/en.yml"))
            .expect("Failed to parse English translations");
    map.insert(Language::English, en);

    let zh: Translations =
        serde_norway::from_str(include_str!("../locales/zh-CN.yml"))
            .expect("Failed to parse Chinese translations");
    map.insert(Language::ChineseSimplified, zh);

    // Other languages currently fall back to English
    // TODO: Add ja.yml, ko.yml, de.yml, fr.yml translation files

    map
}

/// Get translations for the given language.
pub fn get(lang: Language) -> &'static Translations {
    let store = TRANSLATIONS.get_or_init(init_translations);
    store
        .get(&lang)
        .or_else(|| store.get(&Language::English))
        .expect("English translations must always be available")
}
