///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#pragma once

#include <wx/artprov.h>
#include <wx/bitmap.h>
#include <wx/button.h>
#include <wx/checkbox.h>
#include <wx/colour.h>
#include <wx/font.h>
#include <wx/frame.h>
#include <wx/gdicmn.h>
#include <wx/icon.h>
#include <wx/image.h>
#include <wx/scrolwin.h>
#include <wx/settings.h>
#include <wx/sizer.h>
#include <wx/statline.h>
#include <wx/stattext.h>
#include <wx/string.h>
#include <wx/xrc/xmlres.h>

///////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
/// Class TroubleshootingFrame
///////////////////////////////////////////////////////////////////////////////
class TroubleshootingFrame : public wxFrame {
  private:
  protected:
    wxStaticText *title_label;
    wxStaticText *info_label;
    wxStaticLine *m_staticline1;
    wxScrolledWindow *scrollview;
    wxBoxSizer *scrollview_sizer;
    wxCheckBox *dont_show_checkbox;
    wxButton *ignore_button;

    // Virtual event handlers, overide them in your derived class
    virtual void on_dont_show_change(wxCommandEvent &event) { event.Skip(); }
    virtual void on_ignore(wxCommandEvent &event) { event.Skip(); }

  public:
    TroubleshootingFrame(wxWindow *parent, wxWindowID id = wxID_ANY,
                         const wxString &title = wxT("Troubleshooting"),
                         const wxPoint &pos = wxDefaultPosition,
                         const wxSize &size = wxSize(841, 544),
                         long style = wxDEFAULT_FRAME_STYLE | wxTAB_TRAVERSAL);

    ~TroubleshootingFrame();
};
