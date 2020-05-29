extern crate lalrpop;


fn main() {
    #[cfg(feature = "rebuild")]
    lalrpop::Configuration::new().generate_in_source_tree().process_current_dir().unwrap();
}
