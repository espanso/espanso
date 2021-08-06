///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#define _UNICODE

#include "welcome_gui.h"

///////////////////////////////////////////////////////////////////////////

WelcomeFrame::WelcomeFrame( wxWindow* parent, wxWindowID id, const wxString& title, const wxPoint& pos, const wxSize& size, long style ) : wxFrame( parent, id, title, pos, size, style )
{
	this->SetSizeHints( wxDefaultSize, wxDefaultSize );
	this->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer1;
	bSizer1 = new wxBoxSizer( wxVERTICAL );


	bSizer1->Add( 0, 10, 0, wxEXPAND, 5 );

	title_label = new wxStaticText( this, wxID_ANY, wxT("Espanso is running!"), wxDefaultPosition, wxDefaultSize, 0 );
	title_label->Wrap( -1 );
	title_label->SetFont( wxFont( 20, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer1->Add( title_label, 0, wxALIGN_CENTER|wxALL, 10 );

	tray_info_label = new wxStaticText( this, wxID_ANY, wxT("You should now see its icon on the tray bar:"), wxDefaultPosition, wxDefaultSize, 0 );
	tray_info_label->Wrap( -1 );
	bSizer1->Add( tray_info_label, 0, wxALIGN_CENTER|wxALL, 10 );

	tray_bitmap = new wxStaticBitmap( this, wxID_ANY, wxNullBitmap, wxDefaultPosition, wxDefaultSize, 0 );
	bSizer1->Add( tray_bitmap, 0, wxALIGN_CENTER|wxALL, 5 );


	bSizer1->Add( 0, 10, 0, 0, 10 );

	test_label = new wxStaticText( this, wxID_ANY, wxT("Try typing \":espanso\" below (without quotes)"), wxDefaultPosition, wxDefaultSize, 0 );
	test_label->Wrap( -1 );
	bSizer1->Add( test_label, 0, wxALIGN_CENTER|wxALL, 10 );

	test_text_ctrl = new wxTextCtrl( this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize, 0 );
	test_text_ctrl->SetFont( wxFont( 16, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL, false, wxEmptyString ) );

	bSizer1->Add( test_text_ctrl, 0, wxALL|wxEXPAND, 10 );

	doc_label = new wxStaticText( this, wxID_ANY, wxT("Do you want to know more? Visit the documentation:"), wxDefaultPosition, wxDefaultSize, 0 );
	doc_label->Wrap( -1 );
	bSizer1->Add( doc_label, 0, wxALIGN_CENTER|wxALL, 10 );

	m_hyperlink1 = new wxHyperlinkCtrl( this, wxID_ANY, wxT("https://espanso.org/docs/get-started/"), wxT("https://espanso.org/docs/get-started/"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer1->Add( m_hyperlink1, 0, wxALIGN_CENTER|wxALL, 10 );


	bSizer1->Add( 0, 0, 1, wxEXPAND, 5 );

	wxBoxSizer* bSizer2;
	bSizer2 = new wxBoxSizer( wxHORIZONTAL );

	dont_show_checkbox = new wxCheckBox( this, wxID_ANY, wxT("Don't show this again"), wxDefaultPosition, wxDefaultSize, 0 );
	bSizer2->Add( dont_show_checkbox, 0, wxALIGN_CENTER_VERTICAL|wxALL, 10 );


	bSizer2->Add( 0, 0, 1, wxEXPAND, 5 );

	got_it_btn = new wxButton( this, wxID_ANY, wxT("Got it!"), wxDefaultPosition, wxDefaultSize, 0 );

	got_it_btn->SetDefault();
	bSizer2->Add( got_it_btn, 0, wxALIGN_CENTER_VERTICAL|wxALL, 10 );


	bSizer1->Add( bSizer2, 0, wxEXPAND, 10 );


	this->SetSizer( bSizer1 );
	this->Layout();

	this->Centre( wxBOTH );

	// Connect Events
	dont_show_checkbox->Connect( wxEVT_COMMAND_CHECKBOX_CLICKED, wxCommandEventHandler( WelcomeFrame::on_dont_show_change ), NULL, this );
	got_it_btn->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WelcomeFrame::on_complete ), NULL, this );
}

WelcomeFrame::~WelcomeFrame()
{
	// Disconnect Events
	dont_show_checkbox->Disconnect( wxEVT_COMMAND_CHECKBOX_CLICKED, wxCommandEventHandler( WelcomeFrame::on_dont_show_change ), NULL, this );
	got_it_btn->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WelcomeFrame::on_complete ), NULL, this );

}
