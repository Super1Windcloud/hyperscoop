#![allow(
    dead_code,
    clippy::assign_op_pattern,
    clippy::bind_instead_of_map,
    clippy::bool_comparison,
    clippy::borrowed_box,
    clippy::clone_on_copy,
    clippy::explicit_auto_deref,
    clippy::collapsible_if,
    clippy::cmp_owned,
    clippy::derivable_impls,
    clippy::disallowed_names,
    clippy::double_ended_iterator_last,
    clippy::enum_variant_names,
    clippy::expect_fun_call,
    clippy::extra_unused_lifetimes,
    clippy::format_in_format_args,
    clippy::get_first,
    clippy::if_same_then_else,
    clippy::iter_cloned_collect,
    clippy::iter_next_slice,
    clippy::items_after_test_module,
    clippy::iter_count,
    clippy::let_and_return,
    clippy::let_unit_value,
    clippy::len_zero,
    clippy::manual_map,
    clippy::manual_strip,
    clippy::module_inception,
    clippy::needless_borrow,
    clippy::needless_borrows_for_generic_args,
    clippy::needless_bool,
    clippy::needless_match,
    clippy::needless_question_mark,
    clippy::needless_return,
    clippy::needless_range_loop,
    clippy::needless_ifs,
    clippy::new_without_default,
    clippy::nonminimal_bool,
    clippy::op_ref,
    clippy::ptr_arg,
    clippy::question_mark,
    clippy::redundant_closure_call,
    clippy::redundant_pattern_matching,
    clippy::regex_creation_in_loops,
    clippy::single_match,
    clippy::to_string_in_format_args,
    clippy::unnecessary_map_or,
    clippy::unnecessary_to_owned,
    clippy::unnecessary_unwrap,
    clippy::unused_enumerate_index,
    clippy::unused_io_amount,
    clippy::unwrap_or_default,
    clippy::useless_conversion,
    clippy::useless_vec
)]

pub mod buckets;
pub mod cat;
pub mod init_env;
pub mod list;
pub mod merge;
pub mod search;
pub mod utils;
use crate::init_env::HyperScoopGlobal;
pub use init_env::HyperScoop;
pub use list::{display_app_info, list_specific_installed_apps};
pub use std::process::exit;

pub mod cache;
pub mod export;
pub mod home;
pub mod info;
pub mod manifest;

pub mod config;
pub mod import;
pub mod install;
pub mod reset;
pub mod shim;
pub mod uninstall;
pub mod update;

pub fn init_hyperscoop() -> Result<HyperScoop, anyhow::Error> {
    let hyperscoop = HyperScoop::new();
    Ok(hyperscoop)
}

pub fn init_hyperscoop_global() -> Result<HyperScoopGlobal, anyhow::Error> {
    let hyperscoop = HyperScoopGlobal::new();
    Ok(hyperscoop)
}
