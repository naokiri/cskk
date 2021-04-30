use crate::dictionary::CskkDictionary;
use crate::keyevent::CskkKeyEvent;
use crate::skk_modes::{InputMode, PeriodStyle};
use crate::{
    skk_context_get_input_mode_rs, skk_context_new_rs, skk_context_poll_output_rs,
    skk_context_set_dictionaries_rs, skk_context_set_input_mode_rs, skk_file_dict_new_rs,
    CskkContext,
};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;

/// Returns newly allocated CSKKContext.
///
/// # Safety
/// Caller have to retain the pointer returned from this function
/// Caller must free it using skk_free_context
/// dictionary_array must have at least dictionary_count number of CskkDictionary
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new(
    dictionary_array: *const *mut CskkDictionary,
    dictionary_count: usize,
) -> *mut CskkContext {
    let dict_array = dictionaries_from_c_repr(dictionary_array, dictionary_count);
    Box::into_raw(Box::new(skk_context_new_rs(dict_array)))
}

///
/// Creates a skk file dict based on the path_string. Returns the pointer of it.
///
/// # Safety
/// c_path_string and c_encoidng must be a valid c string that terminates with \0.
///
/// Dictionary must be handled by a cskk context on creating a new context or registering later.
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_file_dict_new(
    c_path_string: *const c_char,
    c_encoding: *const c_char,
) -> *mut CskkDictionary {
    let path = CStr::from_ptr(c_path_string);
    let encoding = CStr::from_ptr(c_encoding);

    Box::into_raw(Box::new(skk_file_dict_new_rs(
        path.to_str().unwrap(),
        encoding.to_str().unwrap(),
    )))
}

// /// Reset the context
// #[no_mangle]
// pub extern "C" fn skk_context_reset(context: &mut CskkContext) {
//     // TODO: Flush all the state stack after implementing the register mode
//     // TODO: あとまわし。他のテストがこけはじめたらちゃんと実装する。
//     context.poll_output();
//     context.reset_state_stack();
// }

// /// テスト用途。composition modeを設定する。
// /// 他のステートとの整合性は無視される。
// #[no_mangle]
// pub extern "C" fn skk_context_set_composition_mode(
//     context: &mut CskkContext,
//     composition_mode: CompositionMode,
// ) {
//     context.set_composition_mode(composition_mode)
// }
//

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

/// Library test purpose
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
/// 1入力を処理する
///
/// keyeventは消費される。
///
/// # Safety
/// context and keyevent must be a valid non-null pointer created from this library.
/// keyevent must not be reused after this function call
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
/// RustでallocateされたUTF-8のバイト配列を返す
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
    dictionary_array: *const *mut CskkDictionary,
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
/// # Safety
///
/// dictionary_array must have at least dictionary_count number of CskkDictionary
unsafe fn dictionaries_from_c_repr(
    dictionary_array: *const *mut CskkDictionary,
    dictionary_count: usize,
) -> Vec<CskkDictionary> {
    let tmp_array = slice::from_raw_parts(dictionary_array, dictionary_count);
    let mut dict_array = vec![];
    for dictref in tmp_array {
        let cskkdict = *Box::from_raw(*dictref);
        dict_array.push(cskkdict);
    }
    dict_array
}