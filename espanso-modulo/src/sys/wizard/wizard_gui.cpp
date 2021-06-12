///////////////////////////////////////////////////////////////////////////
// C++ code generated with wxFormBuilder (version Oct 26 2018)
// http://www.wxformbuilder.org/
//
// PLEASE DO *NOT* EDIT THIS FILE!
///////////////////////////////////////////////////////////////////////////

#define _UNICODE

#include "wizard_gui.h"

///////////////////////////////////////////////////////////////////////////

WizardFrame::WizardFrame( wxWindow* parent, wxWindowID id, const wxString& title, const wxPoint& pos, const wxSize& size, long style ) : wxFrame( parent, id, title, pos, size, style )
{
	this->SetSizeHints( wxDefaultSize, wxDefaultSize );
	this->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer1;
	bSizer1 = new wxBoxSizer( wxVERTICAL );

	m_simplebook = new wxSimplebook( this, wxID_ANY, wxDefaultPosition, wxDefaultSize, 0 );
	welcome_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	welcome_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer2;
	bSizer2 = new wxBoxSizer( wxVERTICAL );

	welcome_title_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("Welcome to Espanso!"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_title_text->Wrap( -1 );
	welcome_title_text->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer2->Add( welcome_title_text, 0, wxALIGN_CENTER_HORIZONTAL|wxTOP, 20 );

	welcome_version_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("(version 1.2.3)"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_version_text->Wrap( -1 );
	bSizer2->Add( welcome_version_text, 0, wxALIGN_CENTER_HORIZONTAL|wxALL, 5 );

	welcome_description_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("This wizard will help you to quickly get started with espanso. \n\nClick \"Start\" when you are ready"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_description_text->Wrap( -1 );
	bSizer2->Add( welcome_description_text, 0, wxALL, 20 );


	bSizer2->Add( 0, 0, 1, wxEXPAND, 5 );

	welcome_start_button = new wxButton( welcome_panel, wxID_ANY, wxT("Start"), wxDefaultPosition, wxDefaultSize, 0 );

	welcome_start_button->SetDefault();
	bSizer2->Add( welcome_start_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	welcome_panel->SetSizer( bSizer2 );
	welcome_panel->Layout();
	bSizer2->Fit( welcome_panel );
	m_simplebook->AddPage( welcome_panel, wxT("a page"), false );
	legacy_version_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	legacy_version_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer21;
	bSizer21 = new wxBoxSizer( wxVERTICAL );

	legacy_version_title = new wxStaticText( legacy_version_panel, wxID_ANY, wxT("Legacy version detected"), wxDefaultPosition, wxDefaultSize, 0 );
	legacy_version_title->Wrap( -1 );
	legacy_version_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer21->Add( legacy_version_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );

	legacy_version_description = new wxStaticText( legacy_version_panel, wxID_ANY, wxT("A legacy espanso process has been detected and prevents the new version from working correctly.\n\nPlease terminate and uninstall the old espanso version to proceed.\n\nFor more information, see: \n"), wxDefaultPosition, wxDefaultSize, 0 );
	legacy_version_description->Wrap( 500 );
	bSizer21->Add( legacy_version_description, 0, wxLEFT|wxRIGHT|wxTOP, 20 );

	legacy_version_docs_link = new wxHyperlinkCtrl( legacy_version_panel, wxID_ANY, wxT("https://espanso.org/migration#uninstall"), wxT("https://espanso.org/migration#uninstall"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer21->Add( legacy_version_docs_link, 0, wxLEFT|wxRIGHT, 20 );


	bSizer21->Add( 0, 0, 1, wxEXPAND, 5 );

	legacy_version_continue_button = new wxButton( legacy_version_panel, wxID_ANY, wxT("Continue"), wxDefaultPosition, wxDefaultSize, 0 );

	legacy_version_continue_button->SetDefault();
	legacy_version_continue_button->Enable( false );

	bSizer21->Add( legacy_version_continue_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	legacy_version_panel->SetSizer( bSizer21 );
	legacy_version_panel->Layout();
	bSizer21->Fit( legacy_version_panel );
	m_simplebook->AddPage( legacy_version_panel, wxT("a page"), false );
	migrate_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	migrate_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer211;
	bSizer211 = new wxBoxSizer( wxVERTICAL );

	migrate_title = new wxStaticText( migrate_panel, wxID_ANY, wxT("Migrate configuration"), wxDefaultPosition, wxDefaultSize, 0 );
	migrate_title->Wrap( -1 );
	migrate_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer211->Add( migrate_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );

	migrate_description = new wxStaticText( migrate_panel, wxID_ANY, wxT("The new version uses a slightly different configuration format that powers some exciting features.\n\nTo ease the transition, espanso offers two possible choices: \n\n  - Automatically backup the old configuration in the Documents folder and migrate to the new format (recommended). \n  - Use compatibility mode without changing the configs. \n\nKeep in mind that: \n\n  - Compatibility mode does not support all new espanso features \n  - You can always migrate the configs later \n\nFor more information, see: \n"), wxDefaultPosition, wxDefaultSize, 0 );
	migrate_description->Wrap( 500 );
	bSizer211->Add( migrate_description, 1, wxLEFT|wxRIGHT|wxTOP, 20 );

	migrate_link = new wxHyperlinkCtrl( migrate_panel, wxID_ANY, wxT("https://espanso.org/migration"), wxT("https://espanso.org/migration"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer211->Add( migrate_link, 0, wxLEFT|wxRIGHT, 20 );


	bSizer211->Add( 0, 0, 10, wxEXPAND, 5 );

	wxBoxSizer* bSizer8;
	bSizer8 = new wxBoxSizer( wxHORIZONTAL );

	migrate_compatibility_mode_button = new wxButton( migrate_panel, wxID_ANY, wxT("Use compatibility mode"), wxDefaultPosition, wxDefaultSize, 0 );
	bSizer8->Add( migrate_compatibility_mode_button, 0, wxALL, 10 );


	bSizer8->Add( 0, 0, 1, wxEXPAND, 5 );

	migrate_backup_and_migrate_button = new wxButton( migrate_panel, wxID_ANY, wxT("Backup && Migrate"), wxDefaultPosition, wxDefaultSize, 0 );

	migrate_backup_and_migrate_button->SetDefault();
	bSizer8->Add( migrate_backup_and_migrate_button, 0, wxALL, 10 );


	bSizer211->Add( bSizer8, 1, wxEXPAND, 5 );


	migrate_panel->SetSizer( bSizer211 );
	migrate_panel->Layout();
	bSizer211->Fit( migrate_panel );
	m_simplebook->AddPage( migrate_panel, wxT("a page"), false );

	bSizer1->Add( m_simplebook, 1, wxEXPAND | wxALL, 5 );


	this->SetSizer( bSizer1 );
	this->Layout();

	this->Centre( wxBOTH );

	// Connect Events
	welcome_start_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::welcome_start_clicked ), NULL, this );
}

WizardFrame::~WizardFrame()
{
	// Disconnect Events
	welcome_start_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::welcome_start_clicked ), NULL, this );

}
