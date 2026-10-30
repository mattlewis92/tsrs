// tsrs-only: where the watch keeps a full build's symlink cache (watcher.rs `SharedData`).

use rustc_hash::FxHashMap;
use tsrs_core::arena::Region;

use super::{Scenario, ScenarioFile, ScenarioOp, apply_ops, execute_command, new_test_sys};

const LIB_FILE: &str = "/home/src/tslibs/TS/Lib/lib.esnext.full.d.ts";
const LIB: &str = "/// <reference no-default-lib=\"true\"/>\ninterface Boolean {}\ninterface Function {}\ninterface CallableFunction {}\ninterface NewableFunction {}\ninterface IArguments {}\ninterface Number {}\ninterface Object {}\ninterface RegExp {}\ninterface String {}\ninterface Array<T> { length: number; [n: number]: T; }\ninterface ReadonlyArray<T> {}\n";
const ROOT: &str = "/home/src/workspaces/project";

fn file(path: &str, content: &str) -> ScenarioFile {
    ScenarioFile {
        path: format!("{ROOT}/{path}"),
        content: content.to_string(),
        symlink: String::new(),
    }
}

// `Program::get_symlink_cache` stores the cache with the program's processed-file data, which every program updated
// from the same full build shares, in the data's owner region. A full build of the watch gives its data an owner,
// freed with the data, as the language server does: otherwise the cache goes to the arena of the thread that first
// asks, which is never freed, and each full build whose cache was filled (declaration emit naming a dependency's type)
// kept one.
#[test]
fn a_full_builds_symlink_cache_lives_in_a_region_freed_with_the_build() {
    let mut files = vec![
        file(
            "tsconfig.json",
            r#"{ "compilerOptions": { "target": "esnext", "declaration": true, "rootDir": "src", "outDir": "dist", "module": "nodenext", "moduleResolution": "nodenext", "skipLibCheck": true }, "include": ["src"] }"#,
        ),
        file("node_modules/dep/package.json", r#"{ "name": "dep", "types": "index.d.ts" }"#),
        file(
            "node_modules/dep/index.d.ts",
            "export interface Thing { a: number }\nexport declare function make(): Thing;\n",
        ),
        // The inferred type of `x` names `Thing`, so declaration emit asks for a module specifier.
        file("src/a.ts", "import { make } from \"dep\";\nexport const x = make();\n"),
    ];
    files.push(ScenarioFile {
        path: LIB_FILE.to_string(),
        content: LIB.to_string(),
        symlink: String::new(),
    });
    let test = Scenario {
        baseline: String::new(),
        args: vec!["-w".to_string()],
        cwd: ROOT.to_string(),
        env: FxHashMap::default(),
        output_is_tty: None,
        ignore_case: false,
        windows_style_root: String::new(),
        files,
        default_libs: vec![LIB_FILE.to_string()],
        edits: Vec::new(),
        signature_job: false,
    };
    let sys = new_test_sys(&test);
    let mut baseline = String::new();
    // Snapshots the file system the edits' changes are found against (`changed_paths`).
    sys.baseline_fs_with_diff(&mut baseline);
    let mut watcher = execute_command(sys, &mut baseline, &test.args).expect("a watch session");
    assert!(!sys.get_output(true).contains("error"), "{}", sys.get_output(true));
    // Holds no handle to the region: the watch must be able to free it.
    let cache_in_a_region = |program: &'static tsrs_compiler::Program| {
        Region::containing(program.get_symlink_cache().addr()).is_some()
    };

    let first = watcher.program().expect("the first build's program");
    assert!(cache_in_a_region(first), "the first build's cache is in a region");

    // A new root file is a full build.
    apply_ops(
        sys,
        &[ScenarioOp {
            op: "write".to_string(),
            path: format!("{ROOT}/src/b.ts"),
            content: "export const y = 1;\n".to_string(),
            target: String::new(),
        }],
    );
    sys.mock_watch_backend.get().unwrap().send_changed_paths(&sys.changed_paths());
    watcher.do_cycle();
    let second = watcher.program().expect("the second build's program");
    assert!(!std::ptr::eq(first, second), "adding a root file builds a new program");
    assert!(cache_in_a_region(second), "the second build's cache is in a region");
}
