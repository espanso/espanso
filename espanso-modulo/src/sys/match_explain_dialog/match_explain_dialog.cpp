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
#include <wx/richtext/richtextctrl.h>
#include <wx/tokenzr.h>
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
    wxRichTextCtrl *output_box = nullptr;
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
    void OnTextUrl(wxTextUrlEvent &event);

    void NotifyFocusGained();
    void NotifyFocusLost();
    void RunCheck(bool show_all);
    void UpdateOutput(const wxString &output, bool json_output);
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

    output_box = new wxRichTextCtrl(panel, wxID_ANY, "", wxDefaultPosition,
                                    wxDefaultSize,
                                    wxTE_MULTILINE | wxTE_READONLY);
    wxFont output_font = output_box->GetFont();
    output_font.SetFamily(wxFONTFAMILY_TELETYPE);
    wxTextAttr output_attr;
    output_attr.SetFont(output_font);
    output_box->SetDefaultStyle(output_attr);
    main_sizer->Add(output_box, 1, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);

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
    output_box->Bind(wxEVT_TEXT_URL, &MatchExplainDialogFrame::OnTextUrl, this);

    Bind(wxEVT_ACTIVATE, &MatchExplainDialogFrame::OnActivate, this);
    Bind(wxEVT_CLOSE_WINDOW, &MatchExplainDialogFrame::OnClose, this);
    Bind(wxEVT_CHAR_HOOK, &MatchExplainDialogFrame::OnChar, this);

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
        UpdateOutput(output, json_checkbox->IsChecked());
    } else {
        UpdateOutput("", json_checkbox->IsChecked());
    }
}

void MatchExplainDialogFrame::OnCopy(wxCommandEvent &event) {
    if (wxTheClipboard->Open()) {
        wxTheClipboard->SetData(
            new wxTextDataObject(output_box->GetValue()));
        wxTheClipboard->Close();
    }
}

void MatchExplainDialogFrame::OnTextUrl(wxTextUrlEvent &event) {
    const long start = event.GetURLStart();
    const long end = event.GetURLEnd();
    wxString path;
    if (start >= 0 && end >= start) {
        path = output_box->GetRange(start, end + 1);
    }
    path = path.Trim(true).Trim(false);
    if (!path.IsEmpty()) {
        if (match_explain_metadata && match_explain_metadata->on_open_file) {
            match_explain_metadata->on_open_file(path.utf8_str());
        } else {
            wxLaunchDefaultApplication(path);
        }
    }
}

void MatchExplainDialogFrame::UpdateOutput(const wxString &output,
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
        const wxString marker = "Defined in:";
        wxStringTokenizer tokenizer(output, "\n", wxTOKEN_RET_EMPTY);
        while (tokenizer.HasMoreTokens()) {
            wxString line = tokenizer.GetNextToken();
            wxString trimmed = line.Trim(true).Trim(false);
            if (trimmed.StartsWith(marker)) {
                source_path = trimmed.Mid(marker.Length()).Trim(true).Trim(false);
                break;
            }
        }
    }

    output_box->Freeze();
    output_box->Clear();

    if (source_path.IsEmpty()) {
        output_box->WriteText(output);
        output_box->Thaw();
        return;
    }

    int link_pos = output.Find(source_path);
    if (link_pos == wxNOT_FOUND) {
        output_box->WriteText(output);
        output_box->Thaw();
        return;
    }

    wxString before = output.Left(link_pos);
    wxString after = output.Mid(link_pos + source_path.Length());

    output_box->SetDefaultStyle(wxTextAttr());
    output_box->WriteText(before);
    wxTextAttr link_style;
    link_style.SetFontWeight(wxFONTWEIGHT_BOLD);
    link_style.SetFontUnderlined(true);
    output_box->BeginStyle(link_style);
    output_box->BeginURL(source_path);
    output_box->WriteText(source_path);
    output_box->EndURL();
    output_box->EndStyle();
    output_box->SetDefaultStyle(wxTextAttr());
    output_box->WriteText(after);
    output_box->Thaw();
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
