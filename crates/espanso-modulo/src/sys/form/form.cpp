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

#include <vector>
#include <memory>
#include <unordered_map>

// https://docs.wxwidgets.org/stable/classwx_frame.html
const long DEFAULT_STYLE = wxSTAY_ON_TOP | wxCLOSE_BOX | wxCAPTION;

const int PADDING = 5;
const int MULTILINE_MIN_HEIGHT = 100;
const int MULTILINE_MIN_WIDTH = 100;

FormMetadata *formMetadata = nullptr;
std::vector<ValuePair> values;

// Field Wrappers

class FieldWrapper {
public:
    virtual wxString getValue() = 0;
};

class TextFieldWrapper {
    wxTextCtrl * control;
public:
    explicit TextFieldWrapper(wxTextCtrl * control): control(control) {}

    virtual wxString getValue() {
        return control->GetValue();
    }
};

class ChoiceFieldWrapper {
    wxChoice * control;
public:
    explicit ChoiceFieldWrapper(wxChoice * control): control(control) {}

    virtual wxString getValue() {
        return control->GetStringSelection();
    }
};

class ListFieldWrapper {
    wxListBox * control;
public:
    explicit ListFieldWrapper(wxListBox * control): control(control) {}

    virtual wxString getValue() {
        return control->GetStringSelection();
    }
};

// App Code

class FormApp: public wxApp
{
public:
    virtual bool OnInit();
};
class FormFrame: public wxFrame
{
public:
    FormFrame(const wxString& title, const wxPoint& pos, const wxSize& size);

    wxPanel *panel;
    std::vector<void *> fields;
    std::unordered_map<const char *, std::unique_ptr<FieldWrapper>> idMap;
    wxButton *submit;
    wxStaticText *helpText;
    bool hasFocusedMultilineControl;
private:
    void AddComponent(wxPanel *parent, wxBoxSizer *sizer, FieldMetadata meta);
    void Submit();
    void OnSubmitBtn(wxCommandEvent& event);
    void OnCharHook(wxKeyEvent& event);
    void OnListBoxEvent(wxCommandEvent& event);
    void UpdateHelpText();
    void HandleNormalFocus(wxFocusEvent& event);
    void HandleMultilineFocus(wxFocusEvent& event);
};
enum
{
    ID_Submit = 20000
};

bool FormApp::OnInit()
{
    FormFrame *frame = new FormFrame(wxString::FromUTF8(formMetadata->windowTitle), wxPoint(50, 50), wxSize(450, 340) );
    setFrameIcon(wxString::FromUTF8(formMetadata->iconPath), frame);
    frame->Show( true );
    
    Activate(frame);

    return true;
}
FormFrame::FormFrame(const wxString& title, const wxPoint& pos, const wxSize& size)
        : wxFrame(NULL, wxID_ANY, title, pos, size, DEFAULT_STYLE)
{
    hasFocusedMultilineControl = false;

    panel = new wxPanel(this, wxID_ANY);
    wxBoxSizer *vbox = new wxBoxSizer(wxVERTICAL);
    panel->SetSizer(vbox);

    for (int field = 0; field < formMetadata->fieldSize; field++) {
        FieldMetadata meta = formMetadata->fields[field];
        AddComponent(panel, vbox, meta);
    }

    submit = new wxButton(panel, ID_Submit, "Submit");
    vbox->Add(submit, 1, wxEXPAND | wxALL, PADDING);

    helpText = new wxStaticText(panel, wxID_ANY, "", wxDefaultPosition, wxDefaultSize);
    wxFont helpFont = helpText->GetFont();
    helpFont.SetPointSize(8);
    helpText->SetFont(helpFont);
    vbox->Add(helpText, 0, wxLEFT | wxRIGHT | wxBOTTOM, PADDING);
    UpdateHelpText();

    Bind(wxEVT_BUTTON, &FormFrame::OnSubmitBtn, this, ID_Submit);
    Bind(wxEVT_CHAR_HOOK, &FormFrame::OnCharHook, this, wxID_ANY);

    this->SetClientSize(panel->GetBestSize());
    this->CentreOnScreen();
}

void FormFrame::AddComponent(wxPanel *parent, wxBoxSizer *sizer, FieldMetadata meta) {
    void * control = nullptr;

    switch (meta.fieldType) {
        case FieldType::LABEL:
        {
            const LabelMetadata *labelMeta = static_cast<const LabelMetadata*>(meta.specific);
            auto label = new wxStaticText(parent, wxID_ANY, wxString::FromUTF8(labelMeta->text), wxDefaultPosition, wxDefaultSize);
            control = label;
            fields.push_back(label);
            break;
        }
        case FieldType::TEXT:
        {
            const TextMetadata *textMeta = static_cast<const TextMetadata*>(meta.specific);
            long style = 0;
            if (textMeta->multiline) {
                style |= wxTE_MULTILINE;
            }

            auto textControl = new wxTextCtrl(parent, NewControlId(), wxString::FromUTF8(textMeta->defaultText), wxDefaultPosition, wxDefaultSize, style);
            
            if (textMeta->multiline) {
                textControl->SetMinSize(wxSize(MULTILINE_MIN_WIDTH, MULTILINE_MIN_HEIGHT));
                textControl->Bind(wxEVT_SET_FOCUS, &FormFrame::HandleMultilineFocus, this, wxID_ANY);
            } else {
                textControl->Bind(wxEVT_SET_FOCUS, &FormFrame::HandleNormalFocus, this, wxID_ANY);
            }

            // Create the field wrapper
            std::unique_ptr<FieldWrapper> field((FieldWrapper*) new TextFieldWrapper(textControl));
            idMap[meta.id] = std::move(field);
            control = textControl;
            fields.push_back(textControl);
            break;
        }
        case FieldType::CHOICE:
        {
            const ChoiceMetadata *choiceMeta = static_cast<const ChoiceMetadata*>(meta.specific);

            int selectedItem = -1;
            wxArrayString choices;
            for (int i = 0; i<choiceMeta->valueSize; i++) {
                choices.Add(wxString::FromUTF8(choiceMeta->values[i]));

                if (strcmp(choiceMeta->values[i], choiceMeta->defaultValue) == 0) {
                    selectedItem = i;
                }
            }

            void * choice = nullptr;
            if (choiceMeta->choiceType == ChoiceType::DROPDOWN) {
                choice = (void*) new wxChoice(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, choices);

                if (selectedItem >= 0) {
                    ((wxChoice*)choice)->SetSelection(selectedItem);
                }

                ((wxChoice*)choice)->Bind(wxEVT_SET_FOCUS, &FormFrame::HandleNormalFocus, this, wxID_ANY);

                // Create the field wrapper
                std::unique_ptr<FieldWrapper> field((FieldWrapper*) new ChoiceFieldWrapper((wxChoice*) choice));
                idMap[meta.id] = std::move(field);
            }else {
                choice = (void*) new wxListBox(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, choices);
                
                if (selectedItem >= 0) {
                    ((wxListBox*)choice)->SetSelection(selectedItem);
                }

                ((wxListBox*)choice)->Bind(wxEVT_SET_FOCUS, &FormFrame::HandleNormalFocus, this, wxID_ANY);
                // ListBoxes prevent the global CHAR_HOOK handler from handling the Return key
                // correctly, so we need to handle the double click event too (which is triggered
                // when the enter key is pressed).
                // See: https://github.com/espanso/espanso/issues/857
                ((wxListBox*)choice)->Bind(wxEVT_LISTBOX_DCLICK, &FormFrame::OnListBoxEvent, this, wxID_ANY);
                
                // Create the field wrapper
                std::unique_ptr<FieldWrapper> field((FieldWrapper*) new ListFieldWrapper((wxListBox*) choice));
                idMap[meta.id] = std::move(field);
            }
            
            control = choice;
            fields.push_back(choice);
            break;
        }
        case FieldType::ROW:
        {
            const RowMetadata *rowMeta = static_cast<const RowMetadata*>(meta.specific);

            auto innerPanel = new wxPanel(panel, wxID_ANY);
            wxBoxSizer *hbox = new wxBoxSizer(wxHORIZONTAL);
            innerPanel->SetSizer(hbox);
            sizer->Add(innerPanel, 0, wxEXPAND | wxALL, 0);
            fields.push_back(innerPanel);

            for (int field = 0; field < rowMeta->fieldSize; field++) {
                FieldMetadata innerMeta = rowMeta->fields[field];
                AddComponent(innerPanel, hbox, innerMeta);
            }

            break;
        }
        default:
            // TODO: handle unknown field type
            break;
    }

    if (control) {
        sizer->Add((wxWindow*) control, 0, wxEXPAND | wxALL, PADDING);
    }
}

void FormFrame::Submit() {
    for (auto& field: idMap) {
        FieldWrapper * fieldWrapper = (FieldWrapper*) field.second.get();
        wxString value {fieldWrapper->getValue()};
        wxCharBuffer buffer {value.ToUTF8()};
        char * id = strdup(field.first);
        char * c_value = strdup(buffer.data());
        ValuePair valuePair = {
            id,
            c_value,
        };
        values.push_back(valuePair);
    }

    Close(true);
}

void FormFrame::HandleNormalFocus(wxFocusEvent& event) {
    hasFocusedMultilineControl = false;
    UpdateHelpText();
    event.Skip();
}

void FormFrame::HandleMultilineFocus(wxFocusEvent& event) {
    hasFocusedMultilineControl = true;
    UpdateHelpText();
    event.Skip();
}

void FormFrame::UpdateHelpText() {
    if (hasFocusedMultilineControl) {
        helpText->SetLabel("(or press CTRL+Enter to submit, ESC to cancel)");
    } else {
        helpText->SetLabel("(or press Enter to submit, ESC to cancel)");
    }
    this->SetClientSize(panel->GetBestSize());
}

void FormFrame::OnSubmitBtn(wxCommandEvent &event) {
    Submit();
}

void FormFrame::OnCharHook(wxKeyEvent& event) {
    if (event.GetKeyCode() == WXK_ESCAPE) {
        Close(true);
    }else if(event.GetKeyCode() == WXK_RETURN) {
        if (!hasFocusedMultilineControl || wxGetKeyState(WXK_RAW_CONTROL)) {
            Submit();
        } else {
            event.Skip();
        }
    }else{
        event.Skip();
    }
}

void FormFrame::OnListBoxEvent(wxCommandEvent& event) {
    Submit();
}

extern "C" void interop_show_form(FormMetadata * _metadata, void (*callback)(ValuePair *values, int size, void *data), void *data) {
    // Setup high DPI support on Windows
    #ifdef __WXMSW__
        SetProcessDPIAware();
    #endif
    
    formMetadata = _metadata;

    wxApp::SetInstance(new FormApp());
    int argc = 0;
    wxEntry(argc, (char **)nullptr);

    callback(values.data(), values.size(), data);

    // Free up values
    for (auto pair: values) {
        free((void*) pair.id);
        free((void*) pair.value);
    }
}