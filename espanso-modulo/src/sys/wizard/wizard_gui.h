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
#include <wx/button.h>
#include <wx/sizer.h>
#include <wx/panel.h>
#include <wx/hyperlink.h>
#include <wx/simplebook.h>
#include <wx/frame.h>

///////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////
/// Class WizardFrame
///////////////////////////////////////////////////////////////////////////////
class WizardFrame : public wxFrame
{
	private:

	protected:
		wxSimplebook* m_simplebook;
		wxPanel* welcome_panel;
		wxStaticText* welcome_title_text;
		wxStaticText* welcome_version_text;
		wxStaticText* welcome_description_text;
		wxButton* welcome_start_button;
		wxPanel* legacy_version_panel;
		wxStaticText* legacy_version_title;
		wxStaticText* legacy_version_description;
		wxHyperlinkCtrl* legacy_version_docs_link;
		wxButton* legacy_version_continue_button;
		wxPanel* migrate_panel;
		wxStaticText* migrate_title;
		wxStaticText* migrate_description;
		wxHyperlinkCtrl* migrate_link;
		wxButton* migrate_compatibility_mode_button;
		wxButton* migrate_backup_and_migrate_button;

		// Virtual event handlers, overide them in your derived class
		virtual void welcome_start_clicked( wxCommandEvent& event ) { event.Skip(); }


	public:

		WizardFrame( wxWindow* parent, wxWindowID id = wxID_ANY, const wxString& title = wxT("Espanso"), const wxPoint& pos = wxDefaultPosition, const wxSize& size = wxSize( 550,523 ), long style = wxCAPTION|wxDEFAULT_FRAME_STYLE|wxTAB_TRAVERSAL );

		~WizardFrame();

};

