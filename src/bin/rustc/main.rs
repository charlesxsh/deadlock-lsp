#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use std::env;
use deadlock_lsp::analysis::analyze;
use deadlock_lsp::utils;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use rustc_session::{early_error, config};
use rustc_session::config::ErrorOutputType;
use std::process;
use log::info;


/// Exit status code used for successful compilation and help output.
pub const EXIT_SUCCESS: i32 = 0;

/// Exit status code used for compilation failures and invalid flags.
pub const EXIT_FAILURE: i32 = 1;

fn main() {
    pretty_env_logger::init();

    let result = rustc_driver::catch_fatal_errors(move || {
        let mut rustc_args = env::args_os()
            .enumerate()
            .map(|(i, arg)| {
                arg.into_string().unwrap_or_else(|arg| {
                    early_error(
                        ErrorOutputType::default(),
                        &format!("Argument {} is not valid Unicode: {:?}", i, arg),
                    )
                })
            })
            .collect::<Vec<_>>();

        if let Some(sysroot) = utils::compile_time_sysroot() {
            let sysroot_flag = "--sysroot";
            if !rustc_args.iter().any(|e| e == sysroot_flag) {
                // We need to overwrite the default that librustc would compute.
                rustc_args.push(sysroot_flag.to_owned());
                rustc_args.push(sysroot);
            }
        }

        
        let always_encode_mir = "-Zalways_encode_mir";
        if !rustc_args.iter().any(|e| e == always_encode_mir) {
            // Get MIR code for all code related to the crate (including the dependencies and standard library)
            rustc_args.push(always_encode_mir.to_owned());
        }

        //Add this to support analyzing no_std libraries
        rustc_args.push("-Clink-arg=-nostartfiles".to_owned());

        // Disable unwind to simplify the CFG
        rustc_args.push("-Cpanic=abort".to_owned());
        rustc_args.push("-Zmir-opt-level=0".to_owned());
        //info!("{:?}", rustc_args);
        let mut callbacks = CompilerCallbacks{};
        let run_compiler = rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks);
        run_compiler.run()
        
    })
    .and_then(|result| result);

    let exit_code = match result {
        Ok(_) => EXIT_SUCCESS,
        Err(_) => EXIT_FAILURE,
    };

    process::exit(exit_code);
}

struct CompilerCallbacks {}

impl rustc_driver::Callbacks for CompilerCallbacks {


    fn after_analysis<'tcx>(&mut self, compiler: &interface::Compiler, queries: &'tcx rustc_interface::Queries<'tcx>) -> rustc_driver::Compilation {
        queries
            .global_ctxt()
            .unwrap()
            .peek_mut()
            .enter(|tcx| self.run_analysis(compiler, tcx));
        rustc_driver::Compilation::Continue
    }

}

impl CompilerCallbacks {
    fn run_analysis(&self, compiler: &interface::Compiler, tcx: TyCtxt) {
        match analyze(tcx) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("analyze failed: {}", err);
            },
        }
    }
}

