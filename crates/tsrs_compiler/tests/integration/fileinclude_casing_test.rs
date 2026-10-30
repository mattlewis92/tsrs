// includeprocessor.go addProcessingDiagnosticsForFileCasing: which of TS1149 and TS1261 a casing conflict reports
// depends on the include reasons the file has when the conflict is found.

use std::sync::Arc;

use tsrs_core::P;
use tsrs_tsoptions::{self as tsoptions, ParseConfigHost};
use tsrs_vfs::{FS, vfstest};

use tsrs_compiler::{ProgramOptions, new_compiler_host, new_program};

struct ParseConfigHostImpl {
    fs: Arc<dyn FS>,
}

impl ParseConfigHost for ParseConfigHostImpl {
    fn fs(&self) -> &dyn FS {
        &*self.fs
    }
    fn get_current_directory(&self) -> &'static str {
        "/"
    }
}

fn include_diagnostic_codes_of(files: &[(&str, &str)], file: &str) -> Vec<i32> {
    let fs: Arc<dyn FS> = Arc::new(vfstest::from_map(files.iter().map(|&(k, v)| (k, v)), false));
    let config_host: &'static ParseConfigHostImpl =
        Box::leak(Box::new(ParseConfigHostImpl { fs: Arc::clone(&fs) }));
    let (config, diagnostics) = tsoptions::get_parsed_command_line_of_config_file(
        "/src/tsconfig.json",
        None,
        None,
        config_host,
        None,
    );
    assert_eq!(diagnostics, [] as [P<tsrs_ast::Diagnostic>; 0]);
    let program = new_program(ProgramOptions::new(
        P::new(config.unwrap()),
        new_compiler_host("/", fs, "", None, None, None),
    ));
    let source_file = program.get_source_file(file).expect("the importing file");
    program.get_include_processor_diagnostics(source_file).iter().map(|d| d.code()).collect()
}

// tsgo reports TS1261 at the import: the type library's entry point, which is no reference, meets the file already
// included by the import, which is one.
#[test]
fn a_type_library_entry_after_an_import_with_another_casing_reports_ts1261() {
    let files = [
        (
            "/src/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"noEmit":true,"typeRoots":["./types"],"types":["foo"]},"files":["a.ts"]}"#,
        ),
        ("/src/a.ts", r#"import { foo } from "./types/Foo/index"; export const x = foo;"#),
        ("/src/types/foo/index.d.ts", "export declare const foo: number;"),
    ];
    assert_eq!(include_diagnostic_codes_of(&files, "/src/a.ts"), [1261]);
}
