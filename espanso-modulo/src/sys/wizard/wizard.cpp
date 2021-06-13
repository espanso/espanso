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

#define _UNICODE

#include "../common/common.h"
#include "../interop/interop.h"
#include "./wizard_gui.h"

#include <vector>
#include <memory>
#include <unordered_map>

const int WELCOME_PAGE_INDEX = 0;
const int MOVE_BUNDLE_PAGE_INDEX = WELCOME_PAGE_INDEX + 1;
const int LEGACY_VERSION_PAGE_INDEX = MOVE_BUNDLE_PAGE_INDEX + 1;
const int MIGRATE_PAGE_INDEX = LEGACY_VERSION_PAGE_INDEX + 1;
const int ADD_PATH_PAGE_INDEX = MIGRATE_PAGE_INDEX + 1;
const int ACCESSIBILITY_PAGE_INDEX = ADD_PATH_PAGE_INDEX + 1;
const int MAX_PAGE_INDEX = ACCESSIBILITY_PAGE_INDEX + 1; // Update if a new page is added at the end

WizardMetadata *metadata= nullptr;

// App Code

class WizardApp : public wxApp
{
public:
  virtual bool OnInit();
};

int find_next_page(int current_index) {
  int next_index = current_index + 1;
  if (next_index >= MAX_PAGE_INDEX) {
    return -1;
  }

  switch (next_index) {
    case WELCOME_PAGE_INDEX:
      if (metadata->is_welcome_page_enabled) {
        return WELCOME_PAGE_INDEX;
      }
    case MOVE_BUNDLE_PAGE_INDEX:
      if (metadata->is_move_bundle_page_enabled) {
        return MOVE_BUNDLE_PAGE_INDEX; 
      }
    case LEGACY_VERSION_PAGE_INDEX:
      if (metadata->is_legacy_version_page_enabled) {
        return LEGACY_VERSION_PAGE_INDEX; 
      }
    case MIGRATE_PAGE_INDEX:
      if (metadata->is_migrate_page_enabled) {
        return MIGRATE_PAGE_INDEX; 
      }
    case ADD_PATH_PAGE_INDEX:
      if (metadata->is_add_path_page_enabled) {
        return ADD_PATH_PAGE_INDEX; 
      }
    case ACCESSIBILITY_PAGE_INDEX:
      if (metadata->is_accessibility_page_enabled) {
        return ACCESSIBILITY_PAGE_INDEX; 
      }
  }

  return find_next_page(next_index);
}

class DerivedFrame : public WizardFrame
{
protected:
  void check_timer_tick( wxTimerEvent& event );
  void on_page_changed( wxBookCtrlEvent& event );
  void welcome_start_clicked(wxCommandEvent &event);

  void navigate_to_next_page_or_close();
  void change_default_button(int target_page);
public:
  DerivedFrame(wxWindow *parent);
};

DerivedFrame::DerivedFrame(wxWindow *parent)
    : WizardFrame(parent)
{
  // TODO: load images for accessibility page if on macOS

  this->welcome_version_text->SetLabel(wxString::Format("( version %s )", metadata->version));

  // Load the first page
  int page = find_next_page(-1);
  if (page >= 0) {
    this->m_simplebook->SetSelection(page);
    this->change_default_button(page);
  } else {
    Close(true);
  }
}

void DerivedFrame::navigate_to_next_page_or_close() {
  int current_page = this->m_simplebook->GetSelection();
  int page = find_next_page(current_page);
  if (page >= 0) {
    this->m_simplebook->SetSelection(page);
  } else {
    Close(true);
  }
}

void DerivedFrame::welcome_start_clicked(wxCommandEvent &event)
{
  this->navigate_to_next_page_or_close();
}

void DerivedFrame::check_timer_tick( wxTimerEvent& event ) {
  if (this->m_simplebook->GetSelection() == LEGACY_VERSION_PAGE_INDEX) {
    if (metadata->is_legacy_version_running) {
      if (metadata->is_legacy_version_running() == 0) {
        this->navigate_to_next_page_or_close();
      }
    } 
  }
}

void DerivedFrame::on_page_changed( wxBookCtrlEvent& event ) {
  int current_page = this->m_simplebook->GetSelection();
  this->change_default_button(current_page);
}

void DerivedFrame::change_default_button(int target_page) {
  switch (target_page) {
    case WELCOME_PAGE_INDEX:
    {
      this->welcome_start_button->SetDefault();
      break;
    }
    case MOVE_BUNDLE_PAGE_INDEX:
    {
      this->move_bundle_quit_button->SetDefault();
      break;
    }
    case MIGRATE_PAGE_INDEX:
    {
      this->migrate_backup_and_migrate_button->SetDefault();
      break;
    }
    case ADD_PATH_PAGE_INDEX:
    {
      this->add_path_continue_button->SetDefault();
      break;
    }
    case ACCESSIBILITY_PAGE_INDEX:
    {
      this->accessibility_enable_button->SetDefault();
      break;
    }
  }
}


bool WizardApp::OnInit()
{
  wxInitAllImageHandlers();
  DerivedFrame *frame = new DerivedFrame(NULL);
  
  if (metadata->window_icon_path) {
    setFrameIcon(metadata->window_icon_path, frame);
  }

  frame->Show(true);

  Activate(frame);

  return true;
}

extern "C" void interop_show_wizard(WizardMetadata * _metadata)
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  metadata = _metadata;

  wxApp::SetInstance(new WizardApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);
}