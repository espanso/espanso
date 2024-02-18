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
#include <wx/textctrl.h>
#include <wx/gdicmn.h>
#include <wx/font.h>
#include <wx/colour.h>
#include <wx/settings.h>
#include <wx/bitmap.h>
#include <wx/image.h>
#include <wx/icon.h>
#include <wx/button.h>
#include <wx/sizer.h>
#include <wx/frame.h>

///////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////
/// Class TextViewFrame
///////////////////////////////////////////////////////////////////////////////
class TextViewFrame : public wxFrame
{
	private:

	protected:
		wxTextCtrl* text_content;
		wxButton* copy_to_clipboard_btn;

		// Virtual event handlers, overide them in your derived class
		virtual void on_copy_to_clipboard( wxCommandEvent& event ) { event.Skip(); }


	public:

		TextViewFrame( wxWindow* parent, wxWindowID id = wxID_ANY, const wxString& title = wxT("TextView"), const wxPoint& pos = wxDefaultPosition, const wxSize& size = wxSize( 895,545 ), long style = wxDEFAULT_FRAME_STYLE|wxTAB_TRAVERSAL );

		~TextViewFrame();

};

