/* * Copyright (C) 2016-2019 Mohammed Boujemaoui <mohabouje@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#ifndef WINTOASTLIB_H
#define WINTOASTLIB_H
#include <Windows.h>
#include <sdkddkver.h>
#include <WinUser.h>
#include <ShObjIdl.h>
#include <wrl/implements.h>
#include <wrl/event.h>
#include <windows.ui.notifications.h>
#include <strsafe.h>
#include <Psapi.h>
#include <ShlObj.h>
#include <roapi.h>
#include <propvarutil.h>
#include <functiondiscoverykeys.h>
#include <iostream>
#include <winstring.h>
#include <string.h>
#include <vector>
#include <map>
using namespace Microsoft::WRL;
using namespace ABI::Windows::Data::Xml::Dom;
using namespace ABI::Windows::Foundation;
using namespace ABI::Windows::UI::Notifications;
using namespace Windows::Foundation;


namespace WinToastLib {

    class IWinToastHandler {
    public:
        enum WinToastDismissalReason {
            UserCanceled = ToastDismissalReason::ToastDismissalReason_UserCanceled,
            ApplicationHidden = ToastDismissalReason::ToastDismissalReason_ApplicationHidden,
            TimedOut = ToastDismissalReason::ToastDismissalReason_TimedOut
        };
        virtual ~IWinToastHandler() = default;
        virtual void toastActivated() const = 0;
        virtual void toastActivated(int actionIndex) const = 0;
        virtual void toastDismissed(WinToastDismissalReason state) const = 0;
        virtual void toastFailed() const = 0;
    };

    class WinToastTemplate {
    public:
        enum Duration { System, Short, Long };
        enum AudioOption { Default = 0, Silent, Loop };
        enum TextField { FirstLine = 0, SecondLine, ThirdLine };
        enum WinToastTemplateType {
            ImageAndText01 = ToastTemplateType::ToastTemplateType_ToastImageAndText01,
            ImageAndText02 = ToastTemplateType::ToastTemplateType_ToastImageAndText02,
            ImageAndText03 = ToastTemplateType::ToastTemplateType_ToastImageAndText03,
            ImageAndText04 = ToastTemplateType::ToastTemplateType_ToastImageAndText04,
            Text01 = ToastTemplateType::ToastTemplateType_ToastText01,
            Text02 = ToastTemplateType::ToastTemplateType_ToastText02,
            Text03 = ToastTemplateType::ToastTemplateType_ToastText03,
            Text04 = ToastTemplateType::ToastTemplateType_ToastText04,
        };

        enum AudioSystemFile {
            DefaultSound,
            IM, 
            Mail,
            Reminder, 
            SMS, 
            Alarm,
            Alarm2,
            Alarm3,
            Alarm4,
            Alarm5,
            Alarm6,
            Alarm7,
            Alarm8,
            Alarm9,
            Alarm10,
            Call,
            Call1,
            Call2,
            Call3,
            Call4,
            Call5,
            Call6,
            Call7,
            Call8,
            Call9,
            Call10,
        };


        WinToastTemplate(_In_ WinToastTemplateType type = WinToastTemplateType::ImageAndText02);
        ~WinToastTemplate();

        void setFirstLine(_In_ const std::wstring& text);
        void setSecondLine(_In_ const std::wstring& text);
        void setThirdLine(_In_ const std::wstring& text);
        void setTextField(_In_ const std::wstring& txt, _In_ TextField pos);
        void setAttributionText(_In_ const std::wstring & attributionText);
        void setImagePath(_In_ const std::wstring& imgPath);
        void setAudioPath(_In_ WinToastTemplate::AudioSystemFile audio);
        void setAudioPath(_In_ const std::wstring& audioPath);
        void setAudioOption(_In_ WinToastTemplate::AudioOption audioOption);
        void setDuration(_In_ Duration duration);
        void setExpiration(_In_ INT64 millisecondsFromNow);
        void addAction(_In_ const std::wstring& label);

        std::size_t textFieldsCount() const;
        std::size_t actionsCount() const;
        bool hasImage() const;
        const std::vector<std::wstring>& textFields() const;
        const std::wstring& textField(_In_ TextField pos) const;
        const std::wstring& actionLabel(_In_ std::size_t pos) const;
        const std::wstring& imagePath() const;
        const std::wstring& audioPath() const;
        const std::wstring& attributionText() const;
        INT64 expiration() const;
        WinToastTemplateType type() const;
        WinToastTemplate::AudioOption audioOption() const;
        Duration duration() const;
    private:
        std::vector<std::wstring>			_textFields{};
        std::vector<std::wstring>           _actions{};
        std::wstring                        _imagePath{};
        std::wstring                        _audioPath{};
        std::wstring                        _attributionText{};
        INT64                               _expiration{0};
        AudioOption                         _audioOption{WinToastTemplate::AudioOption::Default};
        WinToastTemplateType                _type{WinToastTemplateType::Text01};
        Duration                            _duration{Duration::System};
    };

    class WinToast {
    public:
        enum WinToastError {
            NoError = 0,
            NotInitialized,
            SystemNotSupported,
            ShellLinkNotCreated,
            InvalidAppUserModelID,
            InvalidParameters,
            InvalidHandler,
            NotDisplayed,
            UnknownError
        };

        enum ShortcutResult {
            SHORTCUT_UNCHANGED = 0,
            SHORTCUT_WAS_CHANGED = 1,
            SHORTCUT_WAS_CREATED = 2,

            SHORTCUT_MISSING_PARAMETERS = -1,
            SHORTCUT_INCOMPATIBLE_OS = -2,
            SHORTCUT_COM_INIT_FAILURE = -3,
            SHORTCUT_CREATE_FAILED = -4
        };

        WinToast(void);
        virtual ~WinToast();
        static WinToast* instance();
        static bool isCompatible();
		static bool	isSupportingModernFeatures();
		static std::wstring configureAUMI(_In_ const std::wstring& companyName,
                                          _In_ const std::wstring& productName,
                                          _In_ const std::wstring& subProduct = std::wstring(),
                                          _In_ const std::wstring& versionInformation = std::wstring());
        static const std::wstring& strerror(_In_ WinToastError error);
        virtual bool initialize(_Out_ WinToastError* error = nullptr);
        virtual bool isInitialized() const;
        virtual bool hideToast(_In_ INT64 id);
        virtual INT64 showToast(_In_ const WinToastTemplate& toast, _In_ IWinToastHandler* handler, _Out_ WinToastError* error = nullptr);
        virtual void clear();
        virtual enum ShortcutResult createShortcut();

        const std::wstring& appName() const;
        const std::wstring& appUserModelId() const;
        void setAppUserModelId(_In_ const std::wstring& aumi);
        void setAppName(_In_ const std::wstring& appName);

    protected:
        bool											_isInitialized{false};
        bool                                            _hasCoInitialized{false};
        std::wstring                                    _appName{};
        std::wstring                                    _aumi{};
        std::map<INT64, ComPtr<IToastNotification>>     _buffer{};

        HRESULT validateShellLinkHelper(_Out_ bool& wasChanged);
        HRESULT createShellLinkHelper();
        HRESULT setImageFieldHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& path);
        HRESULT setAudioFieldHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& path, _In_opt_ WinToastTemplate::AudioOption option = WinToastTemplate::AudioOption::Default);
        HRESULT setTextFieldHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& text, _In_ UINT32 pos);
        HRESULT setAttributionTextFieldHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& text);
        HRESULT addActionHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& action, _In_ const std::wstring& arguments);
        HRESULT addDurationHelper(_In_ IXmlDocument *xml, _In_ const std::wstring& duration);
        ComPtr<IToastNotifier> notifier(_In_ bool* succeded) const;
        void setError(_Out_ WinToastError* error, _In_ WinToastError value);
    };
}
#endif // WINTOASTLIB_H
