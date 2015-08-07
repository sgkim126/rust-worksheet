#![feature(rustc_private)]

#[macro_use]
extern crate log;

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_resolve;
extern crate serialize;
extern crate syntax;

use rustc::ast_map;
use rustc::middle::ty;
use rustc::session::build_session;
use rustc::session::config;
use rustc_driver::driver;
use rustc_resolve::MakeGlobMap;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;
use syntax::diagnostics::registry;
use syntax::feature_gate::UnstableFeatures;

use serialize::json;


fn main() {
    let mut sopts = config::basic_options();
    sopts.optimize = config::No;
    sopts.maybe_sysroot = Some(get_sysroot());
    sopts.crate_types = vec![config::CrateTypeDylib];
    sopts.unstable_features = UnstableFeatures::Allow;

    let local_crate_source_file = None;
    let registry = registry::Registry::new(&rustc::DIAGNOSTICS);

    let sess = build_session(sopts, local_crate_source_file, registry);
    let cfg = config::build_configuration(&sess);
    let input = config::Input::File(PathBuf::from("./sample.rs"));
    let krate = driver::phase_1_parse_input(&sess, cfg, &input);

    println!("{}", json::as_json(&krate));
//    let parse_sess = sess.parse_sess;

    let crate_name = "test";
    let addl_plugins = None;
    let krate = driver::phase_2_configure_and_expand(&sess, krate, crate_name, addl_plugins).expect("...");

    println!("{}", json::as_json(&krate));

    let mut forest = ast_map::Forest::new(krate);
    let arenas = ty::CtxtArenas::new();
    let ast_map = driver::assign_node_ids_and_map(&sess, &mut forest);

    let name = "worksheet".to_string();
    let make_glob_map = MakeGlobMap::No;

    let (_sess, _result) = driver::phase_3_run_analysis_passes(sess, ast_map, &arenas, name, make_glob_map, |_tcx, _analysis| {
    });
}

/// Runs `rustc` to ask for its sysroot path.
fn get_sysroot() -> PathBuf {
    let rustc = if cfg!(windows) { "rustc.exe" } else { "rustc" };

    let output = match Command::new(rustc).args(&["--print", "sysroot"]).output() {
        Ok(output) => output.stdout,
        Err(e) => panic!("failed to run rustc: {}", e),
    };

    let path = from_utf8(&output)
        .ok().expect("sysroot is not valid UTF-8").trim_right_matches(
            |c| c == '\r' || c == '\n');

    debug!("using sysroot: {:?}", path);

    PathBuf::from(path)
}
