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

// App Code

class WizardApp : public wxApp
{
public:
  virtual bool OnInit();
};

class DerivedFrame : public WizardFrame
{
protected:
  void welcome_start_clicked(wxCommandEvent &event);

public:
  DerivedFrame(wxWindow *parent);
};

DerivedFrame::DerivedFrame(wxWindow *parent)
    : WizardFrame(parent)
{
}

void DerivedFrame::welcome_start_clicked(wxCommandEvent &event)
{
  this->m_simplebook->ChangeSelection(2);
}

bool WizardApp::OnInit()
{
  DerivedFrame *frame = new DerivedFrame(NULL);
  //setFrameIcon(formMetadata->iconPath, frame);
  frame->Show(true);

  Activate(frame);

  return true;
}

extern "C" void interop_show_wizard()
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  wxApp::SetInstance(new WizardApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);
}