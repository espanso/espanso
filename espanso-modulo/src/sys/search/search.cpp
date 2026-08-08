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

// Mouse dragging mechanism greatly inspired by:
// https://developpaper.com/wxwidgets-implementing-the-drag-effect-of-titleless-bar-window/

#define _UNICODE

#include "../common/common.h"
#include "../interop/interop.h"

#include "wx/htmllbox.h"

#include <memory>
#include <unordered_map>
#include <vector>

#ifdef __WXOSX__
// Implemented in search_mac.mm. Returns true if the key window's first
// responder currently has marked (uncommitted) text from an IME
// composition (e.g. selecting a CJK candidate).
extern bool IsImeComposingInKeyWindow();
#endif

// Platform-specific styles
#ifdef __WXMSW__
const int SEARCH_BAR_FONT_SIZE = 16;
const long DEFAULT_STYLE = wxSTAY_ON_TOP | wxFRAME_TOOL_WINDOW | wxRESIZE_BORDER;
#endif
#ifdef __WXOSX__
const int SEARCH_BAR_FONT_SIZE = 20;
const long DEFAULT_STYLE = wxSTAY_ON_TOP | wxRESIZE_BORDER;
#endif
#ifdef __LINUX__
const int SEARCH_BAR_FONT_SIZE = 20;
const long DEFAULT_STYLE = wxSTAY_ON_TOP | wxRESIZE_BORDER;
#endif

const int HELP_TEXT_FONT_SIZE = 10;

const wxColour SELECTION_LIGHT_BG = wxColour(164, 210, 253);
const wxColour SELECTION_DARK_BG = wxColour(49, 88, 126);

// https://docs.wxwidgets.org/stable/classwx_frame.html
const int MIN_WIDTH = 500;
const int MIN_HEIGHT = 80;

typedef void (*QueryCallback)(const char *query, void *app, void *data);
typedef void (*ResultCallback)(const char *id, void *data);

SearchMetadata *searchMetadata = nullptr;
QueryCallback queryCallback = nullptr;
ResultCallback resultCallback = nullptr;
void *data = nullptr;
void *resultData = nullptr;
wxArrayString wxItems;
wxArrayString wxTriggers;
wxArrayString wxIds;

// App Code

class SearchApp : public wxApp {
  public:
    virtual bool OnInit();
};

class ResultListBox : public wxHtmlListBox {
  public:
    ResultListBox() {}
    ResultListBox(wxWindow *parent, bool isDark, const wxWindowID id,
                  const wxPoint &pos, const wxSize &size);
    void SetDark(bool dark);

  protected:
    // override this method to return data to be shown in the listbox (this is
    // mandatory)
    virtual wxString OnGetItem(size_t n) const;

    // change the appearance by overriding these functions (this is optional)
    virtual void OnDrawBackground(wxDC &dc, const wxRect &rect, size_t n) const;

    bool isDark = false;

  public:
    wxDECLARE_NO_COPY_CLASS(ResultListBox);
    wxDECLARE_DYNAMIC_CLASS(ResultListBox);
};

wxIMPLEMENT_DYNAMIC_CLASS(ResultListBox, wxHtmlListBox);

ResultListBox::ResultListBox(wxWindow *parent, bool isDark, const wxWindowID id,
                             const wxPoint &pos, const wxSize &size)
    : wxHtmlListBox(parent, id, pos, size, wxBORDER_NONE) {
    this->isDark = isDark;
    SetMargins(5, 5);
    Refresh();
}

void ResultListBox::SetDark(bool dark) {
    isDark = dark;
    RefreshAll();
}

void ResultListBox::OnDrawBackground(wxDC &dc, const wxRect &rect,
                                     size_t n) const {
    if (IsSelected(n)) {
        dc.SetBrush(wxBrush(isDark ? SELECTION_DARK_BG : SELECTION_LIGHT_BG));
    } else {
        dc.SetBrush(
            wxBrush(isDark ? DARK_BG
                           : wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW)));
    }
    dc.SetPen(*wxTRANSPARENT_PEN);
    dc.DrawRectangle(rect.x, rect.y, rect.width, rect.height);
}

// Helper function to escape HTML special characters
wxString EscapeHtml(const wxString &str) {
    wxString escaped = str;
    escaped.Replace(wxT("&"), wxT("&amp;"));  // Must be first to avoid double-escaping
    escaped.Replace(wxT("<"), wxT("&lt;"));
    escaped.Replace(wxT(">"), wxT("&gt;"));
    escaped.Replace(wxT("\""), wxT("&quot;"));
    return escaped;
}

wxString ResultListBox::OnGetItem(size_t n) const {
    wxString textColor = isDark ? "white" : "";
    // Set the cell background explicitly: the HTML renderer paints its own
    // white background before the theme is applied (startup flash), and for
    // the selected row it must match the highlight colour or it overpaints the
    // centre. Derived from the shared palette so the two can't drift apart.
    wxColour bg =
        isDark ? DARK_BG : wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW);
    if (IsSelected(n)) {
        bg = isDark ? SELECTION_DARK_BG : SELECTION_LIGHT_BG;
    }
    wxString bgColor = bg.GetAsString(wxC2S_HTML_SYNTAX);
    // Shortcut hint: lighter on dark, upstream grey on light.
    wxString shortcutColor = isDark ? "#9aa6ad" : "#636e72";
    wxString shortcut =
        (n < 8) ? wxString::Format(wxT("Alt+%i"), (int)n + 1) : " ";

    // Escape HTML special characters in label and trigger to prevent them
    // from being interpreted as HTML tags (fixes issue #974)
    wxString escapedLabel = EscapeHtml(wxItems[n]);
    wxString escapedTrigger = EscapeHtml(wxTriggers[n]);

    wxString result = wxString::Format(
        wxT("<font color='%s'><table width='100%%' bgcolor='%s'><tr><td>%s</"
            "td><td "
            "align='right'><b>%s</b> <font color='%s'> "
            "%s</font></td></tr></table></font>"),
        textColor, bgColor, escapedLabel, escapedTrigger, shortcutColor,
        shortcut);

    return result;
}

class SearchFrame : public wxFrame {
  public:
    SearchFrame(const wxString &title, const wxPoint &pos, const wxSize &size);

    wxPanel *panel = nullptr;
    wxTextCtrl *searchBar = nullptr;
    wxStaticBitmap *iconPanel = nullptr;
    wxStaticText *helpText = nullptr;
    ResultListBox *resultBox = nullptr;
    void SetItems(SearchItem *items, int itemSize);

  private:
    void OnCharEvent(wxKeyEvent &event);
    void OnQueryChange(wxCommandEvent &event);
    void OnItemClickEvent(wxCommandEvent &event);
    void OnActivate(wxActivateEvent &event);

    // Paint the background on erase so the default white brush never flashes.
    void OnEraseBackground(wxEraseEvent &event);

    // Hide before destroy so the final white repaint is never seen.
    void OnClose(wxCloseEvent &event);

#ifdef __WXMSW__
    // Re-theme live when the system colour scheme changes.
    WXLRESULT MSWWindowProc(WXUINT message, WXWPARAM wParam, WXLPARAM lParam);
    void ApplyTheme();
#endif
    void SetTheme(bool dark);

    // Mouse events
    void OnMouseCaptureLost(wxMouseCaptureLostEvent &event);
    void OnMouseLeave(wxMouseEvent &event);
    void OnMouseMove(wxMouseEvent &event);
    void OnMouseLUp(wxMouseEvent &event);
    void OnMouseLDown(wxMouseEvent &event);
    wxPoint mLastPt;

    // Current theme; used by OnEraseBackground to avoid a white flash.
    bool m_dark = false;

    // Selection
    void SelectNext();
    void SelectPrevious();
    void Submit();
};

bool SearchApp::OnInit() {
#ifdef __WXMSW__
    // Opt into Windows immersive dark mode before any window is created.
    enableAppDarkMode();
#endif

    SearchFrame *frame =
        new SearchFrame(wxString::FromUTF8(searchMetadata->windowTitle),
                        wxPoint(50, 50), wxSize(450, 340));
    frame->Show(true);
    // Pre-paint synchronously so the window is fully drawn (dark) before the
    // compositor presents it; otherwise the first frame can flash white.
    frame->Update();
    SetupWindowStyle(frame);
    Activate(frame);
    return true;
}
SearchFrame::SearchFrame(const wxString &title, const wxPoint &pos,
                         const wxSize &size)
    : wxFrame(NULL, wxID_ANY, title, pos, size, DEFAULT_STYLE) {
    wxInitAllImageHandlers();

#ifdef __WXMSW__
    // Read the registry directly; matches what PowerToys Light Switch flips.
    bool isDark = isSystemDark();
#elif wxCHECK_VERSION(3, 1, 3)
    bool isDark = wxSystemSettings::GetAppearance().IsDark();
#else
    // Workaround needed for previous versions of wxWidgets
    const wxColour bg = wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW);
    const wxColour fg = wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT);
    unsigned int bgSum = (bg.Red() + bg.Blue() + bg.Green());
    unsigned int fgSum = (fg.Red() + fg.Blue() + fg.Green());
    bool isDark = fgSum > bgSum;
#endif

    m_dark = isDark;

    panel = new wxPanel(this, wxID_ANY);
    wxBoxSizer *vbox = new wxBoxSizer(wxVERTICAL);
    panel->SetSizer(vbox);

    wxBoxSizer *topBox = new wxBoxSizer(wxHORIZONTAL);

    int iconId = NewControlId();
    iconPanel = nullptr;
    if (searchMetadata->iconPath) {
        wxString iconPath = wxString::FromUTF8(searchMetadata->iconPath);
        if (wxFileExists(iconPath)) {
            wxBitmap bitmap = wxBitmap(iconPath, wxBITMAP_TYPE_PNG);
            if (bitmap.IsOk()) {
                wxImage image = bitmap.ConvertToImage();
                image.Rescale(32, 32, wxIMAGE_QUALITY_HIGH);
                wxBitmap resizedBitmap = wxBitmap(image, -1);
                iconPanel =
                    new wxStaticBitmap(panel, iconId, resizedBitmap,
                                       wxDefaultPosition, wxSize(32, 32));
                topBox->Add(iconPanel, 0, wxEXPAND | wxLEFT | wxUP | wxDOWN,
                            10);
            }
        }
    }

    int textId = NewControlId();
    searchBar =
        new wxTextCtrl(panel, textId, "", wxDefaultPosition, wxDefaultSize);
    wxFont font = searchBar->GetFont();
    font.SetPointSize(SEARCH_BAR_FONT_SIZE);
    searchBar->SetFont(font);
    topBox->Add(searchBar, 1, wxEXPAND | wxALL, 10);

    vbox->Add(topBox, 1, wxEXPAND);

    if (searchMetadata->hintText) {
        helpText = new wxStaticText(
            panel, wxID_ANY, wxString::FromUTF8(searchMetadata->hintText));
        vbox->Add(helpText, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, 10);
        wxFont helpFont = helpText->GetFont();
        helpFont.SetPointSize(HELP_TEXT_FONT_SIZE);
        helpText->SetFont(helpFont);
    }

    wxArrayString choices;
    int resultId = NewControlId();
    resultBox = new ResultListBox(panel, isDark, resultId, wxDefaultPosition,
                                  wxSize(MIN_WIDTH, MIN_HEIGHT));
    vbox->Add(resultBox, 5, wxEXPAND | wxALL, 0);

    Bind(wxEVT_CHAR_HOOK, &SearchFrame::OnCharEvent, this, wxID_ANY);
    searchBar->Bind(wxEVT_CHAR, &SearchFrame::OnCharEvent, this, wxID_ANY);
    Bind(wxEVT_TEXT, &SearchFrame::OnQueryChange, this, textId);
    Bind(wxEVT_LISTBOX_DCLICK, &SearchFrame::OnItemClickEvent, this, resultId);
    Bind(wxEVT_ACTIVATE, &SearchFrame::OnActivate, this, wxID_ANY);

    // Paint on erase so the white brush never flashes (worst on close and first
    // show). NOTE: a wxHtmlListBox owns an internal viewport that receives
    // WM_ERASEBKGND and does not bubble it up, so we must bind to the viewport
    // itself.
    Bind(wxEVT_ERASE_BACKGROUND, &SearchFrame::OnEraseBackground, this);
    panel->Bind(wxEVT_ERASE_BACKGROUND, &SearchFrame::OnEraseBackground, this);
    resultBox->Bind(wxEVT_ERASE_BACKGROUND, &SearchFrame::OnEraseBackground,
                    this);
    if (wxWindow *viewport = resultBox->GetTargetWindow()) {
        viewport->Bind(wxEVT_ERASE_BACKGROUND, &SearchFrame::OnEraseBackground,
                       this);
    }

    // Hide before destroy so the final white repaint on close is never seen.
    Bind(wxEVT_CLOSE_WINDOW, &SearchFrame::OnClose, this);

#ifdef __WXMSW__
    // Apply dark mode up-front; live changes handled in MSWWindowProc.
    applyDarkModeToWindow(GetHandle(), isDark);
    SetTheme(isDark);
#endif

    // Events to handle the mouse drag
    if (iconPanel) {
        iconPanel->Bind(wxEVT_LEFT_UP, &SearchFrame::OnMouseLUp, this);
        iconPanel->Bind(wxEVT_LEFT_DOWN, &SearchFrame::OnMouseLDown, this);
        Bind(wxEVT_MOTION, &SearchFrame::OnMouseMove, this);
        Bind(wxEVT_LEFT_UP, &SearchFrame::OnMouseLUp, this);
        Bind(wxEVT_LEFT_DOWN, &SearchFrame::OnMouseLDown, this);
        Bind(wxEVT_MOUSE_CAPTURE_LOST, &SearchFrame::OnMouseCaptureLost, this);
        Bind(wxEVT_LEAVE_WINDOW, &SearchFrame::OnMouseLeave, this);
    }

    wxSize bestSize = panel->GetBestSize();
    this->SetClientSize(bestSize);
    this->SetSizeHints(MIN_WIDTH, MIN_HEIGHT);
    this->CentreOnScreen();

    // Trigger the first data update
    queryCallback("", (void *)this, data);
}

void SearchFrame::OnCharEvent(wxKeyEvent &event) {
    if (event.GetKeyCode() == WXK_ESCAPE) {
        Close(true);
    } else if (event.GetKeyCode() == WXK_TAB) {
        if (wxGetKeyState(WXK_SHIFT)) {
            SelectPrevious();
        } else {
            SelectNext();
        }
    } else if (event.GetUnicodeKey() >= '1' &&
               event.GetUnicodeKey() <= '9') { // Alt + num shortcut
        int index = event.GetUnicodeKey() - '1';
        if (wxGetKeyState(WXK_ALT)) {
            if (resultBox->GetItemCount() > index) {
                resultBox->SetSelection(index);
                Submit();
            }
        } else {
            event.Skip();
        }
    } else if (event.GetKeyCode() == WXK_DOWN) {
        SelectNext();
    } else if (event.GetKeyCode() == WXK_UP) {
        SelectPrevious();
    } else if (event.GetKeyCode() == 'N' && event.RawControlDown()) {
        SelectNext();
    } else if (event.GetKeyCode() == 'P' && event.RawControlDown()) {
        SelectPrevious();
    } else if (event.GetKeyCode() == WXK_RETURN || event.GetKeyCode() == WXK_NUMPAD_ENTER) {
#ifdef __WXOSX__
        // Don't consume Enter while an IME composition is in progress —
        // the user is committing a candidate (e.g. selecting a Chinese
        // character), not confirming the highlighted match. Let the event
        // propagate so the IME can finalize its selection.
        if (IsImeComposingInKeyWindow()) {
            event.Skip();
            return;
        }
#endif
        Submit();
    } else {
        event.Skip();
    }
}

void SearchFrame::OnQueryChange(wxCommandEvent &event) {
    if (helpText != nullptr) {
        helpText->Destroy();
        panel->Layout();
        helpText = nullptr;
    }

    wxString queryString = searchBar->GetValue();
    const char *query = queryString.ToUTF8();
    queryCallback(query, (void *)this, data);
}

void SearchFrame::OnItemClickEvent(wxCommandEvent &event) {
    resultBox->SetSelection(event.GetInt());
    Submit();
}

void SearchFrame::OnActivate(wxActivateEvent &event) {
    if (!event.GetActive()) {
        Close(true);
    }
    event.Skip();
}

void SearchFrame::SetTheme(bool dark) {
    // wxWidgets 3.1.5 does not auto-darken the client area, so we set the
    // colours explicitly on the frame and every child control. The result
    // list box additionally needs its internal isDark flag (it draws its own
    // HTML text + selection colours).
    m_dark = dark;
#ifdef __WXMSW__
    applyThemeColors(this, dark);
#endif
    if (resultBox != nullptr) {
        resultBox->SetDark(dark);
    }
}

#ifdef __WXMSW__
void SearchFrame::ApplyTheme() {
    bool dark = isSystemDark();
    applyDarkModeToWindow(GetHandle(), dark);
    SetTheme(dark);
}
#endif

void SearchFrame::OnEraseBackground(wxEraseEvent &event) {
    // Erase with the theme-aware background colour instead of the default white
    // brush. We deliberately do NOT call event.Skip(), so the native white
    // erase never runs. This removes the white flash seen on window close (when
    // the panel is destroyed and the frame's client area gets a final
    // WM_ERASEBKGND) and on first show (the list box viewport erases before its
    // first paint).
    //
    // We use m_dark rather than the target window's GetBackgroundColour()
    // because wxHtmlListBox's internal viewport does not reliably surface the
    // colour we set on the list box, so reading it back could still be white.
    wxDC *dc = event.GetDC();
    if (dc != nullptr) {
        wxColour bg = m_dark ? DARK_BG
                             : wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW);
        dc->SetBackground(wxBrush(bg));
        dc->Clear();
    }
}

void SearchFrame::OnClose(wxCloseEvent &event) {
    // Hide first: when the window is destroyed, native controls (the search
    // box, the icon) repaint white for their final frame under dark mode. As
    // long as the window is already hidden, that flash is never shown.
    Hide();
    event.Skip();
}

#ifdef __WXMSW__
WXLRESULT SearchFrame::MSWWindowProc(WXUINT message, WXWPARAM wParam,
                                     WXLPARAM lParam) {
    if (isColorSchemeChange(message, lParam)) {
        ApplyTheme();
    }
    return wxFrame::MSWWindowProc(message, wParam, lParam);
}
#endif

void SearchFrame::OnMouseMove(wxMouseEvent &event) {
    if (event.LeftIsDown() && event.Dragging()) {
        wxPoint pt = event.GetPosition();
        wxPoint wndLeftTopPt = GetPosition();
        int distanceX = pt.x - mLastPt.x;
        int distanceY = pt.y - mLastPt.y;

        wxPoint desPt;
        desPt.x = distanceX + wndLeftTopPt.x - 24;
        desPt.y = distanceY + wndLeftTopPt.y - 24;
        this->Move(desPt);
    }

    if (event.LeftDown()) {
        this->CaptureMouse();
        mLastPt = event.GetPosition();
    }
}

void SearchFrame::OnMouseLeave(wxMouseEvent &event) {
    if (event.LeftIsDown() && event.Dragging()) {
        wxPoint pt = event.GetPosition();
        wxPoint wndLeftTopPt = GetPosition();
        int distanceX = pt.x - mLastPt.x;
        int distanceY = pt.y - mLastPt.y;

        wxPoint desPt;
        desPt.x = distanceX + wndLeftTopPt.x - 24;
        desPt.y = distanceY + wndLeftTopPt.y - 24;
        this->Move(desPt);
    }
}

void SearchFrame::OnMouseLDown(wxMouseEvent &event) {
    if (!HasCapture())
        this->CaptureMouse();
}

void SearchFrame::OnMouseLUp(wxMouseEvent &event) {
    if (HasCapture())
        ReleaseMouse();
}

void SearchFrame::OnMouseCaptureLost(wxMouseCaptureLostEvent &event) {
    if (HasCapture())
        ReleaseMouse();
}

void SearchFrame::SetItems(SearchItem *items, int itemSize) {
    wxItems.Clear();
    wxIds.Clear();
    wxTriggers.Clear();

    for (int i = 0; i < itemSize; i++) {
        wxString item = wxString::FromUTF8(items[i].label);
        wxItems.Add(item);

        wxString id = wxString::FromUTF8(items[i].id);
        wxIds.Add(id);

        wxString trigger = wxString::FromUTF8(items[i].trigger);
        wxTriggers.Add(trigger);
    }

    resultBox->SetItemCount(itemSize);

    if (itemSize > 0) {
        resultBox->SetSelection(0);
    }
    resultBox->RefreshAll();
    resultBox->Refresh();
}

void SearchFrame::SelectNext() {
    if (resultBox->GetItemCount() > 0 &&
        resultBox->GetSelection() != wxNOT_FOUND) {
        int newSelected = 0;
        if (resultBox->GetSelection() < (resultBox->GetItemCount() - 1)) {
            newSelected = resultBox->GetSelection() + 1;
        }

        resultBox->SetSelection(newSelected);
    }
}

void SearchFrame::SelectPrevious() {
    if (resultBox->GetItemCount() > 0 &&
        resultBox->GetSelection() != wxNOT_FOUND) {
        int newSelected = resultBox->GetItemCount() - 1;
        if (resultBox->GetSelection() > 0) {
            newSelected = resultBox->GetSelection() - 1;
        }

        resultBox->SetSelection(newSelected);
    }
}

void SearchFrame::Submit() {
    if (resultBox->GetItemCount() > 0 &&
        resultBox->GetSelection() != wxNOT_FOUND) {
        long index = resultBox->GetSelection();
        wxString id = wxIds[index];
        if (resultCallback) {
            resultCallback(id.ToUTF8(), resultData);
        }

        Close(true);
    }
}

extern "C" void interop_show_search(SearchMetadata *_metadata,
                                    QueryCallback _queryCallback, void *_data,
                                    ResultCallback _resultCallback,
                                    void *_resultData) {
// Setup high DPI support on Windows
#ifdef __WXMSW__
    SetProcessDPIAware();
#endif

    searchMetadata = _metadata;
    queryCallback = _queryCallback;
    resultCallback = _resultCallback;
    data = _data;
    resultData = _resultData;

    wxApp::SetInstance(new SearchApp());
    int argc = 0;
    wxEntry(argc, (char **)nullptr);
}

extern "C" void update_items(void *app, SearchItem *items, int itemSize) {
    SearchFrame *frame = (SearchFrame *)app;
    frame->SetItems(items, itemSize);
}
