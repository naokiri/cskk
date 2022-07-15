use crate::dictionary::CskkDictionary;
use crate::keyevent::CskkKeyEvent;
use crate::skk_modes::{CommaStyle, CompositionMode, InputMode, PeriodStyle};
use crate::{
    skk_context_confirm_candidate_at_rs, skk_context_get_composition_mode_rs,
    skk_context_get_current_candidate_count_rs,
    skk_context_get_current_candidate_cursor_position_rs, skk_context_get_current_candidates_rs,
    skk_context_get_current_to_composite_rs, skk_context_get_input_mode_rs, skk_context_new_rs,
    skk_context_poll_output_rs, skk_context_reset_rs, skk_context_select_candidate_at_rs,
    skk_context_set_auto_start_henkan_keywords_rs, skk_context_set_comma_style_rs,
    skk_context_set_dictionaries_rs, skk_context_set_input_mode_rs,
    skk_context_set_period_style_rs, skk_file_dict_new_rs, skk_user_dict_new_rs, CskkContext,
};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::slice;
use std::sync::Arc;

pub struct CskkDictionaryFfi {
    dictionary: Arc<CskkDictionary>,
}

/// Returns newly allocated CSKKContext.
///
/// # Safety
/// Caller have to retain the pointer returned from this function
/// Caller must free it using skk_free_context
/// dictionary_array must have at least dictionary_count number of CskkDictionary
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new(
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) -> *mut CskkContext {
    let dict_array = dictionaries_from_c_repr(dictionary_array, dictionary_count);
    Box::into_raw(Box::new(skk_context_new_rs(dict_array)))
}

///
/// Creates a skk static file dict based on the path_string. Returns the pointer of it.
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
) -> *mut CskkDictionaryFfi {
    let path = CStr::from_ptr(c_path_string);
    let encoding = CStr::from_ptr(c_encoding);
    let cskk_dictionary_ffi = CskkDictionaryFfi {
        dictionary: Arc::new(skk_file_dict_new_rs(
            path.to_str().unwrap(),
            encoding.to_str().unwrap(),
        )),
    };
    Box::into_raw(Box::new(cskk_dictionary_ffi))
}

///
/// Creates a skk read and write user dict based on the path_string. Returns the pointer of it.
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
) -> *mut CskkDictionaryFfi {
    let path = CStr::from_ptr(c_path_string);
    let encoding = CStr::from_ptr(c_encoding);

    Box::into_raw(Box::new(CskkDictionaryFfi {
        dictionary: Arc::new(skk_user_dict_new_rs(
            path.to_str().unwrap(),
            encoding.to_str().unwrap(),
        )),
    }))
}

///
/// Creates an empty dictionary. Returns the pointer of it.
///
/// # Safety
/// Dictionary must be freed by skk_free_dictionary
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_empty_dict_new() -> *mut CskkDictionaryFfi {
    Box::into_raw(Box::new(CskkDictionaryFfi {
        dictionary: Arc::new(CskkDictionary::new_empty_dict().unwrap()),
    }))
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

/// Only for library test purpose. Do not use.
/// # Safety
///
/// This function must be called by a valid C string terminated by a NULL.
#[no_mangle]
pub unsafe extern "C" fn skk_context_process_key_events(
    context: &mut CskkContext,
    keyevents_cstring: *mut c_char,
) -> bool {
    let keyevents = CStr::from_ptr(keyevents_cstring);
    context.process_key_events_string(keyevents.to_str().unwrap())
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
/// RustでallocateされたCの文字列として扱える(=ヌル終端のある)UTF-8のバイト配列を返す。
/// C++20のchar8_t*のようなもの。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないと実体がメモリリークする。
///
#[no_mangle]
pub extern "C" fn skk_context_poll_output(context: &mut CskkContext) -> *mut c_char {
    CString::new(skk_context_poll_output_rs(context))
        .unwrap()
        .into_raw()
}

///
/// preedit文字列を返す。
/// 返り値は\0終端のUTF-8文字配列。C++20で言うchar8_t*
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないとメモリリークする。
/// ueno/libskkと違う点なので注意が必要
///
#[no_mangle]
pub extern "C" fn skk_context_get_preedit(context: &CskkContext) -> *mut c_char {
    let preedit = context.get_preedit().unwrap();
    CString::new(preedit).unwrap().into_raw()
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
    // Get back ownership in Rust side, then do nothing.
    CString::from_raw(ptr);
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
    Box::from_raw(context_ptr);
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
    Box::from_raw(dictionary_ptr);
}

///
/// Get emphasizing range of preedit.
/// offset: starting offset (in UTF-8 chars) of underline
/// nchars: number of characters to be underlined
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
///
/// # Safety
/// 返り値はCallerがskk_free_stringしなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_get_current_to_composite(
    context: &CskkContext,
) -> *mut c_char {
    CString::new(skk_context_get_current_to_composite_rs(context))
        .unwrap()
        .into_raw()
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
) -> c_uint {
    let candidates = skk_context_get_current_candidates_rs(context);
    let buffer = slice::from_raw_parts_mut(candidate_buf, buf_size as usize);

    let offset = offset as usize;
    let buf_size = buf_size as usize;
    let returning_list = candidates.iter().skip(offset).take(buf_size).enumerate();
    let count = returning_list.len();
    for (i, candidate) in returning_list {
        let c_string = CString::new(candidate.output.to_string()).unwrap();
        buffer[i] = c_string.into_raw();
    }

    count as c_uint
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
        CString::from_raw(*candidate);
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
