use crate::cskkstate::PreCompositionData;
use crate::dictionary::CskkDictionary;
use crate::keyevent::CskkKeyEvent;
use crate::skk_modes::{CommaStyle, CompositionMode, InputMode, PeriodStyle};
use crate::CskkError::Error;
use crate::{
    get_available_rules, skk_context_confirm_candidate_at_rs, skk_context_get_composition_mode_rs,
    skk_context_get_current_candidate_count_rs,
    skk_context_get_current_candidate_cursor_position_rs, skk_context_get_current_candidates_rs,
    skk_context_get_current_to_composite_rs, skk_context_get_input_mode_rs,
    skk_context_poll_output_rs, skk_context_reset_rs, skk_context_select_candidate_at_rs,
    skk_context_set_auto_start_henkan_keywords_rs, skk_context_set_comma_style_rs,
    skk_context_set_dictionaries_rs, skk_context_set_input_mode_rs,
    skk_context_set_period_style_rs, CskkContext, CskkError, CskkStateInfo,
};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::Arc;
use std::{ptr, slice};

pub struct CskkDictionaryFfi {
    dictionary: Arc<CskkDictionary>,
}

#[repr(C)]
pub struct CskkRulesFfi {
    id: *mut c_char,
    name: *mut c_char,
    description: *mut c_char,
}

impl CskkRulesFfi {
    #[allow(clippy::result_unit_err)]
    pub fn new(rust_id: &str, rust_name: &str, rust_description: &str) -> Result<Self, CskkError> {
        let id = CString::new(rust_id.to_string())?;
        let name = CString::new(rust_name.to_string())?;
        let description = CString::new(rust_description.to_string())?;
        Ok(CskkRulesFfi {
            id: id.into_raw(),
            name: name.into_raw(),
            description: description.into_raw(),
        })
    }
}

impl Drop for CskkRulesFfi {
    fn drop(&mut self) {
        unsafe {
            drop(CString::from_raw(self.id));
            drop(CString::from_raw(self.name));
            drop(CString::from_raw(self.description));
        }
    }
}

///
/// 入力途上の状態を返す構造体群
/// CompositionModeに合わせた構造体で、各要素は存在すれば\0終端のUTF-8文字配列、存在しなければNULLが含まれる。
///
// Using long common postfix name because C header cannot have same name even in different enum.
#[repr(C)]
pub enum CskkStateInfoFfi {
    DirectStateInfo(DirectDataFfi),
    /// PreCompositionはAbbreviationモードを含む。
    PreCompositionStateInfo(PreCompositionDataFfi),
    PreCompositionOkuriganaStateInfo(PreCompositionDataFfi),
    CompositionSelectionStateInfo(CompositionSelectionDataFfi),
    RegisterStateInfo(RegisterDataFfi),
    CompleteStateInfo(CompleteDataFfi),
}

#[repr(C)]
pub struct DirectDataFfi {
    /// pollされた時に返す確定済み文字列。
    ///
    /// 通常のIMEでは[CskkContext::poll_output]で都度取り出して確定文字列として渡すので空である。
    pub confirmed: *mut c_char,
    /// まだかな変換を成されていないキー入力の文字列表現
    pub unconverted: *mut c_char,
}

impl Drop for DirectDataFfi {
    fn drop(&mut self) {
        unsafe {
            if !self.confirmed.is_null() {
                drop(CString::from_raw(self.confirmed));
            }
            if !self.unconverted.is_null() {
                drop(CString::from_raw(self.unconverted));
            }
        }
    }
}

#[repr(C)]
pub struct CompositionSelectionDataFfi {
    /// 通常、pollされた時に返す確定済み文字列。
    pub confirmed: *mut c_char,
    /// 現在選択されている変換候補
    pub composited: *mut c_char,
    /// 現在の変換候補に付く送り仮名
    pub okuri: *mut c_char,
    /// 現在の候補のアノテーション
    pub annotation: *mut c_char,
}

impl Drop for CompositionSelectionDataFfi {
    fn drop(&mut self) {
        unsafe {
            if !self.confirmed.is_null() {
                drop(CString::from_raw(self.composited));
            }
            if !self.composited.is_null() {
                drop(CString::from_raw(self.composited));
            }
            if !self.okuri.is_null() {
                drop(CString::from_raw(self.okuri));
            }
            if !self.annotation.is_null() {
                drop(CString::from_raw(self.annotation));
            }
        }
    }
}

#[repr(C)]
pub struct PreCompositionDataFfi {
    /// pollされた時に返す確定済み文字列。
    ///
    /// 通常のIMEでは[poll_output]で都度取り出して確定文字列として渡すので空である。
    pub confirmed: *mut c_char,
    /// 漢字変換に用いようとしている部分
    pub kana_to_composite: *mut c_char,
    /// 漢字変換時に送り仮名として用いようとしている部分
    pub okuri: *mut c_char,
    /// かな変換が成されていない入力キーの文字列表現。
    ///
    /// 現在のCompositionModeがPreCompositionならば漢字変換に用いようとしている部分に付き、
    ///
    /// PreCompositionOkuriganaならば送り仮名に用いようとしている部分に付く。
    ///
    /// surrounding_textで指定範囲のみからの変換に対応していない現在、正常な遷移ではRegisterには存在しない。
    pub unconverted: *mut c_char,
}

impl Drop for PreCompositionDataFfi {
    fn drop(&mut self) {
        unsafe {
            if !self.confirmed.is_null() {
                drop(CString::from_raw(self.confirmed));
            }
            if !self.kana_to_composite.is_null() {
                drop(CString::from_raw(self.kana_to_composite));
            }
            if !self.okuri.is_null() {
                drop(CString::from_raw(self.okuri));
            }
            if !self.unconverted.is_null() {
                drop(CString::from_raw(self.unconverted));
            }
        }
    }
}

#[repr(C)]
pub struct RegisterDataFfi {
    /// pollされた時に返す確定済み文字列。
    ///
    /// 通常のIMEでは[poll_output]で都度取り出して確定文字列として渡すので空である。
    pub confirmed: *mut c_char,
    /// 漢字変換に用いようとしている部分
    pub kana_to_composite: *mut c_char,
    /// 漢字変換時に送り仮名として用いようとしている部分
    pub okuri: *mut c_char,
    /// 漢字変換時に後に付く部分。auto-start-henkanの「。」等
    pub postfix: *mut c_char,
}

impl Drop for RegisterDataFfi {
    fn drop(&mut self) {
        unsafe {
            if !self.confirmed.is_null() {
                drop(CString::from_raw(self.confirmed));
            }
            if !self.kana_to_composite.is_null() {
                drop(CString::from_raw(self.kana_to_composite));
            }
            if !self.okuri.is_null() {
                drop(CString::from_raw(self.okuri));
            }
            if !self.postfix.is_null() {
                drop(CString::from_raw(self.postfix));
            }
        }
    }
}

#[repr(C)]
pub struct CompleteDataFfi {
    /// 通常、pollされた時に返す確定済み文字列。
    pub confirmed: *mut c_char,
    /// 補完に用いようとしている元の部分
    pub complete_origin: *mut c_char,
    /// 補完の送り仮名として用いようとしている部分。v3.0.0では送り仮名付きからの補完は未実装。
    pub origin_okuri: *mut c_char,
    /// 現在選択されている変換候補の見出し。
    pub completed_midashi: *mut c_char,
    /// 現在選択されている変換候補。見出しの候補ではなく変換候補そのもの。
    pub completed: *mut c_char,
    /// 現在の変換候補に付く送り仮名。v3.0.0では送り仮名付きからの補完は未実装。
    pub okuri: *mut c_char,
    /// 現在の候補のアノテーション
    pub annotation: *mut c_char,
}

impl Drop for CompleteDataFfi {
    fn drop(&mut self) {
        unsafe {
            if !self.confirmed.is_null() {
                drop(CString::from_raw(self.complete_origin));
            }
            if !self.complete_origin.is_null() {
                drop(CString::from_raw(self.complete_origin));
            }
            if !self.origin_okuri.is_null() {
                drop(CString::from_raw(self.origin_okuri));
            }
            if !self.completed.is_null() {
                drop(CString::from_raw(self.completed));
            }
            if !self.okuri.is_null() {
                drop(CString::from_raw(self.okuri));
            }
            if !self.annotation.is_null() {
                drop(CString::from_raw(self.annotation));
            }
        }
    }
}

///
/// Returns newly allocated CSKKContext.
/// On error, returns NULL.
///
/// # Safety
/// Caller have to retain the pointer returned from this function
/// Caller must free it using skk_free_context when non-NULL pointer is returned.
/// dictionary_array must have at least dictionary_count number of CskkDictionary
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new(
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) -> *mut CskkContext {
    let dict_array = dictionaries_from_c_repr(dictionary_array, dictionary_count);
    let maybe_context = CskkContext::new(InputMode::Hiragana, CompositionMode::Direct, dict_array);

    if let Ok(context) = maybe_context {
        Box::into_raw(Box::new(context))
    } else {
        ptr::null_mut()
    }
}

///
/// Returns newly allocated CSKKContext.
/// On error, still tries to return an empty context that can convert nothing.
///
/// Try to use skk_context_new when possible.
/// This interface is for IMEs that cannot fail on creating context.
///
/// # Safety
/// Caller have to retain the pointer returned from this function
/// Caller must free it using skk_free_context.
/// dictionary_array must have at least dictionary_count number of CskkDictionary
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new_with_empty_fallback(
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) -> *mut CskkContext {
    let dict_array = dictionaries_from_c_repr(dictionary_array, dictionary_count);
    let context = CskkContext::new_with_empty_fallback(
        InputMode::Hiragana,
        CompositionMode::Direct,
        dict_array,
    );

    Box::into_raw(Box::new(context))
}

///
/// Creates a skk static file dict based on the path_string. Returns the pointer of it.
/// Returns NULL on error. In error case, you don't have to free it.
///
/// # Safety
/// c_path_string and c_encoidng must be a valid c string that terminates with \0.
///
/// Dictionary must be freed by skk_free_dictionary
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_file_dict_new(
    c_path_string: *const c_char,
    c_encoding: *const c_char,
    use_for_completion: bool,
) -> *mut CskkDictionaryFfi {
    let maybe_dictionary = (|| -> anyhow::Result<CskkDictionaryFfi> {
        let path = CStr::from_ptr(c_path_string).to_str()?;
        let encoding = CStr::from_ptr(c_encoding).to_str()?;
        let dictionary = CskkDictionary::new_static_dict(path, encoding, use_for_completion)?;
        Ok(CskkDictionaryFfi {
            dictionary: Arc::new(dictionary),
        })
    })();

    if let Ok(ffi_dictionary) = maybe_dictionary {
        Box::into_raw(Box::new(ffi_dictionary))
    } else {
        ptr::null_mut()
    }
}

///
/// Creates a skk read and write user dict based on the path_string. Returns the pointer of it.
/// Returns NULL on error. In error case, you don't have to free it.
///
/// # Safety
/// c_path_string and c_encoidng must be a valid c string that terminates with \0.
///
/// Dictionary must be freed by skk_free_dictionary
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_user_dict_new(
    c_path_string: *const c_char,
    c_encoding: *const c_char,
    use_for_completion: bool,
) -> *mut CskkDictionaryFfi {
    let maybe_dictionary = (|| -> anyhow::Result<CskkDictionaryFfi> {
        let path = CStr::from_ptr(c_path_string).to_str()?;
        let encoding = CStr::from_ptr(c_encoding).to_str()?;
        let dictionary = CskkDictionary::new_user_dict(path, encoding, use_for_completion)?;
        Ok(CskkDictionaryFfi {
            dictionary: Arc::new(dictionary),
        })
    })();

    if let Ok(ffi_dictionary) = maybe_dictionary {
        Box::into_raw(Box::new(ffi_dictionary))
    } else {
        ptr::null_mut()
    }
}

///
/// Creates an empty dictionary. Returns the pointer of it.
/// On error returns NULL pointer.
///
/// # Safety
/// Dictionary must be freed by skk_free_dictionary
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_empty_dict_new() -> *mut CskkDictionaryFfi {
    let maybe_result = (|| -> anyhow::Result<CskkDictionaryFfi> {
        Ok(CskkDictionaryFfi {
            dictionary: Arc::new(CskkDictionary::new_empty_dict()?),
        })
    })();

    if let Ok(dict) = maybe_result {
        Box::into_raw(Box::new(dict))
    } else {
        ptr::null_mut()
    }
}

///
/// Set the input mode of current state.
///
#[no_mangle]
pub extern "C" fn skk_context_set_input_mode(context: &mut CskkContext, input_mode: InputMode) {
    skk_context_set_input_mode_rs(context, input_mode)
}

///
/// Get the input mode of current state.
///
#[no_mangle]
pub extern "C" fn skk_context_get_input_mode(context: &mut CskkContext) -> InputMode {
    skk_context_get_input_mode_rs(context)
}

///
/// Get the composition mode of current state.
///
#[no_mangle]
pub extern "C" fn skk_context_get_composition_mode(context: &mut CskkContext) -> CompositionMode {
    skk_context_get_composition_mode_rs(context)
}

///
/// Sets the conversion rule to the given rule_name.
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// rule_name must be a valid pointer to a C-style string with string length smaller than 2^32 - 1
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_set_rule(
    context: &mut CskkContext,
    rule_name: *const c_char,
) -> c_int {
    let any_error = (|| -> anyhow::Result<()> {
        let rule_name_str = CStr::from_ptr(rule_name);
        let rule_name_str = rule_name_str.to_str()?;
        context.set_rule(rule_name_str)?;
        Ok(())
    })();

    if any_error.is_err() {
        return -1;
    }

    0
}

/// Only for library test purpose. Do not use.
/// # Safety
///
/// This function must be called by a valid C string terminated by a NULL.
#[no_mangle]
pub unsafe extern "C" fn skk_context_process_key_events(
    context: &mut CskkContext,
    keyevents_cstring: *mut c_char,
) -> bool {
    let maybe_result = (|| -> anyhow::Result<bool> {
        let keyevents = CStr::from_ptr(keyevents_cstring);
        Ok(context.process_key_events_string(keyevents.to_str()?))
    })();

    if let Ok(result) = maybe_result {
        result
    } else {
        false
    }
}

///
/// 1入力を処理する。CSKKコンテキスト内でキーが受理された場合はtrueを返す。
/// CSKKでは入力としてもコマンドとしても受けつけられなかった場合はfalseを返す。
///
/// keyeventはfreeされる。
///
/// # Safety
/// context and keyevent must be a valid non-null pointer created from this library.
/// keyevent will be freed. keyevent must not be reused after this function call
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_process_key_event(
    context: &mut CskkContext,
    keyevent: *mut CskkKeyEvent,
) -> bool {
    let raw_keyevent = Box::from_raw(keyevent);
    context.process_key_event(raw_keyevent.as_ref())
}

/// 現在のoutputをpollingする。
///
/// 成功時にはRustでallocateされたCの文字列として扱える(=ヌル終端のある)UTF-8のバイト配列を返す。
/// C++20のchar8_t*のようなもの。
/// エラー時にはNULLを返す。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないと実体がメモリリークする。
///
#[no_mangle]
pub extern "C" fn skk_context_poll_output(context: &mut CskkContext) -> *mut c_char {
    // Free時にmutである必要があるので*mut c_charで返しているが、c側で変更することを想定していない。
    // Cではどうせ制約を付けられないので、*constで返しても意味はないが、本当は*constで返しておきながらfreeの引数としては*mutで受けたい。
    let maybe_result =
        (|| -> anyhow::Result<CString> { Ok(CString::new(skk_context_poll_output_rs(context))?) })(
        );

    if let Ok(result) = maybe_result {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

///
/// preedit文字列を返す。
/// 返り値は\0終端のUTF-8文字配列。C++20で言うchar8_t*
/// 失敗時にはNULLを返す。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないとメモリリークする。
/// ueno/libskkと違う点なので注意が必要
///
#[no_mangle]
pub extern "C" fn skk_context_get_preedit(context: &CskkContext) -> *mut c_char {
    let maybe_result = (|| -> anyhow::Result<CString> {
        let maybe_preedit = context.get_preedit();
        if let Some(preedit) = maybe_preedit {
            Ok(CString::new(preedit)?)
        } else {
            // 実質 unreachable!()
            Err(Error("no preedit".to_string()).into())
        }
    })();

    if let Ok(result) = maybe_result {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

///
/// preedit状態の配列を返す。
/// 最初のものが一番外側の状態で、Registerモードの時には後にその内側の状態が続く。
/// 結果の配列の長さは引数のstate_stack_lenにセットする。
/// 失敗時にはNULLを返す。
///
/// # Safety
/// 返り値の内容およびポインタは変更してはならない。
/// 返り値はcallerがskk_free_preedit_detailで解放しないとメモリリークする
/// state_stack_lenが有効なunsigned intへのポインタでないと予期せぬ動作を起こす。
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_get_preedit_detail(
    context: &CskkContext,
    state_stack_len: *mut c_uint,
) -> *mut CskkStateInfoFfi {
    let preedit = context.get_preedit_detail();
    let mut converted = preedit
        .into_iter()
        .map(convert_state_info)
        .collect::<Vec<CskkStateInfoFfi>>();
    *state_stack_len = u32::try_from(converted.len()).unwrap_or_default();

    if !converted.is_empty() {
        // Make Vec capacity to be equals length so that we can restore on free function.
        converted.set_len(converted.len());
        let mut retval = ManuallyDrop::new(converted);

        retval.as_mut_ptr()
    } else {
        // Must treat specially since Vec with 0 capacity has some value not guaranteed to be NULL in C.
        // See https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees
        ptr::null_mut()
    }
}

/// CskkStateInfoのStringからcstringに変換したFFI用の構造体を返す。
fn convert_state_info(state_info: CskkStateInfo) -> CskkStateInfoFfi {
    match state_info {
        CskkStateInfo::Direct(direct_data) => {
            let confirmed = CString::new(direct_data.confirmed)
                .unwrap_or_default()
                .into_raw();

            let unconverted = if direct_data.unconverted.is_some() {
                CString::new(direct_data.unconverted.unwrap())
                    .unwrap()
                    .into_raw()
            } else {
                ptr::null_mut()
            };
            CskkStateInfoFfi::DirectStateInfo(DirectDataFfi {
                confirmed,
                unconverted,
            })
        }
        CskkStateInfo::PreComposition(precomposition_data) => {
            CskkStateInfoFfi::PreCompositionStateInfo(convert_precomposition_data(
                precomposition_data,
            ))
        }
        CskkStateInfo::PreCompositionOkurigana(precomposition_data) => {
            CskkStateInfoFfi::PreCompositionOkuriganaStateInfo(convert_precomposition_data(
                precomposition_data,
            ))
        }
        CskkStateInfo::Register(register_data) => {
            let confirmed = CString::new(register_data.confirmed)
                .unwrap_or_default()
                .into_raw();
            let kana_to_composite = CString::new(register_data.kana_to_composite)
                .unwrap_or_default()
                .into_raw();
            let okuri = if let Some(okuri_string) = register_data.okuri {
                CString::new(okuri_string).unwrap_or_default().into_raw()
            } else {
                ptr::null_mut()
            };
            let postfix = if let Some(unconverted_string) = register_data.postfix {
                CString::new(unconverted_string).unwrap().into_raw()
            } else {
                ptr::null_mut()
            };

            CskkStateInfoFfi::RegisterStateInfo(RegisterDataFfi {
                confirmed,
                kana_to_composite,
                okuri,
                postfix,
            })
        }
        CskkStateInfo::CompositionSelection(composition_selection_data) => {
            let confirmed = CString::new(composition_selection_data.confirmed)
                .unwrap()
                .into_raw();
            let composited = CString::new(composition_selection_data.composited)
                .unwrap()
                .into_raw();
            let okuri = if let Some(okuri_string) = composition_selection_data.okuri {
                CString::new(okuri_string).unwrap_or_default().into_raw()
            } else {
                ptr::null_mut()
            };
            let annotation = if let Some(annotation_string) = composition_selection_data.annotation
            {
                CString::new(annotation_string)
                    .unwrap_or_default()
                    .into_raw()
            } else {
                ptr::null_mut()
            };
            CskkStateInfoFfi::CompositionSelectionStateInfo(CompositionSelectionDataFfi {
                confirmed,
                composited,
                okuri,
                annotation,
            })
        }
        CskkStateInfo::Complete(complete_data) => {
            let confirmed = CString::new(complete_data.confirmed).unwrap().into_raw();
            let complete_origin = CString::new(complete_data.complete_origin)
                .unwrap()
                .into_raw();
            let completed_midashi = CString::new(complete_data.completed_midashi)
                .unwrap()
                .into_raw();
            let completed = CString::new(complete_data.completed).unwrap().into_raw();
            let annotation = if let Some(annotation) = complete_data.annotation {
                CString::new(annotation).unwrap().into_raw()
            } else {
                ptr::null_mut()
            };
            CskkStateInfoFfi::CompleteStateInfo(CompleteDataFfi {
                confirmed,
                complete_origin,
                origin_okuri: ptr::null_mut(),
                completed_midashi,
                completed,
                okuri: ptr::null_mut(),
                annotation,
            })
        }
    }
}

fn convert_precomposition_data(precomposition_data: PreCompositionData) -> PreCompositionDataFfi {
    let confirmed = CString::new(precomposition_data.confirmed)
        .unwrap_or_default()
        .into_raw();
    let kana_to_composite = CString::new(precomposition_data.kana_to_composite)
        .unwrap_or_default()
        .into_raw();
    let okuri = if let Some(okuri_string) = precomposition_data.okuri {
        CString::new(okuri_string).unwrap_or_default().into_raw()
    } else {
        ptr::null_mut()
    };
    let unconverted = if let Some(unconverted_string) = precomposition_data.unconverted {
        CString::new(unconverted_string).unwrap().into_raw()
    } else {
        ptr::null_mut()
    };

    PreCompositionDataFfi {
        confirmed,
        kana_to_composite,
        okuri,
        unconverted,
    }
}

///
/// preedit_detailsを解放する。
///
/// # Safety
/// ptrとlengthはskk_context_get_preedit_detailsの返り値でなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_preedit_detail(ptr: *mut CskkStateInfoFfi, length: c_uint) {
    if ptr.is_null() {
        return;
    }
    let length = length as usize;
    drop(Vec::from_raw_parts(ptr, length, length))
}

///
/// cskk libraryが渡したC言語文字列をfreeする。
///
/// # Safety
///
/// CSKKライブラリで返したC言語文字列のポインタ以外を引数に渡してはいけない。
/// 他で管理されるべきメモリを過剰に解放してしまう。
///
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn skk_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // Get back ownership in Rust side, then drop.
    drop(CString::from_raw(ptr));
}

///
/// save current dictionaries
///
#[no_mangle]
pub extern "C" fn skk_context_save_dictionaries(context: &mut CskkContext) {
    context.save_dictionary();
}

///
/// CskkContextを解放する。
///
/// # Safety
///
/// context_ptr は必ずCskkContextのポインタでなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_context(context_ptr: *mut CskkContext) {
    if context_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(context_ptr));
}

///
/// CskkDictionaryを解放する。
///
/// # Safety
///
/// dictionary_ptr は必ずCskkDictionaryのポインタでなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_dictionary(dictionary_ptr: *mut CskkDictionaryFfi) {
    if dictionary_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(dictionary_ptr));
}

///
/// Free the rules given from [skk_context_get_rules]
///
/// # Safety
/// 引数はいずれも[skk_context_get_rules]で得られるペアでなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_rules(rules_ptr: *mut CskkRulesFfi, length: c_uint) {
    let length = length as usize;
    drop(Vec::from_raw_parts(rules_ptr, length, length));
}

///
/// Get emphasizing range of preedit.
/// offset: starting offset (in UTF-8 chars) of underline
/// nchars: number of characters to be underlined
///
/// # Deprecated
/// Fancy formatting will be delegated to IME in favor of [skk_context_get_preedit_detail].
///
/// Deprecated interface might be deleted on major update.
///
/// # Safety
///
/// offset, nchars must be a valid pointer to int type that memory is allocated by the caller.
#[no_mangle]
pub unsafe extern "C" fn skk_context_get_preedit_underline(
    context: &mut CskkContext,
    offset: *mut c_int,
    nchars: *mut c_int,
) {
    let (offset_size, nchars_size) = context.get_preedit_underline();
    *offset = c_int::try_from(offset_size).unwrap_or(0);
    *nchars = c_int::try_from(nchars_size).unwrap_or(0);
}

///
/// Set dictionaries to context.
///
/// # Safety
/// dictionary_array must have at least dictionary_count number of CskkDictionary
#[no_mangle]
pub unsafe extern "C" fn skk_context_set_dictionaries(
    context: &mut CskkContext,
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) {
    let dict_array = dictionaries_from_c_repr(dictionary_array, dictionary_count);
    skk_context_set_dictionaries_rs(context, dict_array);
}

///
/// Create a cskk keyevent type
/// keysym: u32  "X11 Window System Protocol" Appendix A based keysym code.
/// modifier: Fcitx modifier
/// is_release: True if this event for releasing the key
///
/// # Safety
/// Must use this return value with process_key_event. If not, memory leaks.
///
#[no_mangle]
pub extern "C" fn skk_key_event_new_from_fcitx_keyevent(
    keysym: u32,
    modifier: u32,
    is_release: bool,
) -> *mut CskkKeyEvent {
    Box::into_raw(Box::new(CskkKeyEvent::from_fcitx_keyevent(
        keysym, modifier, is_release,
    )))
}

///
/// Reset the context. Doesn't change the inputmode but flushes all inputs and compisitionmode will be reset to direct
///
#[no_mangle]
pub extern "C" fn skk_context_reset(context: &mut CskkContext) {
    skk_context_reset_rs(context);
}

///
/// 現在の漢字変換対象文字列をUTF-8のC文字列で返す。
/// Cで扱えない文字列などの失敗時にはNULLが返る。
///
/// # Safety
/// 返り値はCallerがskk_free_stringしなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_get_current_to_composite(
    context: &CskkContext,
) -> *mut c_char {
    let maybe_result = { CString::new(skk_context_get_current_to_composite_rs(context)) };

    if let Ok(result) = maybe_result {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

///
/// 現在のcandidate listの長さを返す。
///
#[no_mangle]
pub extern "C" fn skk_context_get_current_candidate_count(context: &CskkContext) -> c_uint {
    skk_context_get_current_candidate_count_rs(context) as c_uint
}

///
/// context内の現在のcandidate listとして候補のリストをUTF-8のC形式のNULL終端文字列で返す。
///
/// 現在のリストのoffsetから最大でbuf_sizeまでをcandidate_buf内に入れる。
/// 実際に返した個数は返り値として返す。
///
/// 失敗時には返り値-1を返す。
/// 失敗時にはcandidate_bufの中身については何も保証されず、freeする必要はない。
///
/// # Safety
///
/// candidate_bufに*c_charがbuf_size分の容量があることはCaller側が保証しなければならない。
/// candidate_bufに入れられた文字列はCallerがskk_free_candidate_listで解放しなければならない。
/// candidate_bufに入れられた文字列の長さは変更してはならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_get_current_candidates(
    context: &CskkContext,
    candidate_buf: *mut *mut c_char,
    buf_size: c_uint,
    offset: c_uint,
) -> c_int {
    let candidates = skk_context_get_current_candidates_rs(context);
    let buffer = slice::from_raw_parts_mut(candidate_buf, buf_size as usize);

    let offset = offset as usize;
    let buf_size = buf_size as usize;
    let returning_list = candidates.iter().skip(offset).take(buf_size).enumerate();
    let count = returning_list.len();
    for (i, candidate) in returning_list {
        let maybe_c_string = CString::new(candidate.output.to_string());
        if let Ok(c_string) = maybe_c_string {
            buffer[i] = c_string.into_raw();
        } else {
            // Cleanup the CStrings we allocated so far and return -1
            #[allow(clippy::needless_range_loop)]
            for j in 0..i {
                drop(CString::from_raw(buffer[j]))
            }
            return -1;
        }
    }

    count as c_int
}

///
/// candidate_listの各々の候補を解放する。
///
/// # Safety
///
/// candidate_list_ptr は必ずskk_context_get_current_candidatesで候補を取得した配列のポインタでなければならない。
/// sizeは取得した候補の全数と一致している必要がある。
///
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn skk_free_candidate_list(
    candidate_list_ptr: *mut *mut c_char,
    size: c_uint,
) {
    if candidate_list_ptr.is_null() {
        return;
    }
    let list = slice::from_raw_parts_mut(candidate_list_ptr, size as usize);
    for candidate in list.iter() {
        drop(CString::from_raw(*candidate));
    }
}

/// 何番目の候補を指しているかを返す
/// 現在候補が存在しない場合や、返せない場合、適当に負数を返す
#[no_mangle]
pub extern "C" fn skk_context_get_current_candidate_cursor_position(
    context: &mut CskkContext,
) -> c_int {
    if let Ok(selection) = skk_context_get_current_candidate_cursor_position_rs(context) {
        if selection > c_int::MAX as usize {
            -2
        } else {
            selection as c_int
        }
    } else {
        -1
    }
}

/// i番目の候補を指す。確定はしない。
/// 候補が負の方向に範囲外の場合、▼モードから▽モードに戻る。
/// 候補が正の方向に範囲外の場合、最後の候補を指し辞書登録モードに移る。
///
/// 現在のコンテキストが▼モード(CompositionSelection)でない場合、無視してfalseを返す。
#[no_mangle]
pub extern "C" fn skk_context_select_candidate_at(context: &mut CskkContext, i: c_int) -> bool {
    skk_context_select_candidate_at_rs(context, i)
}

///
/// i番目の候補として確定し、Directモードに移行する。
/// 現在のコンテキストが▼モード(CompositionSelection)でない場合や、iがリスト範囲外になる場合は、無視してfalseを返す。
///
#[no_mangle]
pub extern "C" fn skk_context_confirm_candidate_at(context: &mut CskkContext, i: c_uint) -> bool {
    skk_context_confirm_candidate_at_rs(context, i as usize)
}

/// Set auto start henkan keywords
///
/// # Safety
/// keywords_array must be an pointer of C-style array that contains at least keywords_count number of C string.
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_set_auto_start_henkan_keywords(
    context: &mut CskkContext,
    keywords_array: &*const c_char,
    keywords_count: usize,
) {
    let mut keywords = vec![];
    if keywords_count < 1 || keywords_array.is_null() {
        skk_context_set_auto_start_henkan_keywords_rs(context, keywords);
        return;
    }
    let tmp_array = slice::from_raw_parts(keywords_array, keywords_count);
    for raw_c_keyword in tmp_array {
        let ckeyword = CStr::from_ptr(*raw_c_keyword);
        if let Ok(keyword_str) = ckeyword.to_str() {
            keywords.push(keyword_str.to_string())
        }
    }
    skk_context_set_auto_start_henkan_keywords_rs(context, keywords);
}

///
/// Period style を設定する
///
#[no_mangle]
pub extern "C" fn skk_context_set_period_style(
    context: &mut CskkContext,
    period_style: PeriodStyle,
) {
    skk_context_set_period_style_rs(context, period_style)
}

///
/// Comma style を設定する
///
#[no_mangle]
pub extern "C" fn skk_context_set_comma_style(context: &mut CskkContext, comma_style: CommaStyle) {
    skk_context_set_comma_style_rs(context, comma_style)
}

///
/// Get library version at the compile time.
/// 失敗時にはNULLを返す。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないと実体がメモリリークする。
///
#[no_mangle]
pub extern "C" fn skk_library_get_version() -> *mut c_char {
    let maybe_result = CString::new(CskkContext::get_version());

    if let Ok(result) = maybe_result {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

///
/// returns a pointer to an Rules array. Retruns NULL in error.
/// sets the total number of entries in the array in the given `length` parameter.
/// The parameter is required to free the returned array later.
///
/// 失敗時にはNULLを返し、lengthの内容は保証されない。
/// 失敗時にはfreeする必要はない。
///
/// # Safety
///
/// the parameter `length` must be a pointer to a 32 bit unsigned int allocated in the caller side.
/// caller must not modify any field in the returned struct in the array.
/// caller must free the returned array by [skk_free_rules] API.
///
#[no_mangle]
pub unsafe extern "C" fn skk_get_rules(length: *mut c_uint) -> *mut CskkRulesFfi {
    let mut retval_stack = vec![];

    let maybe_count = (|| -> anyhow::Result<usize> {
        let rulemap = get_available_rules()?;

        let count = rulemap.len();
        for (key, metadataentry) in rulemap {
            let rule = CskkRulesFfi::new(&key, &metadataentry.name, &metadataentry.description)?;
            retval_stack.push(rule);
        }
        // Make Vec capacity to be equals length so that we can restore on free function.
        retval_stack.set_len(count);
        // FIXME: Here, if user had make more than 2^32-1 rules this will cause trouble.
        // 2^32 rules are very unlikely. Low priority for now.
        *length = u32::try_from(count)?;
        Ok(count)
    })();

    if let Ok(count) = maybe_count {
        if count > 0 {
            let mut retval = ManuallyDrop::new(retval_stack);
            retval.as_mut_ptr()
        } else {
            // Must treat specially since Vec with 0 capacity has some value not guaranteed to be NULL in C.
            // See https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees
            ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

///
/// # Safety
///
/// dictionary_array must have at least dictionary_count number of CskkDictionary
unsafe fn dictionaries_from_c_repr(
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) -> Vec<Arc<CskkDictionary>> {
    let mut dict_array = vec![];
    if dictionary_count < 1 || dictionary_array.is_null() {
        return dict_array;
    }

    let tmp_array = slice::from_raw_parts(dictionary_array, dictionary_count);
    for dictref in tmp_array {
        let cskkdict = Box::from_raw(*dictref);
        dict_array.push(cskkdict.dictionary.clone());
        Box::into_raw(cskkdict);
    }
    dict_array
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rulesffi() {
        let rule = CskkRulesFfi::new("id", "name", "description").unwrap();
        unsafe {
            assert_eq!(b'i', *rule.id as u8);
            assert_eq!(b'd', *rule.id.offset(1) as u8);
            assert_eq!(b'\0', *rule.id.offset(2) as u8);
        }
    }
}
