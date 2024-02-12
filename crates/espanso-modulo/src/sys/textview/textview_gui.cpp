///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#define _UNICODE

#include "textview_gui.h"

///////////////////////////////////////////////////////////////////////////

TextViewFrame::TextViewFrame( wxWindow* parent, wxWindowID id, const wxString& title, const wxPoint& pos, const wxSize& size, long style ) : wxFrame( parent, id, title, pos, size, style )
{
	this->SetSizeHints( wxDefaultSize, wxDefaultSize );
	this->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer1;
	bSizer1 = new wxBoxSizer( wxVERTICAL );

	text_content = new wxTextCtrl( this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize, wxTE_MULTILINE|wxTE_READONLY );
	text_content->SetFont( wxFont( wxNORMAL_FONT->GetPointSize(), wxFONTFAMILY_TELETYPE, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL, false, wxEmptyString ) );

	bSizer1->Add( text_content, 1, wxALL|wxEXPAND, 5 );

	wxBoxSizer* bSizer2;
	bSizer2 = new wxBoxSizer( wxHORIZONTAL );


	bSizer2->Add( 0, 0, 1, wxEXPAND, 5 );

	copy_to_clipboard_btn = new wxButton( this, wxID_ANY, wxT("Copy to Clipboard"), wxDefaultPosition, wxDefaultSize, 0 );

	copy_to_clipboard_btn->SetDefault();
	bSizer2->Add( copy_to_clipboard_btn, 0, wxALIGN_CENTER_VERTICAL|wxALL, 10 );


	bSizer1->Add( bSizer2, 0, wxEXPAND, 10 );


	this->SetSizer( bSizer1 );
	this->Layout();

	this->Centre( wxBOTH );

	// Connect Events
	copy_to_clipboard_btn->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( TextViewFrame::on_copy_to_clipboard ), NULL, this );
}

TextViewFrame::~TextViewFrame()
{
	// Disconnect Events
	copy_to_clipboard_btn->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( TextViewFrame::on_copy_to_clipboard ), NULL, this );

}
