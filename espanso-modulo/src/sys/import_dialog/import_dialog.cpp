/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

#include "../common/common.h"
#include "../interop/interop.h"
#include <wx/clipbrd.h>
#include <wx/statline.h>
#include <wx/filedlg.h>
#include <fstream>
#include <sstream>

#ifdef __WXMSW__
#include <windows.h>
#endif

// Window style - stay on top so dialog doesn't get lost behind other windows
const long IMPORT_DIALOG_STYLE = wxDEFAULT_FRAME_STYLE | wxSTAY_ON_TOP;

// Layout constants
const int IMPORT_CHECKBOX_WIDTH = 120;
const int STATUS_LABEL_WIDTH = 140;
const int CLEAR_CHECKBOX_WIDTH = 200;

// Forward declarations
class ImportFrame;

// Metadata pointer for callback
const ImportDialogMetadata *g_importMetadata = nullptr;

// Application class
class ImportDialogApp : public wxApp {
public:
    virtual bool OnInit() wxOVERRIDE;
};

// Frame class
class ImportFrame : public wxFrame {
public:
    ImportFrame(const wxString &title, const wxPoint &pos, const wxSize &size);

private:
    // Event handlers
    void OnPasteButton(wxCommandEvent &event);
    void OnImportFromFileButton(wxCommandEvent &event);
    void OnCheckButton(wxCommandEvent &event);
    void OnImportButton(wxCommandEvent &event);
    void OnCloseButton(wxCommandEvent &event);
    void OnClose(wxCloseEvent &event);
    void OnTextLostFocus(wxFocusEvent &event);
    void OnImportCheckboxChanged(wxCommandEvent &event);

    // Helper methods
    void ValidateAndUpdateUI();
    void UpdateRowUI(wxCheckBox *importCheckbox, wxStaticText *statusLabel,
                     wxCheckBox *clearCheckbox, int status, int prevStatus);
    void UpdateImportButtonState();

    // UI controls
    wxPanel *m_panel;
    wxTextCtrl *m_inputText;
    wxButton *m_pasteButton;
    wxButton *m_importFromFileButton;
    wxButton *m_checkButton;

    // Row controls: import checkbox, status label, clear checkbox
    wxCheckBox *m_importConfigCheckbox;
    wxStaticText *m_configStatusLabel;
    wxCheckBox *m_clearConfigCheckbox;

    wxCheckBox *m_importMatchesCheckbox;
    wxStaticText *m_matchesStatusLabel;
    wxCheckBox *m_clearMatchesCheckbox;

    wxCheckBox *m_importPackagesCheckbox;
    wxStaticText *m_packagesStatusLabel;
    wxCheckBox *m_clearPackagesCheckbox;

    wxButton *m_importButton;
    wxButton *m_closeButton;

    // Validation state
    bool m_isValid;
    int m_configStatusValue;
    int m_matchesStatusValue;
    int m_packagesStatusValue;

    // Previous status values (to detect transitions)
    int m_prevConfigStatus;
    int m_prevMatchesStatus;
    int m_prevPackagesStatus;
};

// Application implementation
bool ImportDialogApp::OnInit() {
    if (!wxApp::OnInit()) {
        return false;
    }

    ImportFrame *frame = new ImportFrame(
        "Import Espanso Configuration",
        wxDefaultPosition,
        wxSize(580, 480)
    );

    if (g_importMetadata && g_importMetadata->window_icon_path) {
        setFrameIcon(wxString::FromUTF8(g_importMetadata->window_icon_path), frame);
    }

    frame->Show(true);
    Activate(frame);

    return true;
}

// Frame implementation
ImportFrame::ImportFrame(const wxString &title, const wxPoint &pos, const wxSize &size)
    : wxFrame(NULL, wxID_ANY, title, pos, size, IMPORT_DIALOG_STYLE),
      m_isValid(false),
      m_configStatusValue(SCOPE_STATUS_EMPTY),
      m_matchesStatusValue(SCOPE_STATUS_EMPTY),
      m_packagesStatusValue(SCOPE_STATUS_EMPTY),
      m_prevConfigStatus(SCOPE_STATUS_EMPTY),
      m_prevMatchesStatus(SCOPE_STATUS_EMPTY),
      m_prevPackagesStatus(SCOPE_STATUS_EMPTY)
{
    m_panel = new wxPanel(this, wxID_ANY);

    // Create main vertical sizer
    wxBoxSizer *mainSizer = new wxBoxSizer(wxVERTICAL);

    // Instructions text
    wxStaticText *instructionsText = new wxStaticText(m_panel, wxID_ANY,
        "Paste the exported configuration code below:");
    mainSizer->Add(instructionsText, 0, wxALL, 10);

    // Input text control (editable, multiline)
    m_inputText = new wxTextCtrl(m_panel, wxID_ANY, wxEmptyString,
        wxDefaultPosition, wxSize(550, 180),
        wxTE_MULTILINE);

    wxFont monoFont(12, wxFONTFAMILY_TELETYPE, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL);
    m_inputText->SetFont(monoFont);
    m_inputText->SetHint("Paste export code here...");

    mainSizer->Add(m_inputText, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Button row: Paste, Import from file, and Check buttons
    wxBoxSizer *pasteRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_pasteButton = new wxButton(m_panel, wxID_ANY, "Paste from Clipboard");
    m_pasteButton->Bind(wxEVT_BUTTON, &ImportFrame::OnPasteButton, this);
    pasteRowSizer->Add(m_pasteButton, 0, wxRIGHT, 10);

    m_importFromFileButton = new wxButton(m_panel, wxID_ANY, "Import from file");
    m_importFromFileButton->Bind(wxEVT_BUTTON, &ImportFrame::OnImportFromFileButton, this);
    pasteRowSizer->Add(m_importFromFileButton, 0, wxRIGHT, 10);

    m_checkButton = new wxButton(m_panel, wxID_ANY, "Check import code");
    m_checkButton->Bind(wxEVT_BUTTON, &ImportFrame::OnCheckButton, this);
    pasteRowSizer->Add(m_checkButton, 0);

    mainSizer->Add(pasteRowSizer, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Separator
    mainSizer->Add(new wxStaticLine(m_panel), 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Header row
    wxBoxSizer *headerSizer = new wxBoxSizer(wxHORIZONTAL);
    wxStaticText *importHeader = new wxStaticText(m_panel, wxID_ANY, "Import");
    wxFont boldFont = importHeader->GetFont();
    boldFont.SetWeight(wxFONTWEIGHT_BOLD);
    importHeader->SetFont(boldFont);
    importHeader->SetMinSize(wxSize(IMPORT_CHECKBOX_WIDTH, -1));
    headerSizer->Add(importHeader, 0, wxALIGN_CENTER_VERTICAL);

    wxStaticText *statusHeader = new wxStaticText(m_panel, wxID_ANY, "Status");
    statusHeader->SetFont(boldFont);
    statusHeader->SetMinSize(wxSize(STATUS_LABEL_WIDTH, -1));
    headerSizer->Add(statusHeader, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    wxStaticText *clearHeader = new wxStaticText(m_panel, wxID_ANY, "Clear before import");
    clearHeader->SetFont(boldFont);
    headerSizer->Add(clearHeader, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    mainSizer->Add(headerSizer, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Config row
    wxBoxSizer *configRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_importConfigCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Config");
    m_importConfigCheckbox->SetValue(true);
    m_importConfigCheckbox->SetMinSize(wxSize(IMPORT_CHECKBOX_WIDTH, -1));
    m_importConfigCheckbox->Bind(wxEVT_CHECKBOX, &ImportFrame::OnImportCheckboxChanged, this);
    configRowSizer->Add(m_importConfigCheckbox, 0, wxALIGN_CENTER_VERTICAL);

    m_configStatusLabel = new wxStaticText(m_panel, wxID_ANY, "");
    m_configStatusLabel->SetMinSize(wxSize(STATUS_LABEL_WIDTH, -1));
    configRowSizer->Add(m_configStatusLabel, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    m_clearConfigCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Clear existing Config");
    m_clearConfigCheckbox->SetValue(true);
    configRowSizer->Add(m_clearConfigCheckbox, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    mainSizer->Add(configRowSizer, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Matches row
    wxBoxSizer *matchesRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_importMatchesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Matches");
    m_importMatchesCheckbox->SetValue(true);
    m_importMatchesCheckbox->SetMinSize(wxSize(IMPORT_CHECKBOX_WIDTH, -1));
    m_importMatchesCheckbox->Bind(wxEVT_CHECKBOX, &ImportFrame::OnImportCheckboxChanged, this);
    matchesRowSizer->Add(m_importMatchesCheckbox, 0, wxALIGN_CENTER_VERTICAL);

    m_matchesStatusLabel = new wxStaticText(m_panel, wxID_ANY, "");
    m_matchesStatusLabel->SetMinSize(wxSize(STATUS_LABEL_WIDTH, -1));
    matchesRowSizer->Add(m_matchesStatusLabel, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    m_clearMatchesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Clear existing Matches");
    m_clearMatchesCheckbox->SetValue(true);
    matchesRowSizer->Add(m_clearMatchesCheckbox, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    mainSizer->Add(matchesRowSizer, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Packages row
    wxBoxSizer *packagesRowSizer = new wxBoxSizer(wxHORIZONTAL);
    m_importPackagesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Packages");
    m_importPackagesCheckbox->SetValue(true);
    m_importPackagesCheckbox->SetMinSize(wxSize(IMPORT_CHECKBOX_WIDTH, -1));
    m_importPackagesCheckbox->Bind(wxEVT_CHECKBOX, &ImportFrame::OnImportCheckboxChanged, this);
    packagesRowSizer->Add(m_importPackagesCheckbox, 0, wxALIGN_CENTER_VERTICAL);

    m_packagesStatusLabel = new wxStaticText(m_panel, wxID_ANY, "");
    m_packagesStatusLabel->SetMinSize(wxSize(STATUS_LABEL_WIDTH, -1));
    packagesRowSizer->Add(m_packagesStatusLabel, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    m_clearPackagesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Clear existing Packages");
    m_clearPackagesCheckbox->SetValue(true);
    packagesRowSizer->Add(m_clearPackagesCheckbox, 0, wxLEFT | wxALIGN_CENTER_VERTICAL, 10);

    mainSizer->Add(packagesRowSizer, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Separator
    mainSizer->Add(new wxStaticLine(m_panel), 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Button sizer (horizontal)
    wxBoxSizer *buttonSizer = new wxBoxSizer(wxHORIZONTAL);

    m_importButton = new wxButton(m_panel, wxID_ANY, "Import");
    m_importButton->Bind(wxEVT_BUTTON, &ImportFrame::OnImportButton, this);
    m_importButton->Disable();  // Disabled until valid data
    buttonSizer->Add(m_importButton, 0, wxRIGHT, 5);

    m_closeButton = new wxButton(m_panel, wxID_CLOSE, "Close");
    m_closeButton->Bind(wxEVT_BUTTON, &ImportFrame::OnCloseButton, this);
    buttonSizer->Add(m_closeButton, 0);

    mainSizer->Add(buttonSizer, 0, wxALIGN_RIGHT | wxALL, 10);

    // Set sizer
    m_panel->SetSizer(mainSizer);

    // Bind events
    Bind(wxEVT_CLOSE_WINDOW, &ImportFrame::OnClose, this);
    m_inputText->Bind(wxEVT_KILL_FOCUS, &ImportFrame::OnTextLostFocus, this);

    // Center on screen
    Centre();
}

void ImportFrame::OnPasteButton(wxCommandEvent &event) {
    if (wxTheClipboard->Open()) {
        if (wxTheClipboard->IsSupported(wxDF_TEXT)) {
            wxTextDataObject data;
            wxTheClipboard->GetData(data);
            m_inputText->SetValue(data.GetText());
        }
        wxTheClipboard->Close();
    }
    ValidateAndUpdateUI();
}

void ImportFrame::OnImportFromFileButton(wxCommandEvent &event) {
    wxFileDialog openFileDialog(this, "Import from file", "", "",
        "Espanso Export files (*.espanso)|*.espanso|Text files (*.txt)|*.txt|All files (*.*)|*.*",
        wxFD_OPEN | wxFD_FILE_MUST_EXIST);

    if (openFileDialog.ShowModal() == wxID_CANCEL) {
        return;
    }

    std::ifstream file(openFileDialog.GetPath().ToStdString());
    if (file.is_open()) {
        std::stringstream buffer;
        buffer << file.rdbuf();
        m_inputText->SetValue(wxString::FromUTF8(buffer.str()));
        file.close();
        ValidateAndUpdateUI();
    } else {
        wxMessageBox("Could not open the selected file.",
                    "Error", wxOK | wxICON_ERROR);
    }
}

void ImportFrame::OnCheckButton(wxCommandEvent &event) {
    // Move focus away from text control to trigger validation
    m_checkButton->SetFocus();
    ValidateAndUpdateUI();
}

void ImportFrame::OnTextLostFocus(wxFocusEvent &event) {
    ValidateAndUpdateUI();
    event.Skip();
}

void ImportFrame::OnImportCheckboxChanged(wxCommandEvent &event) {
    wxCheckBox *checkbox = dynamic_cast<wxCheckBox*>(event.GetEventObject());

    // When import checkbox is unchecked, uncheck the corresponding clear checkbox
    if (checkbox && !checkbox->GetValue()) {
        if (checkbox == m_importConfigCheckbox) {
            m_clearConfigCheckbox->SetValue(false);
        } else if (checkbox == m_importMatchesCheckbox) {
            m_clearMatchesCheckbox->SetValue(false);
        } else if (checkbox == m_importPackagesCheckbox) {
            m_clearPackagesCheckbox->SetValue(false);
        }
    }

    UpdateImportButtonState();
}

void ImportFrame::ValidateAndUpdateUI() {
    wxString inputText = m_inputText->GetValue();

    // Reset status
    m_configStatusValue = SCOPE_STATUS_EMPTY;
    m_matchesStatusValue = SCOPE_STATUS_EMPTY;
    m_packagesStatusValue = SCOPE_STATUS_EMPTY;
    m_isValid = false;

    if (!inputText.IsEmpty()) {
        // Call validation callback if available
        if (g_importMetadata && g_importMetadata->validate_import_data) {
            std::string utf8Input = inputText.ToUTF8().data();
            int result = g_importMetadata->validate_import_data(
                utf8Input.c_str(),
                &m_configStatusValue,
                &m_matchesStatusValue,
                &m_packagesStatusValue
            );
            m_isValid = (result != 0);
        }
    }

    // Update each row's UI (pass previous status to detect transitions)
    UpdateRowUI(m_importConfigCheckbox, m_configStatusLabel, m_clearConfigCheckbox,
                m_configStatusValue, m_prevConfigStatus);
    UpdateRowUI(m_importMatchesCheckbox, m_matchesStatusLabel, m_clearMatchesCheckbox,
                m_matchesStatusValue, m_prevMatchesStatus);
    UpdateRowUI(m_importPackagesCheckbox, m_packagesStatusLabel, m_clearPackagesCheckbox,
                m_packagesStatusValue, m_prevPackagesStatus);

    // Update previous status values
    m_prevConfigStatus = m_configStatusValue;
    m_prevMatchesStatus = m_matchesStatusValue;
    m_prevPackagesStatus = m_packagesStatusValue;

    UpdateImportButtonState();
}

void ImportFrame::UpdateRowUI(wxCheckBox *importCheckbox, wxStaticText *statusLabel,
                               wxCheckBox *clearCheckbox, int status, int prevStatus) {
    wxString text;
    wxColour colour;

    // Detect if status just transitioned to MISSING or INVALID
    bool justBecameUnavailable = (status == SCOPE_STATUS_MISSING || status == SCOPE_STATUS_INVALID) &&
                                  (prevStatus != SCOPE_STATUS_MISSING && prevStatus != SCOPE_STATUS_INVALID);

    switch (status) {
        case SCOPE_STATUS_EMPTY:
            text = "";
            colour = wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT);
            // Enable both checkboxes but don't change their state
            importCheckbox->Enable(true);
            clearCheckbox->Enable(true);
            break;
        case SCOPE_STATUS_PRESENT:
            text = "(found in export)";
            colour = wxColour(0, 160, 0);  // Green
            // Enable both checkboxes
            importCheckbox->Enable(true);
            clearCheckbox->Enable(true);
            break;
        case SCOPE_STATUS_MISSING:
            text = "(not in export)";
            colour = wxColour(128, 128, 128);  // Gray
            // Disable import checkbox but enable clear checkbox
            importCheckbox->Enable(false);
            importCheckbox->SetValue(false);
            clearCheckbox->Enable(true);
            // Only uncheck clear checkbox on transition to this state
            if (justBecameUnavailable) {
                clearCheckbox->SetValue(false);
            }
            break;
        case SCOPE_STATUS_INVALID:
            text = "(invalid data)";
            colour = wxColour(200, 0, 0);  // Red
            // Disable both checkboxes but preserve their state
            importCheckbox->Enable(false);
            clearCheckbox->Enable(false);
            break;
    }

    statusLabel->SetLabel(text);
    statusLabel->SetForegroundColour(colour);
    statusLabel->Refresh();
}

void ImportFrame::UpdateImportButtonState() {
    // Import button is enabled only if:
    // 1. Data is valid (at least one scope present)
    // 2. At least one "import" checkbox is checked for a present scope

    bool canImport = false;

    if (m_isValid) {
        if (m_importConfigCheckbox->GetValue() && m_configStatusValue == SCOPE_STATUS_PRESENT) {
            canImport = true;
        }
        if (m_importMatchesCheckbox->GetValue() && m_matchesStatusValue == SCOPE_STATUS_PRESENT) {
            canImport = true;
        }
        if (m_importPackagesCheckbox->GetValue() && m_packagesStatusValue == SCOPE_STATUS_PRESENT) {
            canImport = true;
        }
    }

    m_importButton->Enable(canImport);
}

void ImportFrame::OnImportButton(wxCommandEvent &event) {
    if (!m_isValid) {
        return;
    }

    // Gather import and clear selections
    int importConfig = (m_importConfigCheckbox->GetValue() && m_configStatusValue == SCOPE_STATUS_PRESENT) ? 1 : 0;
    int importMatches = (m_importMatchesCheckbox->GetValue() && m_matchesStatusValue == SCOPE_STATUS_PRESENT) ? 1 : 0;
    int importPackages = (m_importPackagesCheckbox->GetValue() && m_packagesStatusValue == SCOPE_STATUS_PRESENT) ? 1 : 0;

    int clearConfig = m_clearConfigCheckbox->GetValue() ? 1 : 0;
    int clearMatches = m_clearMatchesCheckbox->GetValue() ? 1 : 0;
    int clearPackages = m_clearPackagesCheckbox->GetValue() ? 1 : 0;

    // Perform import via callback
    if (g_importMetadata && g_importMetadata->perform_import) {
        wxString inputText = m_inputText->GetValue();
        std::string utf8Input = inputText.ToUTF8().data();

        int result = g_importMetadata->perform_import(
            utf8Input.c_str(),
            importConfig, importMatches, importPackages,
            clearConfig, clearMatches, clearPackages
        );

        if (result != 0) {
            wxMessageBox("Configuration imported successfully!\n\nEspanso will automatically detect the changes.",
                        "Import Complete", wxOK | wxICON_INFORMATION);
            Close(true);
        } else {
            wxMessageBox("An error occurred during import.\n\nPlease check the espanso logs for details.",
                        "Import Error", wxOK | wxICON_ERROR);
        }
    }
}

void ImportFrame::OnCloseButton(wxCommandEvent &event) {
    Close(true);
}

void ImportFrame::OnClose(wxCloseEvent &event) {
    Destroy();
}

// C interface
extern "C" void interop_show_import_dialog(const ImportDialogMetadata *metadata) {
// Setup high DPI support on Windows
#ifdef __WXMSW__
    SetProcessDPIAware();
#endif

    g_importMetadata = metadata;

    wxApp::SetInstance(new ImportDialogApp());

    int argc = 0;
    char **argv = NULL;
    wxEntry(argc, argv);
}
