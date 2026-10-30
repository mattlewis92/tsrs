// What an embedder of tsrs's crates calls to merge command-line options over a tsconfig's, as tsconfig parsing does.

use tsrs_core::{CompilerOptions, Tristate};

use tsrs_tsoptions::parsinghelpers::merge_compiler_options;

#[test]
fn merge_compiler_options_sets_what_the_source_sets() {
    let mut target = CompilerOptions { strict: Tristate::True, ..CompilerOptions::default() };
    let source = CompilerOptions { no_emit: Tristate::True, ..CompilerOptions::default() };
    merge_compiler_options(&mut target, Some(&source), None);
    assert_eq!(target.no_emit, Tristate::True);
    assert_eq!(target.strict, Tristate::True);
}
