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
#include <wx/filedlg.h>
#include <fstream>

#ifdef __WXMSW__
#include <windows.h>
#endif

// Window style - stay on top so dialog doesn't get lost behind other windows
const long EXPORT_DIALOG_STYLE = wxDEFAULT_FRAME_STYLE | wxSTAY_ON_TOP;

// Forward declarations
class ExportFrame;

// Metadata pointer for callback
const ExportDialogMetadata *g_exportMetadata = nullptr;

// Application class
class ExportDialogApp : public wxApp {
public:
    virtual bool OnInit() wxOVERRIDE;
};

// Frame class (using wxFrame like other modulo dialogs)
class ExportFrame : public wxFrame {
public:
    ExportFrame(const wxString &title, const wxPoint &pos, const wxSize &size);

private:
    // Event handlers
    void OnCheckboxChanged(wxCommandEvent &event);
    void OnCopyButton(wxCommandEvent &event);
    void OnExportToFileButton(wxCommandEvent &event);
    void OnCloseButton(wxCommandEvent &event);
    void OnClose(wxCloseEvent &event);

    // Helper methods
    void UpdatePreview();
    wxString GenerateExportCode();

    // UI controls
    wxPanel *m_panel;
    wxCheckBox *m_configCheckbox;
    wxCheckBox *m_matchesCheckbox;
    wxCheckBox *m_packagesCheckbox;
    wxTextCtrl *m_previewText;
    wxButton *m_copyButton;
    wxButton *m_exportToFileButton;
    wxButton *m_closeButton;
};

// Application implementation
bool ExportDialogApp::OnInit() {
    if (!wxApp::OnInit()) {
        return false;
    }

    ExportFrame *frame = new ExportFrame(
        "Export Espanso Configuration",
        wxDefaultPosition,
        wxSize(700, 550)
    );

    if (g_exportMetadata && g_exportMetadata->window_icon_path) {
        setFrameIcon(wxString::FromUTF8(g_exportMetadata->window_icon_path), frame);
    }

    frame->Show(true);
    Activate(frame);

    return true;
}

// Frame implementation
ExportFrame::ExportFrame(const wxString &title, const wxPoint &pos, const wxSize &size)
    : wxFrame(NULL, wxID_ANY, title, pos, size, EXPORT_DIALOG_STYLE)
{
    m_panel = new wxPanel(this, wxID_ANY);

    // Create main vertical sizer
    wxBoxSizer *mainSizer = new wxBoxSizer(wxVERTICAL);

    // Title text
    wxStaticText *titleText = new wxStaticText(m_panel, wxID_ANY,
        "Select what to export:");
    mainSizer->Add(titleText, 0, wxALL, 10);

    // Checkboxes section
    m_configCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Config Files");
    m_configCheckbox->SetValue(true);
    m_configCheckbox->Bind(wxEVT_CHECKBOX, &ExportFrame::OnCheckboxChanged, this);
    mainSizer->Add(m_configCheckbox, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    m_matchesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Matches");
    m_matchesCheckbox->SetValue(true);
    m_matchesCheckbox->Bind(wxEVT_CHECKBOX, &ExportFrame::OnCheckboxChanged, this);
    mainSizer->Add(m_matchesCheckbox, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    m_packagesCheckbox = new wxCheckBox(m_panel, wxID_ANY, "Packages");
    m_packagesCheckbox->SetValue(true);
    m_packagesCheckbox->Bind(wxEVT_CHECKBOX, &ExportFrame::OnCheckboxChanged, this);
    mainSizer->Add(m_packagesCheckbox, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Preview label
    wxStaticText *previewLabel = new wxStaticText(m_panel, wxID_ANY,
        "Export Code (copy and save this):");
    mainSizer->Add(previewLabel, 0, wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Preview text control with monospace font
    m_previewText = new wxTextCtrl(m_panel, wxID_ANY, wxEmptyString,
        wxDefaultPosition, wxSize(600, 300),
        wxTE_MULTILINE | wxTE_READONLY);

    wxFont monoFont(12, wxFONTFAMILY_TELETYPE, wxFONTSTYLE_NORMAL, wxFONTWEIGHT_NORMAL);
    m_previewText->SetFont(monoFont);

    mainSizer->Add(m_previewText, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    // Button sizer (horizontal)
    wxBoxSizer *buttonSizer = new wxBoxSizer(wxHORIZONTAL);

    m_copyButton = new wxButton(m_panel, wxID_ANY, "Copy to Clipboard");
    m_copyButton->Bind(wxEVT_BUTTON, &ExportFrame::OnCopyButton, this);
    buttonSizer->Add(m_copyButton, 0, wxRIGHT, 5);

    m_exportToFileButton = new wxButton(m_panel, wxID_ANY, "Export to file");
    m_exportToFileButton->Bind(wxEVT_BUTTON, &ExportFrame::OnExportToFileButton, this);
    buttonSizer->Add(m_exportToFileButton, 0, wxRIGHT, 5);

    m_closeButton = new wxButton(m_panel, wxID_CLOSE, "Close");
    m_closeButton->Bind(wxEVT_BUTTON, &ExportFrame::OnCloseButton, this);
    buttonSizer->Add(m_closeButton, 0);

    mainSizer->Add(buttonSizer, 0, wxALIGN_RIGHT | wxALL, 10);

    // Set sizer
    m_panel->SetSizer(mainSizer);

    // Bind close event
    Bind(wxEVT_CLOSE_WINDOW, &ExportFrame::OnClose, this);

    // Center on screen
    Centre();

    // Generate initial preview
    UpdatePreview();
}

void ExportFrame::OnCheckboxChanged(wxCommandEvent &event) {
    UpdatePreview();
}

void ExportFrame::OnCopyButton(wxCommandEvent &event) {
    if (wxTheClipboard->Open()) {
        wxTheClipboard->SetData(new wxTextDataObject(m_previewText->GetValue()));
        wxTheClipboard->Flush();  // Ensure clipboard data persists after app closes
        wxTheClipboard->Close();

        // Visual feedback
        wxString originalLabel = m_copyButton->GetLabel();
        m_copyButton->SetLabel("Copied!");
        m_copyButton->Refresh();
        m_copyButton->Update();

        wxMilliSleep(800);

        m_copyButton->SetLabel(originalLabel);
        m_copyButton->Refresh();
    }
}

void ExportFrame::OnExportToFileButton(wxCommandEvent &event) {
    wxString exportCode = m_previewText->GetValue();

    if (exportCode.IsEmpty()) {
        wxMessageBox("Nothing to export. Please select at least one option.",
                    "Export", wxOK | wxICON_WARNING);
        return;
    }

    wxFileDialog saveFileDialog(this, "Export to file", "", "espanso_export.espanso",
        "Espanso Export files (*.espanso)|*.espanso|Text files (*.txt)|*.txt|All files (*.*)|*.*",
        wxFD_SAVE | wxFD_OVERWRITE_PROMPT);

    if (saveFileDialog.ShowModal() == wxID_CANCEL) {
        return;
    }

    std::ofstream file(saveFileDialog.GetPath().ToStdString());
    if (file.is_open()) {
        file << exportCode.ToUTF8().data();
        file.close();

        // Visual feedback
        wxString originalLabel = m_exportToFileButton->GetLabel();
        m_exportToFileButton->SetLabel("Saved!");
        m_exportToFileButton->Refresh();
        m_exportToFileButton->Update();

        wxMilliSleep(800);

        m_exportToFileButton->SetLabel(originalLabel);
        m_exportToFileButton->Refresh();
    } else {
        wxMessageBox("Could not save to the selected file.",
                    "Error", wxOK | wxICON_ERROR);
    }
}

void ExportFrame::OnCloseButton(wxCommandEvent &event) {
    Close(true);
}

void ExportFrame::OnClose(wxCloseEvent &event) {
    Destroy();
}

void ExportFrame::UpdatePreview() {
    wxString exportCode = GenerateExportCode();
    m_previewText->SetValue(exportCode);
}

wxString ExportFrame::GenerateExportCode() {
    int exportConfig = m_configCheckbox->GetValue() ? 1 : 0;
    int exportMatches = m_matchesCheckbox->GetValue() ? 1 : 0;
    int exportPackages = m_packagesCheckbox->GetValue() ? 1 : 0;

    // If nothing is selected, show empty field
    if (!exportConfig && !exportMatches && !exportPackages) {
        return "";
    }

    if (!g_exportMetadata || !g_exportMetadata->generate_export_code) {
        return "Error: Export function not available";
    }

    const char *code = g_exportMetadata->generate_export_code(
        exportConfig, exportMatches, exportPackages);

    if (code) {
        return wxString::FromUTF8(code);
    }

    return "Error generating export code";
}

// C interface
extern "C" void interop_show_export_dialog(const ExportDialogMetadata *metadata) {
// Setup high DPI support on Windows
#ifdef __WXMSW__
    SetProcessDPIAware();
#endif

    g_exportMetadata = metadata;

    wxApp::SetInstance(new ExportDialogApp());

    int argc = 0;
    char **argv = NULL;
    wxEntry(argc, argv);
}
