///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#define _UNICODE

#include "troubleshooting_gui.h"

///////////////////////////////////////////////////////////////////////////

TroubleshootingFrame::TroubleshootingFrame( wxWindow* parent, wxWindowID id, const wxString& title, const wxPoint& pos, const wxSize& size, long style ) : wxFrame( parent, id, title, pos, size, style )
{
	this->SetSizeHints( wxDefaultSize, wxDefaultSize );
	this->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer1;
	bSizer1 = new wxBoxSizer( wxVERTICAL );


	bSizer1->Add( 0, 10, 0, wxEXPAND, 5 );

	title_label = new wxStaticText( this, wxID_ANY, wxT("Errors detected"), wxDefaultPosition, wxDefaultSize, 0 );
	title_label->Wrap( -1 );
	title_label->SetFont( wxFont( 20, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer1->Add( title_label, 0, wxALL, 10 );

	info_label = new wxStaticText( this, wxID_ANY, wxT("Espanso couldn't load some files due to configuration errors. Some snippets or settings might not be available until you fix them."), wxDefaultPosition, wxSize( -1,-1 ), 0 );
	info_label->Wrap( -1 );
	bSizer1->Add( info_label, 0, wxALL, 10 );

	scrollview = new wxScrolledWindow( this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxHSCROLL|wxVSCROLL );
	scrollview->SetScrollRate( 5, 5 );
	scrollview_sizer = new wxBoxSizer( wxVERTICAL );


	scrollview->SetSizer( scrollview_sizer );
	scrollview->Layout();
	scrollview_sizer->Fit( scrollview );
	bSizer1->Add( scrollview, 5, wxEXPAND | wxALL, 5 );

	wxBoxSizer* bSizer2;
	bSizer2 = new wxBoxSizer( wxHORIZONTAL );

	dont_show_checkbox = new wxCheckBox( this, wxID_ANY, wxT("Don't show again for non-critical errors"), wxDefaultPosition, wxDefaultSize, 0 );
	bSizer2->Add( dont_show_checkbox, 0, wxALIGN_CENTER_VERTICAL|wxALL, 10 );


	bSizer2->Add( 0, 0, 1, wxEXPAND, 5 );

	ignore_button = new wxButton( this, wxID_ANY, wxT("Ignore errors"), wxDefaultPosition, wxDefaultSize, 0 );
	bSizer2->Add( ignore_button, 0, wxALIGN_CENTER_VERTICAL|wxALL, 10 );


	bSizer1->Add( bSizer2, 0, wxEXPAND, 10 );


	this->SetSizer( bSizer1 );
	this->Layout();

	this->Centre( wxBOTH );

	// Connect Events
	dont_show_checkbox->Connect( wxEVT_COMMAND_CHECKBOX_CLICKED, wxCommandEventHandler( TroubleshootingFrame::on_dont_show_change ), NULL, this );
	ignore_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( TroubleshootingFrame::on_ignore ), NULL, this );
}

TroubleshootingFrame::~TroubleshootingFrame()
{
	// Disconnect Events
	dont_show_checkbox->Disconnect( wxEVT_COMMAND_CHECKBOX_CLICKED, wxCommandEventHandler( TroubleshootingFrame::on_dont_show_change ), NULL, this );
	ignore_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( TroubleshootingFrame::on_ignore ), NULL, this );

}
