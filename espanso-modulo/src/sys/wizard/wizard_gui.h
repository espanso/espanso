///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#pragma once

#include <wx/artprov.h>
#include <wx/xrc/xmlres.h>
#include <wx/timer.h>
#include <wx/bitmap.h>
#include <wx/image.h>
#include <wx/icon.h>
#include <wx/statbmp.h>
#include <wx/gdicmn.h>
#include <wx/font.h>
#include <wx/colour.h>
#include <wx/settings.h>
#include <wx/string.h>
#include <wx/stattext.h>
#include <wx/button.h>
#include <wx/sizer.h>
#include <wx/panel.h>
#include <wx/hyperlink.h>
#include <wx/checkbox.h>
#include <wx/scrolwin.h>
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
		wxTimer check_timer;
		wxSimplebook* m_simplebook;
		wxPanel* welcome_panel;
		wxStaticBitmap* welcome_image;
		wxStaticText* welcome_title_text;
		wxStaticText* welcome_version_text;
		wxStaticText* welcome_description_text;
		wxButton* welcome_start_button;
		wxPanel* move_bundle_panel;
		wxStaticText* move_bundle_title;
		wxStaticText* move_bundle_description;
		wxButton* move_bundle_quit_button;
		wxPanel* legacy_version_panel;
		wxStaticText* legacy_version_title;
		wxStaticText* legacy_version_description;
		wxHyperlinkCtrl* legacy_version_docs_link;
		wxButton* legacy_version_continue_button;
		wxPanel* wrong_edition_panel;
		wxStaticText* wrong_edition_title;
		wxStaticText* wrong_edition_description_x11;
		wxStaticText* wrong_edition_description_wayland;
		wxHyperlinkCtrl* wrong_edition_link;
		wxButton* wrong_edition_button;
		wxPanel* migrate_panel;
		wxStaticText* migrate_title;
		wxStaticText* migrate_description;
		wxHyperlinkCtrl* migrate_link;
		wxButton* migrate_compatibility_mode_button;
		wxButton* migrate_backup_and_migrate_button;
		wxPanel* add_path_panel;
		wxStaticText* add_path_title;
		wxStaticText* add_path_description;
		wxCheckBox* add_path_checkbox;
		wxStaticText* add_path_note;
		wxButton* add_path_continue_button;
		wxPanel* accessibility_panel;
		wxStaticText* accessibility_title;
		wxScrolledWindow* m_scrolledWindow1;
		wxStaticText* accessibility_description;
		wxStaticBitmap* accessibility_image1;
		wxStaticText* accessibility_description2;
		wxStaticBitmap* accessibility_image2;
		wxButton* accessibility_enable_button;

		// Virtual event handlers, overide them in your derived class
		virtual void check_timer_tick( wxTimerEvent& event ) { event.Skip(); }
		virtual void on_page_changed( wxBookCtrlEvent& event ) { event.Skip(); }
		virtual void welcome_start_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void move_bundle_quit_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void quit_espanso_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void migrate_compatibility_mode_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void migrate_button_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void add_path_continue_clicked( wxCommandEvent& event ) { event.Skip(); }
		virtual void accessibility_enable_clicked( wxCommandEvent& event ) { event.Skip(); }


	public:

		WizardFrame( wxWindow* parent, wxWindowID id = wxID_ANY, const wxString& title = wxT("Espanso"), const wxPoint& pos = wxDefaultPosition, const wxSize& size = wxSize( 550,577 ), long style = wxCAPTION|wxCLOSE_BOX|wxSYSTEM_MENU|wxTAB_TRAVERSAL );

		~WizardFrame();

};

