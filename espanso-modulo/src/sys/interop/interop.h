/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

// FORM

typedef enum FieldType {
  ROW,
  LABEL,
  TEXT,
  CHOICE,
  CHECKBOX,
} FieldType;

typedef struct LabelMetadata {
  const char *text;
} LabelMetadata;

typedef struct TextMetadata {
  const char *defaultText;
  const int multiline;
} TextMetadata;

typedef enum ChoiceType {
  DROPDOWN,
  LIST,
} ChoiceType;

typedef struct ChoiceMetadata {
  const char * const * values;
  const int valueSize;
  const char *defaultValue;
  const ChoiceType choiceType;
} ChoiceMetadata;

typedef struct FieldMetadata {
  const char * id;
  FieldType fieldType;
  const void * specific;
} FieldMetadata;

typedef struct RowMetadata {
  const FieldMetadata *fields;
  const int fieldSize;
} RowMetadata;

typedef struct FormMetadata {
  const char *windowTitle;
  const char *iconPath;
  const FieldMetadata *fields;
  const int fieldSize;
} FormMetadata;

typedef struct ValuePair {
  const char *id;
  const char *value;
} ValuePair;

// SEARCH

typedef struct SearchItem {
  const char *id;
  const char *label;
  const char *trigger;
} SearchItem;

typedef struct SearchResults {
  const SearchItem * items;
  const int itemSize;
} SearchResults;

typedef struct SearchMetadata {
  const char *windowTitle;
  const char *iconPath;
} SearchMetadata;

// WIZARD

const int MIGRATE_RESULT_SUCCESS = 0;
const int MIGRATE_RESULT_CLEAN_FAILURE = 1;
const int MIGRATE_RESULT_DIRTY_FAILURE = 2;
const int MIGRATE_RESULT_UNKNOWN_FAILURE = 3;

typedef struct WizardMetadata {
  const char *version;

  const int is_welcome_page_enabled; 
  const int is_move_bundle_page_enabled; 
  const int is_legacy_version_page_enabled; 
  const int is_migrate_page_enabled; 
  const int is_add_path_page_enabled; 
  const int is_accessibility_page_enabled; 

  const char *window_icon_path;
  const char *welcome_image_path;
  const char *accessibility_image_1_path;
  const char *accessibility_image_2_path;

  // METHODS
  int (*is_legacy_version_running)();
  int (*backup_and_migrate)();
  int (*add_to_path)();
  int (*enable_accessibility)();
  int (*is_accessibility_enabled)();
  void (*on_completed)();
} WizardMetadata;

typedef struct WelcomeMetadata {
  const char *window_icon_path;
  const char *tray_image_path;

  // METHODS
  int (*dont_show_again_changed)(int);
} WelcomeMetadata;

