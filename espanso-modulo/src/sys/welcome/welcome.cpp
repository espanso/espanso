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
#include "./welcome_gui.h"

#include <vector>
#include <memory>
#include <unordered_map>

WelcomeMetadata *welcome_metadata = nullptr;

// App Code

class WelcomeApp : public wxApp
{
public:
  virtual bool OnInit();
};

class DerivedWelcomeFrame : public WelcomeFrame
{
protected:
  void on_dont_show_change( wxCommandEvent& event );
  void on_complete( wxCommandEvent& event );

public:
  DerivedWelcomeFrame(wxWindow *parent);
};

DerivedWelcomeFrame::DerivedWelcomeFrame(wxWindow *parent)
    : WelcomeFrame(parent)
{
  // Welcome images

  if (welcome_metadata->tray_image_path)
  {
    wxBitmap trayBitmap = wxBitmap(welcome_metadata->tray_image_path, wxBITMAP_TYPE_PNG);
    this->tray_bitmap->SetBitmap(trayBitmap);
    #ifdef __WXOSX__
      this->tray_info_label->SetLabel("You should see the espanso icon on the status bar:");
    #endif
  }
  else 
  {
    this->tray_info_label->Hide();
  }
}

void DerivedWelcomeFrame::on_dont_show_change( wxCommandEvent& event ) {
  if (welcome_metadata->dont_show_again_changed) {
    int value = this->dont_show_checkbox->IsChecked() ? 1 : 0;
    welcome_metadata->dont_show_again_changed(value);
  }
}

void DerivedWelcomeFrame::on_complete( wxCommandEvent& event ) {
  Close(true);
}


bool WelcomeApp::OnInit()
{
  wxInitAllImageHandlers();
  DerivedWelcomeFrame *frame = new DerivedWelcomeFrame(NULL);

  if (welcome_metadata->window_icon_path)
  {
    setFrameIcon(welcome_metadata->window_icon_path, frame);
  }

  frame->Show(true);

  Activate(frame);

  return true;
}

extern "C" void interop_show_welcome(WelcomeMetadata *_metadata)
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  welcome_metadata = _metadata;

  wxApp::SetInstance(new WelcomeApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);
}