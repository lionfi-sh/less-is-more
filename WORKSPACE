load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

load(":fly_machine.bzl", "fly_machine_api")
fly_machine_api(name = "fly")

# To find additional information on this release or newer ones visit:
# https://github.com/bazelbuild/rules_rust/releases
http_archive(
    name = "rules_rust",
    integrity = "sha256-Weev1uz2QztBlDA88JX6A1N72SucD1V8lBsaliM0TTg=",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.48.0/rules_rust-v0.48.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rust_register_toolchains(
    edition = "2018",
    versions = [
        "1.82.0",
        "nightly/2024-10-17",
    ],
)

rules_rust_dependencies()

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies()

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//:Cargo.lock",
    lockfile = "//:Cargo.Bazel.lock",
    manifests = [
        "//:Cargo.toml",
        # Have to double import this to force fly to be built
        "//:fly/Cargo.toml",
        "@fly//fly:Cargo.toml"
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()
