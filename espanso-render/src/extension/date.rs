/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use chrono::{DateTime, Duration, Local, Locale};

use crate::{Extension, ExtensionOutput, ExtensionResult, Number, Params, Value};

pub trait LocaleProvider {
    fn get_system_locale(&self) -> String;
}

pub struct DateExtension<'a> {
    fixed_date: Option<DateTime<Local>>,
    locale_provider: &'a dyn LocaleProvider,
}

#[allow(clippy::new_without_default)]
impl<'a> DateExtension<'a> {
    pub fn new(locale_provider: &'a dyn LocaleProvider) -> Self {
        Self {
            fixed_date: None,
            locale_provider,
        }
    }
}

impl<'a> Extension for DateExtension<'a> {
    fn name(&self) -> &str {
        "date"
    }

    fn calculate(
        &self,
        _: &crate::Context,
        _: &crate::Scope,
        params: &Params,
    ) -> crate::ExtensionResult {
        let mut now = self.get_date();

        // Compute the given offset
        let offset = params.get("offset");
        if let Some(Value::Number(Number::Integer(offset))) = offset {
            let offset = Duration::seconds(*offset);
            now = now + offset;
        }

        let format = params.get("format");
        let locale = params
            .get("locale")
            .and_then(|val| val.as_string())
            .map_or_else(|| self.locale_provider.get_system_locale(), String::from);

        let date = if let Some(Value::String(format)) = format {
            DateExtension::format_date_with_locale_string(now, format, &locale)
        } else {
            now.to_rfc2822()
        };

        ExtensionResult::Success(ExtensionOutput::Single(date))
    }
}

impl<'a> DateExtension<'a> {
    fn get_date(&self) -> DateTime<Local> {
        if let Some(fixed_date) = self.fixed_date {
            fixed_date
        } else {
            Local::now()
        }
    }

    fn format_date_with_locale(date: DateTime<Local>, format: &str, locale: Locale) -> String {
        date.format_localized(format, locale).to_string()
    }

    fn format_date_with_locale_string(
        date: DateTime<Local>,
        format: &str,
        locale_str: &str,
    ) -> String {
        let locale = convert_locale_string_to_locale(locale_str).unwrap_or(Locale::en_US);
        Self::format_date_with_locale(date, format, locale)
    }
}

fn convert_locale_string_to_locale(locale_str: &str) -> Option<Locale> {
    match locale_str {
        "aa-DJ" => Some(Locale::aa_DJ),
        "aa-ER" => Some(Locale::aa_ER),
        "aa-ET" => Some(Locale::aa_ET),
        "af-ZA" => Some(Locale::af_ZA),
        "agr-PE" => Some(Locale::agr_PE),
        "ak-GH" => Some(Locale::ak_GH),
        "am-ET" => Some(Locale::am_ET),
        "an-ES" => Some(Locale::an_ES),
        "anp-IN" => Some(Locale::anp_IN),
        "ar-AE" => Some(Locale::ar_AE),
        "ar-BH" => Some(Locale::ar_BH),
        "ar-DZ" => Some(Locale::ar_DZ),
        "ar-EG" => Some(Locale::ar_EG),
        "ar-IN" => Some(Locale::ar_IN),
        "ar-IQ" => Some(Locale::ar_IQ),
        "ar-JO" => Some(Locale::ar_JO),
        "ar-KW" => Some(Locale::ar_KW),
        "ar-LB" => Some(Locale::ar_LB),
        "ar-LY" => Some(Locale::ar_LY),
        "ar-MA" => Some(Locale::ar_MA),
        "ar-OM" => Some(Locale::ar_OM),
        "ar-QA" => Some(Locale::ar_QA),
        "ar-SA" => Some(Locale::ar_SA),
        "ar-SD" => Some(Locale::ar_SD),
        "ar-SS" => Some(Locale::ar_SS),
        "ar-SY" => Some(Locale::ar_SY),
        "ar-TN" => Some(Locale::ar_TN),
        "ar-YE" => Some(Locale::ar_YE),
        "as-IN" => Some(Locale::as_IN),
        "ast-ES" => Some(Locale::ast_ES),
        "ayc-PE" => Some(Locale::ayc_PE),
        "az-AZ" => Some(Locale::az_AZ),
        "az-IR" => Some(Locale::az_IR),
        "be-BY" => Some(Locale::be_BY),
        "bem-ZM" => Some(Locale::bem_ZM),
        "ber-DZ" => Some(Locale::ber_DZ),
        "ber-MA" => Some(Locale::ber_MA),
        "bg-BG" => Some(Locale::bg_BG),
        "bhb-IN" => Some(Locale::bhb_IN),
        "bho-IN" => Some(Locale::bho_IN),
        "bho-NP" => Some(Locale::bho_NP),
        "bi-VU" => Some(Locale::bi_VU),
        "bn-BD" => Some(Locale::bn_BD),
        "bn-IN" => Some(Locale::bn_IN),
        "bo-CN" => Some(Locale::bo_CN),
        "bo-IN" => Some(Locale::bo_IN),
        "br-FR" => Some(Locale::br_FR),
        "brx-IN" => Some(Locale::brx_IN),
        "bs-BA" => Some(Locale::bs_BA),
        "byn-ER" => Some(Locale::byn_ER),
        "ca-AD" => Some(Locale::ca_AD),
        "ca-ES" => Some(Locale::ca_ES),
        "ca-FR" => Some(Locale::ca_FR),
        "ca-IT" => Some(Locale::ca_IT),
        "ce-RU" => Some(Locale::ce_RU),
        "chr-US" => Some(Locale::chr_US),
        "cmn-TW" => Some(Locale::cmn_TW),
        "crh-UA" => Some(Locale::crh_UA),
        "cs-CZ" => Some(Locale::cs_CZ),
        "csb-PL" => Some(Locale::csb_PL),
        "cv-RU" => Some(Locale::cv_RU),
        "cy-GB" => Some(Locale::cy_GB),
        "da-DK" => Some(Locale::da_DK),
        "de-AT" => Some(Locale::de_AT),
        "de-BE" => Some(Locale::de_BE),
        "de-CH" => Some(Locale::de_CH),
        "de-DE" => Some(Locale::de_DE),
        "de-IT" => Some(Locale::de_IT),
        "de-LI" => Some(Locale::de_LI),
        "de-LU" => Some(Locale::de_LU),
        "doi-IN" => Some(Locale::doi_IN),
        "dsb-DE" => Some(Locale::dsb_DE),
        "dv-MV" => Some(Locale::dv_MV),
        "dz-BT" => Some(Locale::dz_BT),
        "el-CY" => Some(Locale::el_CY),
        "el-GR" => Some(Locale::el_GR),
        "en-AG" => Some(Locale::en_AG),
        "en-AU" => Some(Locale::en_AU),
        "en-BW" => Some(Locale::en_BW),
        "en-CA" => Some(Locale::en_CA),
        "en-DK" => Some(Locale::en_DK),
        "en-GB" => Some(Locale::en_GB),
        "en-HK" => Some(Locale::en_HK),
        "en-IE" => Some(Locale::en_IE),
        "en-IL" => Some(Locale::en_IL),
        "en-IN" => Some(Locale::en_IN),
        "en-NG" => Some(Locale::en_NG),
        "en-NZ" => Some(Locale::en_NZ),
        "en-PH" => Some(Locale::en_PH),
        "en-SC" => Some(Locale::en_SC),
        "en-SG" => Some(Locale::en_SG),
        "en-US" => Some(Locale::en_US),
        "en-ZA" => Some(Locale::en_ZA),
        "en-ZM" => Some(Locale::en_ZM),
        "en-ZW" => Some(Locale::en_ZW),
        "eo" => Some(Locale::eo),
        "es-AR" => Some(Locale::es_AR),
        "es-BO" => Some(Locale::es_BO),
        "es-CL" => Some(Locale::es_CL),
        "es-CO" => Some(Locale::es_CO),
        "es-CR" => Some(Locale::es_CR),
        "es-CU" => Some(Locale::es_CU),
        "es-DO" => Some(Locale::es_DO),
        "es-EC" => Some(Locale::es_EC),
        "es-ES" => Some(Locale::es_ES),
        "es-GT" => Some(Locale::es_GT),
        "es-HN" => Some(Locale::es_HN),
        "es-MX" => Some(Locale::es_MX),
        "es-NI" => Some(Locale::es_NI),
        "es-PA" => Some(Locale::es_PA),
        "es-PE" => Some(Locale::es_PE),
        "es-PR" => Some(Locale::es_PR),
        "es-PY" => Some(Locale::es_PY),
        "es-SV" => Some(Locale::es_SV),
        "es-US" => Some(Locale::es_US),
        "es-UY" => Some(Locale::es_UY),
        "es-VE" => Some(Locale::es_VE),
        "et-EE" => Some(Locale::et_EE),
        "eu-ES" => Some(Locale::eu_ES),
        "fa-IR" => Some(Locale::fa_IR),
        "ff-SN" => Some(Locale::ff_SN),
        "fi-FI" => Some(Locale::fi_FI),
        "fil-PH" => Some(Locale::fil_PH),
        "fo-FO" => Some(Locale::fo_FO),
        "fr-BE" => Some(Locale::fr_BE),
        "fr-CA" => Some(Locale::fr_CA),
        "fr-CH" => Some(Locale::fr_CH),
        "fr-FR" => Some(Locale::fr_FR),
        "fr-LU" => Some(Locale::fr_LU),
        "fur-IT" => Some(Locale::fur_IT),
        "fy-DE" => Some(Locale::fy_DE),
        "fy-NL" => Some(Locale::fy_NL),
        "ga-IE" => Some(Locale::ga_IE),
        "gd-GB" => Some(Locale::gd_GB),
        "gez-ER" => Some(Locale::gez_ER),
        "gez-ET" => Some(Locale::gez_ET),
        "gl-ES" => Some(Locale::gl_ES),
        "gu-IN" => Some(Locale::gu_IN),
        "gv-GB" => Some(Locale::gv_GB),
        "ha-NG" => Some(Locale::ha_NG),
        "hak-TW" => Some(Locale::hak_TW),
        "he-IL" => Some(Locale::he_IL),
        "hi-IN" => Some(Locale::hi_IN),
        "hif-FJ" => Some(Locale::hif_FJ),
        "hne-IN" => Some(Locale::hne_IN),
        "hr-HR" => Some(Locale::hr_HR),
        "hsb-DE" => Some(Locale::hsb_DE),
        "ht-HT" => Some(Locale::ht_HT),
        "hu-HU" => Some(Locale::hu_HU),
        "hy-AM" => Some(Locale::hy_AM),
        "ia-FR" => Some(Locale::ia_FR),
        "id-ID" => Some(Locale::id_ID),
        "ig-NG" => Some(Locale::ig_NG),
        "ik-CA" => Some(Locale::ik_CA),
        "is-IS" => Some(Locale::is_IS),
        "it-CH" => Some(Locale::it_CH),
        "it-IT" => Some(Locale::it_IT),
        "iu-CA" => Some(Locale::iu_CA),
        "ja-JP" => Some(Locale::ja_JP),
        "ka-GE" => Some(Locale::ka_GE),
        "kab-DZ" => Some(Locale::kab_DZ),
        "kk-KZ" => Some(Locale::kk_KZ),
        "kl-GL" => Some(Locale::kl_GL),
        "km-KH" => Some(Locale::km_KH),
        "kn-IN" => Some(Locale::kn_IN),
        "ko-KR" => Some(Locale::ko_KR),
        "kok-IN" => Some(Locale::kok_IN),
        "ks-IN" => Some(Locale::ks_IN),
        "ku-TR" => Some(Locale::ku_TR),
        "kw-GB" => Some(Locale::kw_GB),
        "ky-KG" => Some(Locale::ky_KG),
        "lb-LU" => Some(Locale::lb_LU),
        "lg-UG" => Some(Locale::lg_UG),
        "li-BE" => Some(Locale::li_BE),
        "li-NL" => Some(Locale::li_NL),
        "lij-IT" => Some(Locale::lij_IT),
        "ln-CD" => Some(Locale::ln_CD),
        "lo-LA" => Some(Locale::lo_LA),
        "lt-LT" => Some(Locale::lt_LT),
        "lv-LV" => Some(Locale::lv_LV),
        "lzh-TW" => Some(Locale::lzh_TW),
        "mag-IN" => Some(Locale::mag_IN),
        "mai-IN" => Some(Locale::mai_IN),
        "mai-NP" => Some(Locale::mai_NP),
        "mfe-MU" => Some(Locale::mfe_MU),
        "mg-MG" => Some(Locale::mg_MG),
        "mhr-RU" => Some(Locale::mhr_RU),
        "mi-NZ" => Some(Locale::mi_NZ),
        "miq-NI" => Some(Locale::miq_NI),
        "mjw-IN" => Some(Locale::mjw_IN),
        "mk-MK" => Some(Locale::mk_MK),
        "ml-IN" => Some(Locale::ml_IN),
        "mn-MN" => Some(Locale::mn_MN),
        "mni-IN" => Some(Locale::mni_IN),
        "mnw-MM" => Some(Locale::mnw_MM),
        "mr-IN" => Some(Locale::mr_IN),
        "ms-MY" => Some(Locale::ms_MY),
        "mt-MT" => Some(Locale::mt_MT),
        "my-MM" => Some(Locale::my_MM),
        "nan-TW" => Some(Locale::nan_TW),
        "nb-NO" => Some(Locale::nb_NO),
        "nds-DE" => Some(Locale::nds_DE),
        "nds-NL" => Some(Locale::nds_NL),
        "ne-NP" => Some(Locale::ne_NP),
        "nhn-MX" => Some(Locale::nhn_MX),
        "niu-NU" => Some(Locale::niu_NU),
        "niu-NZ" => Some(Locale::niu_NZ),
        "nl-AW" => Some(Locale::nl_AW),
        "nl-BE" => Some(Locale::nl_BE),
        "nl-NL" => Some(Locale::nl_NL),
        "nn-NO" => Some(Locale::nn_NO),
        "nr-ZA" => Some(Locale::nr_ZA),
        "nso-ZA" => Some(Locale::nso_ZA),
        "oc-FR" => Some(Locale::oc_FR),
        "om-ET" => Some(Locale::om_ET),
        "om-KE" => Some(Locale::om_KE),
        "or-IN" => Some(Locale::or_IN),
        "os-RU" => Some(Locale::os_RU),
        "pa-IN" => Some(Locale::pa_IN),
        "pa-PK" => Some(Locale::pa_PK),
        "pap-AW" => Some(Locale::pap_AW),
        "pap-CW" => Some(Locale::pap_CW),
        "pl-PL" => Some(Locale::pl_PL),
        "ps-AF" => Some(Locale::ps_AF),
        "pt-BR" => Some(Locale::pt_BR),
        "pt-PT" => Some(Locale::pt_PT),
        "quz-PE" => Some(Locale::quz_PE),
        "raj-IN" => Some(Locale::raj_IN),
        "ro-RO" => Some(Locale::ro_RO),
        "ru-RU" => Some(Locale::ru_RU),
        "ru-UA" => Some(Locale::ru_UA),
        "rw-RW" => Some(Locale::rw_RW),
        "sa-IN" => Some(Locale::sa_IN),
        "sah-RU" => Some(Locale::sah_RU),
        "sat-IN" => Some(Locale::sat_IN),
        "sc-IT" => Some(Locale::sc_IT),
        "sd-IN" => Some(Locale::sd_IN),
        "se-NO" => Some(Locale::se_NO),
        "sgs-LT" => Some(Locale::sgs_LT),
        "shn-MM" => Some(Locale::shn_MM),
        "shs-CA" => Some(Locale::shs_CA),
        "si-LK" => Some(Locale::si_LK),
        "sid-ET" => Some(Locale::sid_ET),
        "sk-SK" => Some(Locale::sk_SK),
        "sl-SI" => Some(Locale::sl_SI),
        "sm-WS" => Some(Locale::sm_WS),
        "so-DJ" => Some(Locale::so_DJ),
        "so-ET" => Some(Locale::so_ET),
        "so-KE" => Some(Locale::so_KE),
        "so-SO" => Some(Locale::so_SO),
        "sq-AL" => Some(Locale::sq_AL),
        "sq-MK" => Some(Locale::sq_MK),
        "sr-ME" => Some(Locale::sr_ME),
        "sr-RS" => Some(Locale::sr_RS),
        "ss-ZA" => Some(Locale::ss_ZA),
        "st-ZA" => Some(Locale::st_ZA),
        "sv-FI" => Some(Locale::sv_FI),
        "sv-SE" => Some(Locale::sv_SE),
        "sw-KE" => Some(Locale::sw_KE),
        "sw-TZ" => Some(Locale::sw_TZ),
        "szl-PL" => Some(Locale::szl_PL),
        "ta-IN" => Some(Locale::ta_IN),
        "ta-LK" => Some(Locale::ta_LK),
        "tcy-IN" => Some(Locale::tcy_IN),
        "te-IN" => Some(Locale::te_IN),
        "tg-TJ" => Some(Locale::tg_TJ),
        "th-TH" => Some(Locale::th_TH),
        "the-NP" => Some(Locale::the_NP),
        "ti-ER" => Some(Locale::ti_ER),
        "ti-ET" => Some(Locale::ti_ET),
        "tig-ER" => Some(Locale::tig_ER),
        "tk-TM" => Some(Locale::tk_TM),
        "tl-PH" => Some(Locale::tl_PH),
        "tn-ZA" => Some(Locale::tn_ZA),
        "to-TO" => Some(Locale::to_TO),
        "tpi-PG" => Some(Locale::tpi_PG),
        "tr-CY" => Some(Locale::tr_CY),
        "tr-TR" => Some(Locale::tr_TR),
        "ts-ZA" => Some(Locale::ts_ZA),
        "tt-RU" => Some(Locale::tt_RU),
        "ug-CN" => Some(Locale::ug_CN),
        "uk-UA" => Some(Locale::uk_UA),
        "unm-US" => Some(Locale::unm_US),
        "ur-IN" => Some(Locale::ur_IN),
        "ur-PK" => Some(Locale::ur_PK),
        "uz-UZ" => Some(Locale::uz_UZ),
        "ve-ZA" => Some(Locale::ve_ZA),
        "vi-VN" => Some(Locale::vi_VN),
        "wa-BE" => Some(Locale::wa_BE),
        "wae-CH" => Some(Locale::wae_CH),
        "wal-ET" => Some(Locale::wal_ET),
        "wo-SN" => Some(Locale::wo_SN),
        "xh-ZA" => Some(Locale::xh_ZA),
        "yi-US" => Some(Locale::yi_US),
        "yo-NG" => Some(Locale::yo_NG),
        "yue-HK" => Some(Locale::yue_HK),
        "yuw-PG" => Some(Locale::yuw_PG),
        "zh-CN" => Some(Locale::zh_CN),
        "zh-HK" => Some(Locale::zh_HK),
        "zh-SG" => Some(Locale::zh_SG),
        "zh-TW" => Some(Locale::zh_TW),
        "zu-ZA" => Some(Locale::zu_ZA),
        _ => None,
    }
}

pub struct DefaultLocaleProvider {}
impl LocaleProvider for DefaultLocaleProvider {
    fn get_system_locale(&self) -> String {
        sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"))
    }
}
#[allow(clippy::new_without_default)]
impl DefaultLocaleProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use chrono::offset::TimeZone;

    struct MockLocaleProvider {
        locale: String,
    }
    impl LocaleProvider for MockLocaleProvider {
        fn get_system_locale(&self) -> String {
            self.locale.clone()
        }
    }
    impl MockLocaleProvider {
        pub fn new() -> Self {
            Self {
                locale: "en-US".to_string(),
            }
        }

        pub fn new_with_locale(locale: String) -> Self {
            Self { locale }
        }
    }

    #[test]
    fn date_formatted_correctly() {
        let locale_provider = MockLocaleProvider::new();
        let mut extension = DateExtension::new(&locale_provider);
        extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let param = vec![("format".to_string(), Value::String("%H:%M:%S".to_string()))]
            .into_iter()
            .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("09:10:11".to_string())
        );
    }

    #[test]
    fn offset_works_correctly() {
        let locale_provider = MockLocaleProvider::new();
        let mut extension = DateExtension::new(&locale_provider);
        extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let param = vec![
            ("format".to_string(), Value::String("%H:%M:%S".to_string())),
            ("offset".to_string(), Value::Number(Number::Integer(3600))),
        ]
        .into_iter()
        .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("10:10:11".to_string())
        );
    }

    #[test]
    fn default_locale_works_correctly() {
        let locale_provider = MockLocaleProvider::new_with_locale("it-IT".to_string());
        let mut extension = DateExtension::new(&locale_provider);
        extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let param = vec![("format".to_string(), Value::String("%A".to_string()))]
            .into_iter()
            .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("martedì".to_string())
        );
    }

    #[test]
    fn invalid_locale_should_default_to_en_us() {
        let locale_provider = MockLocaleProvider::new_with_locale("invalid".to_string());
        let mut extension = DateExtension::new(&locale_provider);
        extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let param = vec![("format".to_string(), Value::String("%A".to_string()))]
            .into_iter()
            .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("Tuesday".to_string())
        );
    }

    #[test]
    fn override_locale() {
        let locale_provider = MockLocaleProvider::new();
        let mut extension = DateExtension::new(&locale_provider);
        extension.fixed_date = Some(Local.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let param = vec![
            ("format".to_string(), Value::String("%A".to_string())),
            ("locale".to_string(), Value::String("it-IT".to_string())),
        ]
        .into_iter()
        .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("martedì".to_string())
        );
    }
}
