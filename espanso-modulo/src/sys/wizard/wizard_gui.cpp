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

	check_timer.SetOwner( this, wxID_ANY );
	check_timer.Start( 500 );

	wxBoxSizer* bSizer1;
	bSizer1 = new wxBoxSizer( wxVERTICAL );

	m_simplebook = new wxSimplebook( this, wxID_ANY, wxDefaultPosition, wxDefaultSize, 0 );
	welcome_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	welcome_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer2;
	bSizer2 = new wxBoxSizer( wxVERTICAL );

	welcome_image = new wxStaticBitmap( welcome_panel, wxID_ANY, wxNullBitmap, wxDefaultPosition, wxSize( 256,256 ), 0 );
	welcome_image->SetMinSize( wxSize( 256,256 ) );

	bSizer2->Add( welcome_image, 0, wxALIGN_CENTER|wxALL, 0 );

	welcome_title_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("Welcome to Espanso!"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_title_text->Wrap( -1 );
	welcome_title_text->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer2->Add( welcome_title_text, 0, wxALIGN_CENTER_HORIZONTAL|wxTOP, 20 );

	welcome_version_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("(version 1.2.3)"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_version_text->Wrap( -1 );
	bSizer2->Add( welcome_version_text, 0, wxALIGN_CENTER_HORIZONTAL|wxALL, 5 );


	bSizer2->Add( 0, 20, 0, 0, 5 );

	welcome_description_text = new wxStaticText( welcome_panel, wxID_ANY, wxT("This wizard will help you to quickly get started with espanso. \n\nClick \"Start\" when you are ready"), wxDefaultPosition, wxDefaultSize, 0 );
	welcome_description_text->Wrap( -1 );
	bSizer2->Add( welcome_description_text, 0, wxALL, 10 );


	bSizer2->Add( 0, 0, 1, wxEXPAND, 5 );

	welcome_start_button = new wxButton( welcome_panel, wxID_ANY, wxT("Start"), wxDefaultPosition, wxDefaultSize, 0 );

	welcome_start_button->SetDefault();
	bSizer2->Add( welcome_start_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	welcome_panel->SetSizer( bSizer2 );
	welcome_panel->Layout();
	bSizer2->Fit( welcome_panel );
	m_simplebook->AddPage( welcome_panel, wxT("a page"), false );
	move_bundle_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	move_bundle_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer22;
	bSizer22 = new wxBoxSizer( wxVERTICAL );

	move_bundle_title = new wxStaticText( move_bundle_panel, wxID_ANY, wxT("Move to /Applications folder"), wxDefaultPosition, wxDefaultSize, 0 );
	move_bundle_title->Wrap( -1 );
	move_bundle_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer22->Add( move_bundle_title, 0, wxALIGN_CENTER_HORIZONTAL|wxTOP, 20 );


	bSizer22->Add( 0, 20, 0, 0, 5 );

	move_bundle_description = new wxStaticText( move_bundle_panel, wxID_ANY, wxT("Espanso is being run from outside the Applications directory, which prevents it from working correctly.\n\nPlease move the Espanso.app bundle inside your Applications folder and start it again.\n"), wxDefaultPosition, wxDefaultSize, 0 );
	move_bundle_description->Wrap( -1 );
	bSizer22->Add( move_bundle_description, 0, wxALL, 10 );


	bSizer22->Add( 0, 20, 1, wxEXPAND, 5 );

	move_bundle_quit_button = new wxButton( move_bundle_panel, wxID_ANY, wxT("Quit"), wxDefaultPosition, wxDefaultSize, 0 );

	move_bundle_quit_button->SetDefault();
	bSizer22->Add( move_bundle_quit_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	move_bundle_panel->SetSizer( bSizer22 );
	move_bundle_panel->Layout();
	bSizer22->Fit( move_bundle_panel );
	m_simplebook->AddPage( move_bundle_panel, wxT("a page"), false );
	legacy_version_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	legacy_version_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer21;
	bSizer21 = new wxBoxSizer( wxVERTICAL );

	legacy_version_title = new wxStaticText( legacy_version_panel, wxID_ANY, wxT("Legacy version detected"), wxDefaultPosition, wxDefaultSize, 0 );
	legacy_version_title->Wrap( -1 );
	legacy_version_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer21->Add( legacy_version_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer21->Add( 0, 20, 0, 0, 5 );

	legacy_version_description = new wxStaticText( legacy_version_panel, wxID_ANY, wxT("A legacy espanso process has been detected and prevents the new version from working correctly.\n\nPlease terminate and uninstall the old espanso version to proceed.\n\nIf you already uninstalled the previous version, you might need to restart your computer for changes to be detected.\n\nFor more information, see: "), wxDefaultPosition, wxDefaultSize, 0 );
	legacy_version_description->Wrap( 500 );
	bSizer21->Add( legacy_version_description, 0, wxLEFT|wxRIGHT|wxTOP, 10 );

	legacy_version_docs_link = new wxHyperlinkCtrl( legacy_version_panel, wxID_ANY, wxT("https://espanso.org/legacy/uninstall"), wxT("https://espanso.org/legacy/uninstall"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer21->Add( legacy_version_docs_link, 0, wxLEFT|wxRIGHT, 10 );


	bSizer21->Add( 0, 0, 1, wxEXPAND, 5 );

	legacy_version_continue_button = new wxButton( legacy_version_panel, wxID_ANY, wxT("Continue"), wxDefaultPosition, wxDefaultSize, 0 );

	legacy_version_continue_button->SetDefault();
	legacy_version_continue_button->Enable( false );

	bSizer21->Add( legacy_version_continue_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	legacy_version_panel->SetSizer( bSizer21 );
	legacy_version_panel->Layout();
	bSizer21->Fit( legacy_version_panel );
	m_simplebook->AddPage( legacy_version_panel, wxT("a page"), false );
	wrong_edition_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	wrong_edition_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer213;
	bSizer213 = new wxBoxSizer( wxVERTICAL );

	wrong_edition_title = new wxStaticText( wrong_edition_panel, wxID_ANY, wxT("Incompatibility detected"), wxDefaultPosition, wxDefaultSize, 0 );
	wrong_edition_title->Wrap( -1 );
	wrong_edition_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer213->Add( wrong_edition_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer213->Add( 0, 20, 0, 0, 5 );

	wrong_edition_description_x11 = new wxStaticText( wrong_edition_panel, wxID_ANY, wxT("This version of espanso was compiled to support X11-based systems, but it seems you are on a Wayland-based desktop environment.\n\nUnfortunately, the two versions are incompatible. To use espanso, either switch to an X11-based environment or download the Wayland version from the website.\n\nFor more information:"), wxDefaultPosition, wxDefaultSize, 0 );
	wrong_edition_description_x11->Wrap( 500 );
	bSizer213->Add( wrong_edition_description_x11, 0, wxEXPAND|wxLEFT|wxRIGHT|wxTOP, 10 );

	wrong_edition_description_wayland = new wxStaticText( wrong_edition_panel, wxID_ANY, wxT("This version of espanso was compiled to support Wayland-based systems, but it seems you are on a X11-based desktop environment.\n\nUnfortunately, the two versions are incompatible. To use espanso, either switch to a Wayland-based environment or download the X11 version from the website.\n\nFor more information:"), wxDefaultPosition, wxDefaultSize, 0 );
	wrong_edition_description_wayland->Wrap( 500 );
	bSizer213->Add( wrong_edition_description_wayland, 0, wxEXPAND|wxLEFT|wxTOP, 10 );

	wrong_edition_link = new wxHyperlinkCtrl( wrong_edition_panel, wxID_ANY, wxT("https://espanso.org/install"), wxT("https://espanso.org/install"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer213->Add( wrong_edition_link, 0, wxLEFT|wxRIGHT, 10 );


	bSizer213->Add( 0, 0, 1, wxEXPAND, 5 );

	wrong_edition_button = new wxButton( wrong_edition_panel, wxID_ANY, wxT("Quit Espanso"), wxDefaultPosition, wxDefaultSize, 0 );

	wrong_edition_button->SetDefault();
	bSizer213->Add( wrong_edition_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	wrong_edition_panel->SetSizer( bSizer213 );
	wrong_edition_panel->Layout();
	bSizer213->Fit( wrong_edition_panel );
	m_simplebook->AddPage( wrong_edition_panel, wxT("a page"), false );
	migrate_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	migrate_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer211;
	bSizer211 = new wxBoxSizer( wxVERTICAL );

	migrate_title = new wxStaticText( migrate_panel, wxID_ANY, wxT("Migrate configuration"), wxDefaultPosition, wxDefaultSize, 0 );
	migrate_title->Wrap( -1 );
	migrate_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer211->Add( migrate_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer211->Add( 0, 20, 0, 0, 5 );

	migrate_description = new wxStaticText( migrate_panel, wxID_ANY, wxT("The new version uses a slightly different configuration format that powers some exciting new features.\n\nTo ease the transition, espanso offers two possible choices: \n\n  - Automatically backup the old configuration in the Documents folder and migrate to the new format (recommended). \n  - Use compatibility mode without changing the configs. \n\nKeep in mind that: \n\n  - Compatibility mode does not support all new espanso features \n  - You can always migrate the configs later \n\nFor more information, see: "), wxDefaultPosition, wxDefaultSize, 0 );
	migrate_description->Wrap( 500 );
	bSizer211->Add( migrate_description, 1, wxLEFT|wxRIGHT|wxTOP, 10 );

	migrate_link = new wxHyperlinkCtrl( migrate_panel, wxID_ANY, wxT("https://espanso.org/migration"), wxT("https://espanso.org/migration"), wxDefaultPosition, wxDefaultSize, wxHL_DEFAULT_STYLE );
	bSizer211->Add( migrate_link, 0, wxLEFT|wxRIGHT, 10 );


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
	auto_start_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	auto_start_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer2122;
	bSizer2122 = new wxBoxSizer( wxVERTICAL );

	auto_start_title = new wxStaticText( auto_start_panel, wxID_ANY, wxT("Launch on System startup"), wxDefaultPosition, wxDefaultSize, 0 );
	auto_start_title->Wrap( -1 );
	auto_start_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer2122->Add( auto_start_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer2122->Add( 0, 20, 0, 0, 5 );

	auto_start_description = new wxStaticText( auto_start_panel, wxID_ANY, wxT("Espanso can be launched automatically when you start your PC. \n\nDo you want to proceed?"), wxDefaultPosition, wxDefaultSize, 0 );
	auto_start_description->Wrap( 500 );
	bSizer2122->Add( auto_start_description, 0, wxLEFT|wxRIGHT|wxTOP, 10 );

	auto_start_checkbox = new wxCheckBox( auto_start_panel, wxID_ANY, wxT("Yes, launch Espanso on system startup (recommended)"), wxDefaultPosition, wxDefaultSize, 0 );
	auto_start_checkbox->SetValue(true);
	bSizer2122->Add( auto_start_checkbox, 0, wxALL, 20 );

	auto_start_note = new wxStaticText( auto_start_panel, wxID_ANY, wxT("Note: you can always disable this option later."), wxDefaultPosition, wxDefaultSize, 0 );
	auto_start_note->Wrap( 500 );
	bSizer2122->Add( auto_start_note, 0, wxALL, 10 );


	bSizer2122->Add( 0, 0, 1, wxEXPAND, 5 );

	auto_start_continue = new wxButton( auto_start_panel, wxID_ANY, wxT("Continue"), wxDefaultPosition, wxDefaultSize, 0 );

	auto_start_continue->SetDefault();
	bSizer2122->Add( auto_start_continue, 0, wxALIGN_RIGHT|wxALL, 10 );


	auto_start_panel->SetSizer( bSizer2122 );
	auto_start_panel->Layout();
	bSizer2122->Fit( auto_start_panel );
	m_simplebook->AddPage( auto_start_panel, wxT("a page"), false );
	add_path_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	add_path_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer212;
	bSizer212 = new wxBoxSizer( wxVERTICAL );

	add_path_title = new wxStaticText( add_path_panel, wxID_ANY, wxT("Add to PATH"), wxDefaultPosition, wxDefaultSize, 0 );
	add_path_title->Wrap( -1 );
	add_path_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer212->Add( add_path_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer212->Add( 0, 20, 0, 0, 5 );

	add_path_description = new wxStaticText( add_path_panel, wxID_ANY, wxT("Espanso offers a rich CLI interface that enables some powerful features and comes handy when debugging configuration problems.\n\nTo be easily accessed, espanso can be added to the PATH environment variable automatically. Do you want to proceed?\n"), wxDefaultPosition, wxDefaultSize, 0 );
	add_path_description->Wrap( 500 );
	bSizer212->Add( add_path_description, 0, wxLEFT|wxRIGHT|wxTOP, 10 );

	add_path_checkbox = new wxCheckBox( add_path_panel, wxID_ANY, wxT("Yes, add espanso to PATH"), wxDefaultPosition, wxDefaultSize, 0 );
	add_path_checkbox->SetValue(true);
	bSizer212->Add( add_path_checkbox, 0, wxALL, 20 );

	add_path_note = new wxStaticText( add_path_panel, wxID_ANY, wxT("Note: if you don't know what the PATH env variable is, you should probably keep this checked."), wxDefaultPosition, wxDefaultSize, 0 );
	add_path_note->Wrap( 500 );
	bSizer212->Add( add_path_note, 0, wxALL, 10 );


	bSizer212->Add( 0, 0, 1, wxEXPAND, 5 );

	add_path_continue_button = new wxButton( add_path_panel, wxID_ANY, wxT("Continue"), wxDefaultPosition, wxDefaultSize, 0 );

	add_path_continue_button->SetDefault();
	bSizer212->Add( add_path_continue_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	add_path_panel->SetSizer( bSizer212 );
	add_path_panel->Layout();
	bSizer212->Fit( add_path_panel );
	m_simplebook->AddPage( add_path_panel, wxT("a page"), false );
	accessibility_panel = new wxPanel( m_simplebook, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxTAB_TRAVERSAL );
	accessibility_panel->SetBackgroundColour( wxSystemSettings::GetColour( wxSYS_COLOUR_WINDOW ) );

	wxBoxSizer* bSizer2121;
	bSizer2121 = new wxBoxSizer( wxVERTICAL );

	accessibility_title = new wxStaticText( accessibility_panel, wxID_ANY, wxT("Enable Accessibility"), wxDefaultPosition, wxDefaultSize, 0 );
	accessibility_title->Wrap( -1 );
	accessibility_title->SetFont( wxFont( 18, wxFONTFAMILY_DEFAULT, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_BOLD, false, wxEmptyString ) );

	bSizer2121->Add( accessibility_title, 0, wxALIGN_CENTER_HORIZONTAL|wxALIGN_LEFT|wxTOP, 20 );


	bSizer2121->Add( 0, 20, 0, 0, 5 );

	m_scrolledWindow1 = new wxScrolledWindow( accessibility_panel, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxVSCROLL );
	m_scrolledWindow1->SetScrollRate( 5, 5 );
	wxBoxSizer* bSizer81;
	bSizer81 = new wxBoxSizer( wxVERTICAL );

	accessibility_description = new wxStaticText( m_scrolledWindow1, wxID_ANY, wxT("Espanso needs Accessibility permissions to detect and insert snippets into applications. \n\nTo enable it, follow these steps:\n\n1. Click on \"Enable\" (at the bottom right)\n2. In the dialog that appears, click on \"Open System Preferences\"\n"), wxDefaultPosition, wxDefaultSize, 0 );
	accessibility_description->Wrap( 500 );
	bSizer81->Add( accessibility_description, 0, wxLEFT|wxRIGHT|wxTOP, 10 );

	accessibility_image1 = new wxStaticBitmap( m_scrolledWindow1, wxID_ANY, wxNullBitmap, wxDefaultPosition, wxDefaultSize, 0 );
	bSizer81->Add( accessibility_image1, 0, wxALIGN_CENTER_HORIZONTAL|wxALL, 5 );

	accessibility_description2 = new wxStaticText( m_scrolledWindow1, wxID_ANY, wxT("3. Then, under the \"Privacy\" panel click on the Lock icon (1) to enable edits and then check \"Espanso\" (2), as shown in the picture:"), wxDefaultPosition, wxDefaultSize, 0 );
	accessibility_description2->Wrap( 500 );
	bSizer81->Add( accessibility_description2, 0, wxALL, 10 );

	accessibility_image2 = new wxStaticBitmap( m_scrolledWindow1, wxID_ANY, wxNullBitmap, wxDefaultPosition, wxDefaultSize, 0 );
	bSizer81->Add( accessibility_image2, 0, wxALIGN_CENTER_HORIZONTAL|wxALL, 5 );


	m_scrolledWindow1->SetSizer( bSizer81 );
	m_scrolledWindow1->Layout();
	bSizer81->Fit( m_scrolledWindow1 );
	bSizer2121->Add( m_scrolledWindow1, 1, wxEXPAND | wxALL, 0 );

	accessibility_enable_button = new wxButton( accessibility_panel, wxID_ANY, wxT("Enable"), wxDefaultPosition, wxDefaultSize, 0 );

	accessibility_enable_button->SetDefault();
	bSizer2121->Add( accessibility_enable_button, 0, wxALIGN_RIGHT|wxALL, 10 );


	accessibility_panel->SetSizer( bSizer2121 );
	accessibility_panel->Layout();
	bSizer2121->Fit( accessibility_panel );
	m_simplebook->AddPage( accessibility_panel, wxT("a page"), false );

	bSizer1->Add( m_simplebook, 1, wxEXPAND | wxALL, 5 );


	this->SetSizer( bSizer1 );
	this->Layout();

	this->Centre( wxBOTH );

	// Connect Events
	this->Connect( wxID_ANY, wxEVT_TIMER, wxTimerEventHandler( WizardFrame::check_timer_tick ) );
	m_simplebook->Connect( wxEVT_COMMAND_BOOKCTRL_PAGE_CHANGED, wxBookCtrlEventHandler( WizardFrame::on_page_changed ), NULL, this );
	welcome_start_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::welcome_start_clicked ), NULL, this );
	move_bundle_quit_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::move_bundle_quit_clicked ), NULL, this );
	wrong_edition_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::quit_espanso_clicked ), NULL, this );
	migrate_compatibility_mode_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::migrate_compatibility_mode_clicked ), NULL, this );
	migrate_backup_and_migrate_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::migrate_button_clicked ), NULL, this );
	auto_start_continue->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::auto_start_continue_clicked ), NULL, this );
	add_path_continue_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::add_path_continue_clicked ), NULL, this );
	accessibility_enable_button->Connect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::accessibility_enable_clicked ), NULL, this );
}

WizardFrame::~WizardFrame()
{
	// Disconnect Events
	this->Disconnect( wxID_ANY, wxEVT_TIMER, wxTimerEventHandler( WizardFrame::check_timer_tick ) );
	m_simplebook->Disconnect( wxEVT_COMMAND_BOOKCTRL_PAGE_CHANGED, wxBookCtrlEventHandler( WizardFrame::on_page_changed ), NULL, this );
	welcome_start_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::welcome_start_clicked ), NULL, this );
	move_bundle_quit_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::move_bundle_quit_clicked ), NULL, this );
	wrong_edition_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::quit_espanso_clicked ), NULL, this );
	migrate_compatibility_mode_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::migrate_compatibility_mode_clicked ), NULL, this );
	migrate_backup_and_migrate_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::migrate_button_clicked ), NULL, this );
	auto_start_continue->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::auto_start_continue_clicked ), NULL, this );
	add_path_continue_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::add_path_continue_clicked ), NULL, this );
	accessibility_enable_button->Disconnect( wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler( WizardFrame::accessibility_enable_clicked ), NULL, this );

}
