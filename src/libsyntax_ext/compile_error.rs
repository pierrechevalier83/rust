// The compiler code necessary to support the compile_error! extension.

use syntax::ext::base::*;
use syntax::ext::base;
use syntax_pos::Span;
use syntax::tokenstream;

pub fn expand_compile_error<'cx>(cx: &'cx mut ExtCtxt,
                              sp: Span,
                              tts: &[tokenstream::TokenTree])
                              -> Box<dyn base::MacResult + 'cx> {
    let var = match get_single_str_from_tts(cx, sp, tts, "compile_error!") {
        None => return DummyResult::expr(sp),
        Some(v) => v,
    };

    cx.span_err(sp, &var);

    DummyResult::any(sp)
}
