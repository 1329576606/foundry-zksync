use crate::utils::generate_large_contract;
use foundry_config::Config;
use foundry_test_utils::{forgetest, snapbox::IntoData, str, util::OutputExt};
use globset::Glob;
use regex::Regex;

forgetest_init!(can_parse_build_filters, |prj, cmd| {
    prj.clear();

    cmd.args(["build", "--names", "--skip", "tests", "scripts"]).assert_success().stdout_eq(str![
        [r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!
  compiler version: [..]
    - Counter

"#]
    ]);
});

forgetest!(throws_on_conflicting_args, |prj, cmd| {
    prj.clear();

    cmd.args(["compile", "--format-json", "--quiet"]).assert_failure().stderr_eq(str![[r#"
error: the argument '--json' cannot be used with '--quiet'

Usage: forge[..] build --json [PATHS]...

For more information, try '--help'.

"#]]);
});

// tests that json is printed when --format-json is passed
forgetest!(compile_json, |prj, cmd| {
    prj.add_source(
        "jsonError",
        r"
contract Dummy {
    uint256 public number;
    function something(uint256 newNumber) public {
        number = newnumber; // error here
    }
}
",
    )
    .unwrap();

    // set up command
    cmd.args(["compile", "--format-json"]).assert_success().stdout_eq(str![[r#"
{
  "errors": [
    {
      "sourceLocation": {
        "file": "src/jsonError.sol",
        "start": 184,
        "end": 193
      },
      "type": "DeclarationError",
      "component": "general",
      "severity": "error",
      "errorCode": "7576",
      "message": "Undeclared identifier. Did you mean \"newNumber\"?",
      "formattedMessage": "DeclarationError: Undeclared identifier. Did you mean \"newNumber\"?\n [FILE]:7:18:\n  |\n7 |         number = newnumber; // error here\n  |                  ^^^^^^^^^\n\n"
    }
  ],
  "sources": {},
  "contracts": {},
  "build_infos": "{...}"
}
"#]].is_json());
});

forgetest!(initcode_size_exceeds_limit, |prj, cmd| {
    prj.add_source("LargeContract", generate_large_contract(5450).as_str()).unwrap();
    cmd.args(["build", "--sizes"]).assert_failure().stdout_eq(str![
        r#"
...
| Contract     | Runtime Size (B) | Initcode Size (B) | Runtime Margin (B) | Initcode Margin (B) |
|--------------|------------------|-------------------|--------------------|---------------------|
| HugeContract |              202 |            49,359 |             24,374 |                -207 |
...
"#
    ]);
});

forgetest!(initcode_size_limit_can_be_ignored, |prj, cmd| {
    prj.add_source("LargeContract", generate_large_contract(5450).as_str()).unwrap();
    cmd.args(["build", "--sizes", "--ignore-eip-3860"]).assert_success().stdout_eq(str![
        r#"
...
| Contract     | Runtime Size (B) | Initcode Size (B) | Runtime Margin (B) | Initcode Margin (B) |
|--------------|------------------|-------------------|--------------------|---------------------|
| HugeContract |              202 |            49,359 |             24,374 |                -207 |
...
"#
    ]);
});

// tests build output is as expected
forgetest_init!(exact_build_output, |prj, cmd| {
    cmd.args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

// tests build output is as expected
forgetest_init!(build_sizes_no_forge_std, |prj, cmd| {
    cmd.args(["build", "--sizes"]).assert_success().stdout_eq(str![
        r#"
...
| Contract | Runtime Size (B) | Initcode Size (B) | Runtime Margin (B) | Initcode Margin (B) |
|----------|------------------|-------------------|--------------------|---------------------|
| Counter  |              247 |               277 |             24,329 |              48,875 |
...
"#
    ]);
});

// tests build output is as expected in zksync mode
forgetest_init!(test_zk_build_sizes, |prj, cmd| {
    cmd.args(["build", "--sizes", "--zksync", "--evm-version", "shanghai"]);
    let stdout = cmd.assert_success().get_output().stdout_lossy();
    let pattern =
        Regex::new(r"\|\s*Counter\s*\|\s*800\s*\|\s*800\s*\|\s*450,199\s*\|\s*450,199\s*\|")
            .unwrap();

    assert!(pattern.is_match(&stdout), "Unexpected size output:\n{stdout}");
});

// tests that skip key in config can be used to skip non-compilable contract
forgetest_init!(test_can_skip_contract, |prj, cmd| {
    prj.add_source(
        "InvalidContract",
        r"
contract InvalidContract {
    some_invalid_syntax
}
",
    )
    .unwrap();

    prj.add_source(
        "ValidContract",
        r"
contract ValidContract {}
",
    )
    .unwrap();

    let config =
        Config { skip: vec![Glob::new("src/InvalidContract.sol").unwrap()], ..Default::default() };
    prj.write_config(config);

    cmd.args(["build"]).assert_success();
});
