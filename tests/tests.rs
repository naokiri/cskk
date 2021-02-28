extern crate cskk;

use cskk::{create_new_context, skk_context_set_composition_mode, skk_context_set_input_mode};
use cskk::skk_modes::{CompositionMode, InputMode};


#[test]
fn basic_input() {
    let mut context = create_new_context();
    skk_context_set_composition_mode(context.as_mut(), CompositionMode::Direct);
    skk_context_set_input_mode(context.as_mut(), InputMode::Hiragana);


}