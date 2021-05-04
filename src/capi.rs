use crate::dictionary::{CskkDictionary, CskkDictionaryType};
use crate::keyevent::CskkKeyEvent;
use crate::skk_modes::{InputMode, PeriodStyle};
use crate::{
    skk_context_get_input_mode_rs, skk_context_new_rs, skk_context_poll_output_rs,
    skk_context_reset_rs, skk_context_set_dictionaries_rs, skk_context_set_input_mode_rs,
    skk_file_dict_new_rs, skk_user_dict_new_rs, CskkContext,
};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;
use std::sync::{Arc, Mutex};

pub struct CskkDictionaryFfi {
    dictionary: Arc<Mutex<CskkDictionaryType>>,
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
/// Period style を設定する
///
#[no_mangle]
pub extern "C" fn skk_context_set_period_style(
    context: &mut CskkContext,
    period_style: PeriodStyle,
) {
    context.kana_converter.set_period_style(period_style)
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
/// context_ptr は必ずCskkDictionaryのポインタでなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_dictionary(context_ptr: *mut CskkDictionaryFfi) {
    if context_ptr.is_null() {
        return;
    }
    Box::from_raw(context_ptr);
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
/// # Safety
///
/// dictionary_array must have at least dictionary_count number of CskkDictionary
unsafe fn dictionaries_from_c_repr(
    dictionary_array: &*mut CskkDictionaryFfi,
    dictionary_count: usize,
) -> Vec<Arc<CskkDictionary>> {
    let mut dict_array = vec![];
    if dictionary_array.is_null() {
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
