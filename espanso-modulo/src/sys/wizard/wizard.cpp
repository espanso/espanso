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
const int WRONG_EDITION_PAGE_INDEX = LEGACY_VERSION_PAGE_INDEX + 1;
const int MIGRATE_PAGE_INDEX = WRONG_EDITION_PAGE_INDEX + 1;
const int AUTO_START_PAGE_INDEX = MIGRATE_PAGE_INDEX + 1;
const int ADD_PATH_PAGE_INDEX = AUTO_START_PAGE_INDEX + 1;
const int ACCESSIBILITY_PAGE_INDEX = ADD_PATH_PAGE_INDEX + 1;
const int MAX_PAGE_INDEX = ACCESSIBILITY_PAGE_INDEX + 1; // Update if a new page is added at the end

WizardMetadata *wizard_metadata = nullptr;
int completed_successfully = 0;

// App Code

class WizardApp : public wxApp
{
public:
  virtual bool OnInit();
};

int find_next_page(int current_index)
{
  int next_index = current_index + 1;
  if (next_index >= MAX_PAGE_INDEX)
  {
    return -1;
  }

  switch (next_index)
  {
  case WELCOME_PAGE_INDEX:
    if (wizard_metadata->is_welcome_page_enabled)
    {
      return WELCOME_PAGE_INDEX;
    }
  case MOVE_BUNDLE_PAGE_INDEX:
    if (wizard_metadata->is_move_bundle_page_enabled)
    {
      return MOVE_BUNDLE_PAGE_INDEX;
    }
  case LEGACY_VERSION_PAGE_INDEX:
    if (wizard_metadata->is_legacy_version_page_enabled)
    {
      return LEGACY_VERSION_PAGE_INDEX;
    }
  case WRONG_EDITION_PAGE_INDEX:
    if (wizard_metadata->is_wrong_edition_page_enabled)
    {
      return WRONG_EDITION_PAGE_INDEX;
    }
  case MIGRATE_PAGE_INDEX:
    if (wizard_metadata->is_migrate_page_enabled)
    {
      return MIGRATE_PAGE_INDEX;
    }
  case AUTO_START_PAGE_INDEX:
    if (wizard_metadata->is_auto_start_page_enabled)
    {
      return AUTO_START_PAGE_INDEX;
    }
  case ADD_PATH_PAGE_INDEX:
    if (wizard_metadata->is_add_path_page_enabled)
    {
      return ADD_PATH_PAGE_INDEX;
    }
  case ACCESSIBILITY_PAGE_INDEX:
    if (wizard_metadata->is_accessibility_page_enabled)
    {
      return ACCESSIBILITY_PAGE_INDEX;
    }
  }

  return find_next_page(next_index);
}

class DerivedFrame : public WizardFrame
{
protected:
  void check_timer_tick(wxTimerEvent &event);
  void on_page_changed(wxBookCtrlEvent &event);
  void welcome_start_clicked(wxCommandEvent &event);
  void migrate_button_clicked(wxCommandEvent &event);
  void migrate_compatibility_mode_clicked(wxCommandEvent &event);
	void auto_start_continue_clicked( wxCommandEvent& event );
	void add_path_continue_clicked( wxCommandEvent& event );
	void accessibility_enable_clicked( wxCommandEvent& event );
	void quit_espanso_clicked( wxCommandEvent& event );

  void navigate_to_next_page_or_close();
  void change_default_button(int target_page);

public:
  DerivedFrame(wxWindow *parent);
};

DerivedFrame::DerivedFrame(wxWindow *parent)
    : WizardFrame(parent)
{
  // Welcome images

  if (wizard_metadata->welcome_image_path)
  {
    wxBitmap welcomeBitmap = wxBitmap(wizard_metadata->welcome_image_path, wxBITMAP_TYPE_PNG);
    this->welcome_image->SetBitmap(welcomeBitmap);
  }

  this->welcome_version_text->SetLabel(wxString::Format("( version %s )", wizard_metadata->version));

  // Accessiblity images

  if (wizard_metadata->accessibility_image_1_path)
  {
    wxBitmap accessiblityImage1 = wxBitmap(wizard_metadata->accessibility_image_1_path, wxBITMAP_TYPE_PNG);
    this->accessibility_image1->SetBitmap(accessiblityImage1);
  }
  if (wizard_metadata->accessibility_image_2_path)
  {
    wxBitmap accessiblityImage2 = wxBitmap(wizard_metadata->accessibility_image_2_path, wxBITMAP_TYPE_PNG);
    this->accessibility_image2->SetBitmap(accessiblityImage2);
  }

  // Wrong edition
  if (wizard_metadata->is_wrong_edition_page_enabled) {
    if (wizard_metadata->detected_os == DETECTED_OS_X11) {
      this->wrong_edition_description_x11->Hide();
    }
    if (wizard_metadata->detected_os == DETECTED_OS_WAYLAND) {
      this->wrong_edition_description_wayland->Hide();
    }
  }

  // Load the first page
  int page = find_next_page(-1);
  if (page >= 0)
  {
    this->m_simplebook->SetSelection(page);
    this->change_default_button(page);
  }
  else
  {
    Close(true);
  }
}

void DerivedFrame::navigate_to_next_page_or_close()
{
  int current_page = this->m_simplebook->GetSelection();
  int page = find_next_page(current_page);
  if (page >= 0)
  {
    this->m_simplebook->SetSelection(page);
  }
  else
  {
    if (wizard_metadata->on_completed) {
      wizard_metadata->on_completed();
      completed_successfully = 1;
    }

    Close(true);
  }
}

void DerivedFrame::welcome_start_clicked(wxCommandEvent &event)
{
  this->navigate_to_next_page_or_close();
}

void DerivedFrame::migrate_compatibility_mode_clicked(wxCommandEvent &event)
{
  this->navigate_to_next_page_or_close();
}

void DerivedFrame::migrate_button_clicked(wxCommandEvent &event)
{
  if (wizard_metadata->backup_and_migrate)
  {
    int result = wizard_metadata->backup_and_migrate();
    if (result == MIGRATE_RESULT_SUCCESS)
    {
      this->navigate_to_next_page_or_close();
    }
    else if (result == MIGRATE_RESULT_CLEAN_FAILURE)
    {
      wxMessageBox(wxT("An error occurred during the migration, but your old files were not modified.\n\nPlease run 'espanso log' in a terminal for more information."), wxT("Migration error"), wxICON_ERROR);
    }
    else if (result == MIGRATE_RESULT_DIRTY_FAILURE)
    {
      wxMessageBox(wxT("An error occurred during the migration and espanso couldn't complete the process. Some configuration files might be missing, but you'll find the backup in the Documents folder.\n\nPlease run 'espanso log' in a terminal for more information."), wxT("Migration error"), wxICON_ERROR);
    }
    else if (result == MIGRATE_RESULT_UNKNOWN_FAILURE)
    {
      wxMessageBox(wxT("An error occurred during the migration.\n\nPlease run 'espanso log' in a terminal for more information."), wxT("Migration error"), wxICON_ERROR);
    }
  }
}

void DerivedFrame::auto_start_continue_clicked( wxCommandEvent& event ) {
  if (!auto_start_checkbox->IsChecked()) {
    if (wizard_metadata->auto_start)
    {
      wizard_metadata->auto_start(0);
    }
    this->navigate_to_next_page_or_close();
    return;
  } 

  if (wizard_metadata->auto_start)
  {
    while (true) {
      int result = wizard_metadata->auto_start(1);
      if (result == 1)
      {
        this->navigate_to_next_page_or_close();
        return;
      }
      else 
      {
        wxMessageDialog* dialog = new wxMessageDialog(this,
          "An error occurred while registering Espanso as a service, please check the logs for more information.\nDo you want to retry? You can always configure this option later",
          "Operation failed",
          wxCENTER | wxOK_DEFAULT | wxOK | wxCANCEL |
          wxICON_EXCLAMATION);

        dialog->SetOKLabel("Retry");

        int prompt_result = dialog->ShowModal(); 
        if (prompt_result == wxID_CANCEL) {
          this->navigate_to_next_page_or_close();
          break;
        }
      }
    }
  }
}

void DerivedFrame::add_path_continue_clicked( wxCommandEvent& event ) {
  if (!add_path_checkbox->IsChecked()) {
    this->navigate_to_next_page_or_close();
    return;
  } 

  if (wizard_metadata->add_to_path)
  {
    while (true) {
      int result = wizard_metadata->add_to_path();
      if (result == 1)
      {
        this->navigate_to_next_page_or_close();
        return;
      }
      else 
      {
        wxMessageDialog* dialog = new wxMessageDialog(this,
          "An error occurred while registering the 'espanso' command to the PATH, please check the logs for more information.\nDo you want to retry? You can always add espanso to the PATH later",
          "Operation failed",
          wxCENTER | wxOK_DEFAULT | wxOK | wxCANCEL |
          wxICON_EXCLAMATION);

        dialog->SetOKLabel("Retry");

        int prompt_result = dialog->ShowModal(); 
        if (prompt_result == wxID_CANCEL) {
          this->navigate_to_next_page_or_close();
          break;
        }
      }
    }
  }
}


void DerivedFrame::accessibility_enable_clicked( wxCommandEvent& event )
{
  if (wizard_metadata->enable_accessibility)
  {
    wizard_metadata->enable_accessibility();
  }
}

void DerivedFrame::quit_espanso_clicked( wxCommandEvent& event )
{
  Close(true);
}

void DerivedFrame::check_timer_tick(wxTimerEvent &event)
{
  if (this->m_simplebook->GetSelection() == LEGACY_VERSION_PAGE_INDEX)
  {
    if (wizard_metadata->is_legacy_version_running)
    {
      if (wizard_metadata->is_legacy_version_running() == 0)
      {
        this->navigate_to_next_page_or_close();
      }
    }
  } else if (this->m_simplebook->GetSelection() == ACCESSIBILITY_PAGE_INDEX) {
    if (wizard_metadata->is_accessibility_enabled)
    {
      if (wizard_metadata->is_accessibility_enabled() == 1)
      {
        this->navigate_to_next_page_or_close();
      }
    }
  }
}

void DerivedFrame::on_page_changed(wxBookCtrlEvent &event)
{
  int current_page = this->m_simplebook->GetSelection();
  this->change_default_button(current_page);
}

void DerivedFrame::change_default_button(int target_page)
{
  switch (target_page)
  {
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

  if (wizard_metadata->window_icon_path)
  {
    setFrameIcon(wizard_metadata->window_icon_path, frame);
  }

  frame->Show(true);

  Activate(frame);

  return true;
}

extern "C" int interop_show_wizard(WizardMetadata *_metadata)
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  wizard_metadata = _metadata;

  wxApp::SetInstance(new WizardApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);

  return completed_successfully;
}