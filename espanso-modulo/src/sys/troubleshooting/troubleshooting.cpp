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
#include "./troubleshooting_gui.h"

#include <vector>
#include <memory>
#include <unordered_map>

TroubleshootingMetadata *troubleshooting_metadata = nullptr;

// App Code

class TroubleshootingApp : public wxApp
{
public:
  virtual bool OnInit();
};

// Custom controller to display an ErrorSet

class ErrorSetPanel : public wxPanel
{
private:
protected:
  wxStaticText *filename_label;
  wxButton *open_file_btn;
  wxTextCtrl *error_text_ctrl;
  const ErrorSetMetadata * error_set_metadata;

public:
  ErrorSetPanel(wxWindow *parent, const ErrorSetMetadata * error_set_metadata) : wxPanel(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL)
  {
    this->error_set_metadata = error_set_metadata;

    wxBoxSizer *main_file_sizer;
    main_file_sizer = new wxBoxSizer(wxVERTICAL);
    main_file_sizer->SetMinSize(0, 150);

    wxBoxSizer *header_sizer;
    header_sizer = new wxBoxSizer(wxHORIZONTAL);

    wxString path = wxString::FromUTF8(error_set_metadata->file_path);
    wxString filename = wxString::Format(wxT("%s (%i errors)"), path, error_set_metadata->errors_count);
    filename_label = new wxStaticText(this, wxID_ANY, filename, wxDefaultPosition, wxDefaultSize, 0);
    filename_label->Wrap(-1);
    filename_label->SetFont(wxFont(wxNORMAL_FONT->GetPointSize(), wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString));

    header_sizer->Add(filename_label, 0, wxALIGN_CENTER_VERTICAL | wxALL, 5);

    header_sizer->Add(0, 0, 1, wxEXPAND, 5);

    open_file_btn = new wxButton(this, wxID_ANY, wxT("Open file"), wxDefaultPosition, wxDefaultSize, 0);
    header_sizer->Add(open_file_btn, 0, wxALIGN_CENTER_VERTICAL | wxALL, 5);

    main_file_sizer->Add(header_sizer, 0, wxEXPAND, 5);

    wxString errors_text = wxEmptyString;
    for (int i = 0; i<error_set_metadata->errors_count; i++) {
      wxString level = wxT("ERROR");
      if (error_set_metadata->errors[i].level == ERROR_METADATA_LEVEL_WARNING) {
        level = wxT("WARNING");
      }
      wxString error_text = wxString::Format(wxT("[%s] %s\n"), level, error_set_metadata->errors[i].message);
      errors_text.Append(error_text);
    }

    error_text_ctrl = new wxTextCtrl(this, wxID_ANY, errors_text, wxDefaultPosition, wxDefaultSize, wxTE_MULTILINE | wxTE_READONLY);

    main_file_sizer->Add(error_text_ctrl, 1, wxALL | wxEXPAND, 5);

    this->SetSizer(main_file_sizer);
    this->Layout();
    main_file_sizer->Fit(this);

    if (!this->error_set_metadata->file_path) {
      filename_label->Hide();
      open_file_btn->Hide();
    }

    open_file_btn->Connect(wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler(ErrorSetPanel::on_open_file), NULL, this);
  }

  void ErrorSetPanel::on_open_file(wxCommandEvent &event)
  {
    if (troubleshooting_metadata->open_file && this->error_set_metadata->file_path) {
      troubleshooting_metadata->open_file(this->error_set_metadata->file_path);
    }
  }

  ~ErrorSetPanel()
  {
    open_file_btn->Disconnect(wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler(ErrorSetPanel::on_open_file), NULL, this);
  }
};

// Frame

class DerivedTroubleshootingFrame : public TroubleshootingFrame
{
protected:
  void on_dont_show_change(wxCommandEvent &event);
  void on_ignore(wxCommandEvent &event);

public:
  DerivedTroubleshootingFrame(wxWindow *parent);
};

DerivedTroubleshootingFrame::DerivedTroubleshootingFrame(wxWindow *parent)
    : TroubleshootingFrame(parent)
{
  if (troubleshooting_metadata->is_fatal_error) {
    dont_show_checkbox->Hide();
    ignore_button->Hide();
    info_label->SetLabel(wxT("Espanso couldn't load some files due to configuration errors and won't be able to start until you fix them."));
    title_label->SetLabel(wxT("Errors detected, action needed"));
  }

  for (int i = 0; i<troubleshooting_metadata->error_sets_count; i++) {
    const ErrorSetMetadata * metadata = &troubleshooting_metadata->error_sets[i];
    ErrorSetPanel *panel = new ErrorSetPanel(scrollview, metadata);
    this->scrollview_sizer->Add(panel, 0, wxEXPAND | wxALL, 5);
  }
}

void DerivedTroubleshootingFrame::on_dont_show_change(wxCommandEvent &event)
{
  if (troubleshooting_metadata->dont_show_again_changed)
  {
    int value = this->dont_show_checkbox->IsChecked() ? 1 : 0;
    troubleshooting_metadata->dont_show_again_changed(value);
  }
}

void DerivedTroubleshootingFrame::on_ignore(wxCommandEvent &event)
{
  Close(true);
}

bool TroubleshootingApp::OnInit()
{
  DerivedTroubleshootingFrame *frame = new DerivedTroubleshootingFrame(NULL);

  if (troubleshooting_metadata->window_icon_path)
  {
    setFrameIcon(troubleshooting_metadata->window_icon_path, frame);
  }

  frame->Show(true);

  Activate(frame);

  return true;
}

extern "C" void interop_show_troubleshooting(TroubleshootingMetadata *_metadata)
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  troubleshooting_metadata = _metadata;

  wxApp::SetInstance(new TroubleshootingApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);
}