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
#include "./textview_gui.h"

#include <wx/clipbrd.h>
#include <vector>
#include <memory>
#include <unordered_map>

TextViewMetadata *text_view_metadata = nullptr;

// App Code

class TextViewApp : public wxApp
{
public:
  virtual bool OnInit();
};

class DerivedTextViewFrame : public TextViewFrame
{
protected:
  void on_copy_to_clipboard( wxCommandEvent& event );

public:
  DerivedTextViewFrame(wxWindow *parent);
};

DerivedTextViewFrame::DerivedTextViewFrame(wxWindow *parent)
    : TextViewFrame(parent)
{
  this->text_content->SetValue(wxString::FromUTF8(text_view_metadata->content));
  this->SetTitle(wxString::FromUTF8(text_view_metadata->title));
}

void DerivedTextViewFrame::on_copy_to_clipboard( wxCommandEvent& event ) {
  if (wxTheClipboard->Open())
  {
    wxTheClipboard->SetData( new wxTextDataObject(wxString::FromUTF8(text_view_metadata->content)) );
    wxTheClipboard->Close();
  }
}

bool TextViewApp::OnInit()
{
  DerivedTextViewFrame *frame = new DerivedTextViewFrame(NULL);

  if (text_view_metadata->window_icon_path)
  {
    setFrameIcon(wxString::FromUTF8(text_view_metadata->window_icon_path), frame);
  }

  frame->Show(true);
  Activate(frame);

  return true;
}

extern "C" void interop_show_text_view(TextViewMetadata *_metadata)
{
// Setup high DPI support on Windows
#ifdef __WXMSW__
  SetProcessDPIAware();
#endif

  text_view_metadata = _metadata;

  wxApp::SetInstance(new TextViewApp());
  int argc = 0;
  wxEntry(argc, (char **)nullptr);
}