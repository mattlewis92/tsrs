// tsrs-only: what the watch does with the configs it reloads (watcher.rs `reload_config`).

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

// A file added to an `include`d directory reloads the config's file names into a new config, which the full build
// after it uses. The config it replaces is freed once no program uses it, by the next full build; it was kept for the
// session, with its file names, so each full build kept a few hundred bytes per program file.
#[test]
fn a_reloaded_config_is_freed_by_the_full_build_after_the_one_that_replaced_it() {
    let test = Scenario {
        baseline: String::new(),
        args: vec!["-w".to_string()],
        cwd: ROOT.to_string(),
        env: FxHashMap::default(),
        output_is_tty: None,
        ignore_case: false,
        windows_style_root: String::new(),
        files: vec![
            file("tsconfig.json", r#"{ "compilerOptions": { "target": "esnext", "noEmit": true }, "include": ["src"] }"#),
            file("src/a.ts", "export const x = 1;\n"),
            ScenarioFile { path: LIB_FILE.to_string(), content: LIB.to_string(), symlink: String::new() },
        ],
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
    let mut add_root_file = |name: &str| {
        apply_ops(
            sys,
            &[ScenarioOp {
                op: "write".to_string(),
                path: format!("{ROOT}/src/{name}"),
                content: "export const y = 1;\n".to_string(),
                target: String::new(),
            }],
        );
        sys.mock_watch_backend.get().unwrap().send_changed_paths(&sys.changed_paths());
        watcher.do_cycle();
        let config = watcher.config().expect("the watch's config");
        assert!(config.file_names().iter().any(|f| f.ends_with(name)), "{name} is a root file");
        std::ptr::from_ref(config).addr()
    };
    // Holds no handle to a region: the watch must be able to free it.
    let in_a_live_region = |addr: usize| Region::containing(addr).is_some();

    let first = add_root_file("b.ts");
    assert!(in_a_live_region(first), "the reloaded config is in a region");
    let second = add_root_file("c.ts");
    assert_ne!(first, second, "adding a root file reloads the config");
    assert!(in_a_live_region(second), "the second reloaded config is in a region");
    assert!(!in_a_live_region(first), "the full build that used the second config freed the first");
}
