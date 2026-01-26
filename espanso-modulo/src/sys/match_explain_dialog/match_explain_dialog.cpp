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

#define _UNICODE

#include "../common/common.h"
#include "../interop/interop.h"

#include <wx/clipbrd.h>
#include <wx/hyperlink.h>
#include <wx/utils.h>

#ifdef __WXMSW__
const long DEFAULT_STYLE = wxDEFAULT_FRAME_STYLE | wxSTAY_ON_TOP;
#endif
#ifdef __WXOSX__
const long DEFAULT_STYLE = wxDEFAULT_FRAME_STYLE | wxSTAY_ON_TOP;
#endif
#ifdef __LINUX__
const long DEFAULT_STYLE = wxDEFAULT_FRAME_STYLE | wxSTAY_ON_TOP;
#endif

MatchExplainDialogMetadata *match_explain_metadata = nullptr;

class MatchExplainDialogApp : public wxApp {
  public:
    bool OnInit() override;
};

class MatchExplainDialogFrame : public wxFrame {
  public:
    MatchExplainDialogFrame();

  private:
    wxTextCtrl *trigger_input = nullptr;
    wxButton *check_button = nullptr;
    wxButton *check_all_button = nullptr;
    wxTextCtrl *output_box = nullptr;
    wxStaticText *file_label = nullptr;
    wxHyperlinkCtrl *file_link = nullptr;
    wxCheckBox *json_checkbox = nullptr;
    wxButton *copy_button = nullptr;
    wxButton *close_button = nullptr;

    void OnCheck(wxCommandEvent &event);
    void OnCheckAll(wxCommandEvent &event);
    void OnCopy(wxCommandEvent &event);
    void OnCloseClick(wxCommandEvent &event);
    void OnActivate(wxActivateEvent &event);
    void OnClose(wxCloseEvent &event);
    void OnChar(wxKeyEvent &event);
    void OnFileLink(wxHyperlinkEvent &event);

    void NotifyFocusGained();
    void NotifyFocusLost();
    void RunCheck(bool show_all);
    void UpdateFileLink(const wxString &output, bool json_output);
};

MatchExplainDialogFrame::MatchExplainDialogFrame()
    : wxFrame(NULL, wxID_ANY, "Explain match", wxDefaultPosition,
              wxSize(720, 480), DEFAULT_STYLE) {
    wxPanel *panel = new wxPanel(this, wxID_ANY);
    wxBoxSizer *main_sizer = new wxBoxSizer(wxVERTICAL);
    panel->SetSizer(main_sizer);

    wxBoxSizer *top_sizer = new wxBoxSizer(wxHORIZONTAL);
    wxStaticText *trigger_label =
        new wxStaticText(panel, wxID_ANY, "Trigger");
    top_sizer->Add(trigger_label, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 8);

    trigger_input = new wxTextCtrl(panel, wxID_ANY, "", wxDefaultPosition,
                                   wxDefaultSize, wxTE_PROCESS_ENTER);
    top_sizer->Add(trigger_input, 1, wxRIGHT, 8);

    check_button = new wxButton(panel, wxID_ANY, "Check");
    top_sizer->Add(check_button, 0);

    check_all_button = new wxButton(panel, wxID_ANY, "Check all");
    top_sizer->Add(check_all_button, 0, wxLEFT, 8);

    main_sizer->Add(top_sizer, 0, wxEXPAND | wxALL, 10);

    output_box = new wxTextCtrl(panel, wxID_ANY, "", wxDefaultPosition,
                                wxDefaultSize,
                                wxTE_MULTILINE | wxTE_READONLY | wxTE_RICH2);
    wxFont output_font = output_box->GetFont();
    output_font.SetFamily(wxFONTFAMILY_TELETYPE);
    output_box->SetFont(output_font);
    main_sizer->Add(output_box, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    wxBoxSizer *file_sizer = new wxBoxSizer(wxHORIZONTAL);
    file_label = new wxStaticText(panel, wxID_ANY, "Defined in:");
    file_sizer->Add(file_label, 0, wxALIGN_CENTER_VERTICAL | wxRIGHT, 6);
    file_link = new wxHyperlinkCtrl(panel, wxID_ANY, "", "");
    file_sizer->Add(file_link, 1, wxALIGN_CENTER_VERTICAL);
    main_sizer->Add(file_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    wxBoxSizer *bottom_sizer = new wxBoxSizer(wxHORIZONTAL);
    json_checkbox = new wxCheckBox(panel, wxID_ANY, "JSON output");
    bottom_sizer->Add(json_checkbox, 0, wxALIGN_CENTER_VERTICAL);
    bottom_sizer->AddStretchSpacer(1);
    copy_button = new wxButton(panel, wxID_ANY, "Copy to Clipboard");
    close_button = new wxButton(panel, wxID_ANY, "Close");
    bottom_sizer->Add(copy_button, 0, wxRIGHT, 8);
    bottom_sizer->Add(close_button, 0);
    main_sizer->Add(bottom_sizer, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

    check_button->Bind(wxEVT_BUTTON, &MatchExplainDialogFrame::OnCheck, this);
    check_all_button->Bind(wxEVT_BUTTON, &MatchExplainDialogFrame::OnCheckAll,
                           this);
    copy_button->Bind(wxEVT_BUTTON, &MatchExplainDialogFrame::OnCopy, this);
    close_button->Bind(wxEVT_BUTTON, &MatchExplainDialogFrame::OnCloseClick,
                       this);
    trigger_input->Bind(wxEVT_TEXT_ENTER,
                        &MatchExplainDialogFrame::OnCheck, this);
    file_link->Bind(wxEVT_HYPERLINK, &MatchExplainDialogFrame::OnFileLink, this);

    Bind(wxEVT_ACTIVATE, &MatchExplainDialogFrame::OnActivate, this);
    Bind(wxEVT_CLOSE_WINDOW, &MatchExplainDialogFrame::OnClose, this);
    Bind(wxEVT_CHAR_HOOK, &MatchExplainDialogFrame::OnChar, this);

    file_label->Hide();
    file_link->Hide();
    Centre();
}

void MatchExplainDialogFrame::NotifyFocusGained() {
    if (match_explain_metadata && match_explain_metadata->on_focus_gained) {
        match_explain_metadata->on_focus_gained();
    }
}

void MatchExplainDialogFrame::NotifyFocusLost() {
    if (match_explain_metadata && match_explain_metadata->on_focus_lost) {
        match_explain_metadata->on_focus_lost();
    }
}

void MatchExplainDialogFrame::OnActivate(wxActivateEvent &event) {
    if (event.GetActive()) {
        NotifyFocusGained();
    } else {
        NotifyFocusLost();
    }
    event.Skip();
}

void MatchExplainDialogFrame::OnClose(wxCloseEvent &event) {
    NotifyFocusLost();
    event.Skip();
}

void MatchExplainDialogFrame::OnCloseClick(wxCommandEvent &event) {
    Close(true);
}

void MatchExplainDialogFrame::OnChar(wxKeyEvent &event) {
    if (event.GetKeyCode() == WXK_ESCAPE) {
        Close(true);
        return;
    }
    event.Skip();
}

void MatchExplainDialogFrame::OnCheck(wxCommandEvent &event) {
    RunCheck(false);
}

void MatchExplainDialogFrame::OnCheckAll(wxCommandEvent &event) {
    RunCheck(true);
}

void MatchExplainDialogFrame::RunCheck(bool show_all) {
    if (!match_explain_metadata || !match_explain_metadata->on_check) {
        return;
    }

    wxString trigger = trigger_input->GetValue();
    const char *response =
        match_explain_metadata->on_check(trigger.utf8_str(), show_all,
                                         json_checkbox->IsChecked() ? 1 : 0);
    if (response) {
        wxString output = wxString::FromUTF8(response);
        output_box->SetValue(output);
        UpdateFileLink(output, json_checkbox->IsChecked());
    } else {
        output_box->SetValue("");
        UpdateFileLink("", json_checkbox->IsChecked());
    }
}

void MatchExplainDialogFrame::OnCopy(wxCommandEvent &event) {
    if (wxTheClipboard->Open()) {
        wxTheClipboard->SetData(
            new wxTextDataObject(output_box->GetValue()));
        wxTheClipboard->Close();
    }
}

void MatchExplainDialogFrame::OnFileLink(wxHyperlinkEvent &event) {
    wxString path = file_link->GetURL();
    if (!path.IsEmpty()) {
        wxLaunchDefaultApplication(path);
    }
}

void MatchExplainDialogFrame::UpdateFileLink(const wxString &output,
                                             bool json_output) {
    wxString source_path;

    if (json_output) {
        const wxString marker = "\"source_file\": \"";
        int pos = output.Find(marker);
        if (pos != wxNOT_FOUND) {
            pos += marker.Length();
            wxString tail = output.Mid(pos);
            int rel_end = tail.Find("\"");
            if (rel_end != wxNOT_FOUND) {
                int end = pos + rel_end;
                source_path = output.Mid(pos, end - pos);
            }
        }
    } else {
        const wxString marker = "Defined in: ";
        int pos = output.Find(marker);
        if (pos != wxNOT_FOUND) {
            pos += marker.Length();
            wxString tail = output.Mid(pos);
            int rel_end = tail.Find("\n");
            int end = rel_end == wxNOT_FOUND ? output.Length() : pos + rel_end;
            source_path =
                output.Mid(pos, end - pos).Trim(true).Trim(false);
        }
    }

    if (!source_path.IsEmpty()) {
        file_link->SetLabel(source_path);
        file_link->SetURL(source_path);
        file_label->Show();
        file_link->Show();
    } else {
        file_label->Hide();
        file_link->Hide();
    }

    Layout();
}

bool MatchExplainDialogApp::OnInit() {
    MatchExplainDialogFrame *frame = new MatchExplainDialogFrame();
    if (match_explain_metadata && match_explain_metadata->window_icon_path) {
        setFrameIcon(
            wxString::FromUTF8(match_explain_metadata->window_icon_path), frame);
    }

    frame->Show(true);
    Activate(frame);
    if (match_explain_metadata && match_explain_metadata->on_focus_gained) {
        match_explain_metadata->on_focus_gained();
    }

    return true;
}

extern "C" void interop_show_match_explain_dialog(
    MatchExplainDialogMetadata *_metadata) {
#ifdef __WXMSW__
    SetProcessDPIAware();
#endif

    match_explain_metadata = _metadata;

    wxApp::SetInstance(new MatchExplainDialogApp());
    int argc = 0;
    wxEntry(argc, (char **)nullptr);
}
