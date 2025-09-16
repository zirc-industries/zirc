mod common;
mod repl;

fn main() {
    repl::start_repl_with_backend(repl::Backend::Interp);
}
