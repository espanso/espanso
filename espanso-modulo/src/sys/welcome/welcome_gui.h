///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#pragma once

#include <wx/artprov.h>
#include <wx/xrc/xmlres.h>
#include <wx/string.h>
#include <wx/stattext.h>
#include <wx/gdicmn.h>
#include <wx/font.h>
#include <wx/colour.h>
#include <wx/settings.h>
#include <wx/bitmap.h>
#include <wx/image.h>
#include <wx/icon.h>
#include <wx/statbmp.h>
#include <wx/textctrl.h>
#include <wx/hyperlink.h>
#include <wx/checkbox.h>
#include <wx/button.h>
#include <wx/sizer.h>
#include <wx/frame.h>

///////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////
/// Class WelcomeFrame
///////////////////////////////////////////////////////////////////////////////
class WelcomeFrame : public wxFrame
{
	private:

	protected:
		wxStaticText* title_label;
		wxStaticText* tray_info_label;
		wxStaticBitmap* tray_bitmap;
		wxStaticText* test_label;
		wxTextCtrl* test_text_ctrl;
		wxStaticText* doc_label;
		wxHyperlinkCtrl* m_hyperlink1;
		wxCheckBox* dont_show_checkbox;
		wxButton* got_it_btn;

		// Virtual event handlers, overide them in your derived class
		virtual void on_dont_show_change( wxCommandEvent& event ) { event.Skip(); }
		virtual void on_complete( wxCommandEvent& event ) { event.Skip(); }


	public:

		WelcomeFrame( wxWindow* parent, wxWindowID id = wxID_ANY, const wxString& title = wxT("Espanso is running!"), const wxPoint& pos = wxDefaultPosition, const wxSize& size = wxSize( 521,597 ), long style = wxCAPTION|wxCLOSE_BOX|wxSYSTEM_MENU|wxTAB_TRAVERSAL );

		~WelcomeFrame();

};

