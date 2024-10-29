def _fly_machine_impl(ctx):
    print("\n\nLoading...\n\n")
    """
    Implementation of the fly_machine repository rule.
    
    This rule runs a Docker container using the OpenAPI Generator CLI to generate
    Go client code from a specified OpenAPI specification.
    """
    
    result = ctx.execute([
        "docker", "run", "--rm",
        "-v", str(ctx.workspace_root) + ":/local",
        "openapitools/openapi-generator-cli",
        "generate",
        "-i", "/local/fly-machines-spec.json",
        "-g", "rust",
        "--skip-validate-spec",
        "-o", "/local/fly",
    ])
    print("Result: " + result.stdout)
    print("Result err: " + result.stderr)
    print("From: " + str(ctx.workspace_root) + "/fly")
    print("To: " + ctx.execute(["pwd"]).stdout.strip() + "/fly")
    result = ctx.execute(["cp", "-r", "-v", str(ctx.workspace_root) + "/fly", ctx.execute(["pwd"]).stdout.strip() + "/fly"])
    print(result.stdout)
    print(result.stderr)
    
    ctx.file("fly/BUILD", """
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")
load("@rules_rust//crate_universe:defs.bzl", "crates_vendor", "crate")

rust_library(
    name = "{name}",
    visibility = ["//visibility:public"],
    srcs = glob(["src/**/*.rs"]),
    edition = "2021",
    deps = all_crate_deps(
        normal = True,
    ),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
)""".format(name="fly", path=str(ctx.workspace_root))        
    )

    ctx.file("BUILD", """
exports_files(["fly/Cargo.toml"])
""")
                        
# Define the repository rule
fly_machine_api = repository_rule(
    implementation = _fly_machine_impl,
    local = True,
)
