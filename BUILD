load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

rust_binary(
    name = "lim",
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    deps = ["@fly//fly"] + all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    env = {
        "RUST_LOG": "info",
    },
)

rust_test(
    name = "unit_test",
    crate = ":lim",
    env = {
        "RUST_LOG": "info",
        "RUST_BACKTRACE": "1",
    },
    aliases = aliases(
        normal_dev = True,
        proc_macro_dev = True,
    ),
    deps = all_crate_deps(
        normal_dev = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro_dev = True,
    ),
)
