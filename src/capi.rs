use crate::dictionary::CskkDictionary;
use crate::skk_modes::{InputMode, PeriodStyle};
use crate::{
    skk_context_get_input_mode_rs, skk_context_new_rs, skk_context_poll_output_rs,
    skk_context_set_input_mode_rs, skk_file_dict_new_rs, CskkContext,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

/// Returns newly allocated CSKKContext.
///
/// # Safety
/// Caller have to retain the pointer
/// Caller must free the memory using skk_context_destroy
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new(
    dictionary_array: *const *mut CskkDictionary,
    dictionary_count: usize,
) -> *mut CskkContext {
    let tmp_array = slice::from_raw_parts(dictionary_array, dictionary_count);
    let mut dict_array = vec![];
    for dictref in tmp_array {
        let cskkdict = *Box::from_raw(*dictref);
        dict_array.push(cskkdict);
    }
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

/// 現在のoutputをpollingする。
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

/// テスト用途？。preedit文字列と同じ内容の文字列を取得する。
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
pub unsafe extern "C" fn skk_context_free(context_ptr: *mut CskkContext) {
    if context_ptr.is_null() {
        return;
    }
    Box::from_raw(context_ptr);
}
