```.
├── bench
│   └── README.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── crab.rs
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── crab-cli
│   │   ├── Cargo.toml
│   │   └── src
│   ├── crab-codegen
│   │   ├── Cargo.toml
│   │   └── src
│   ├── crab-core
│   │   ├── Cargo.toml
│   │   └── src
│   ├── crab-ffi
│   │   ├── Cargo.toml
│   │   └── src
│   ├── crab-lexer
│   │   ├── Cargo.toml
│   │   └── src
│   ├── crab-parser
│   │   ├── Cargo.toml
│   │   └── src
│   ├── run_test.sh
│   ├── target
│   │   ├── CACHEDIR.TAG
│   │   ├── debug
│   │   ├── flycheck0
│   │   └── release
│   ├── test_fix.sh
│   ├── test_parse.rs
│   └── tests
│       ├── c_interop_e2e.rs
│       └── transpile_and_run.rs
├── Dockerfile
├── editor
│   ├── neovim
│   │   └── crab.nvim
│   └── vscode
│       └── crab-language-support
├── examples
│   ├── 01_hello_world.crab
│   ├── 02_variables.crab
│   ├── 03_null_safety.crab
│   ├── 04_functions.crab
│   ├── 05_control_flow.crab
│   ├── 06_classes.crab
│   ├── 07_generics.crab
│   ├── 08_collections.crab
│   ├── 09_async.crab
│   ├── 10_error_handling.crab
│   ├── 11_operators.crab
│   ├── 12_extensions.crab
│   ├── 13_strings.crab
│   ├── cookie_shop_api
│   │   ├── crab.toml
│   │   ├── docs
│   │   ├── README.md
│   │   ├── src
│   │   └── tests
│   ├── hello.crab
│   ├── hello_test
│   │   ├── crab.toml
│   │   └── src
│   ├── null_safety.crab
│   ├── README.md
│   ├── test_01
│   │   ├── crab.toml
│   │   └── src
│   ├── test_02
│   │   ├── crab.toml
│   │   └── src
│   ├── test_03
│   │   ├── crab.toml
│   │   └── src
│   ├── test_04
│   │   ├── crab.toml
│   │   └── src
│   ├── test_05
│   │   ├── crab.toml
│   │   └── src
│   ├── test_06
│   │   ├── crab.toml
│   │   └── src
│   ├── test_07
│   │   ├── crab.toml
│   │   └── src
│   ├── test_08
│   │   ├── crab.toml
│   │   └── src
│   ├── test_09
│   │   ├── crab.toml
│   │   └── src
│   ├── test_10
│   │   ├── crab.toml
│   │   └── src
│   ├── test_basic
│   │   ├── crab.toml
│   │   └── src
│   ├── test_const.crab
│   ├── test_hello
│   │   └── main.crab
│   ├── test_index
│   │   ├── crab.toml
│   │   └── src
│   ├── test_is
│   │   ├── crab.toml
│   │   └── src
│   ├── test_minimal
│   │   ├── crab.toml
│   │   └── src
│   ├── test_null_safety
│   │   ├── crab.toml
│   │   └── src
│   ├── test_project
│   │   ├── crab.toml
│   │   └── src
│   ├── test_switch
│   │   ├── crab.toml
│   │   └── src
│   ├── test_switch2
│   │   ├── crab.toml
│   │   └── src
│   ├── test_switch.crab
│   ├── test_temp
│   │   ├── crab.toml
│   │   └── src
│   ├── test_todo_cmd
│   │   ├── crab.toml
│   │   └── src
│   ├── todo_list
│   │   ├── crab.toml
│   │   ├── README.md
│   │   └── src
│   └── variables.crab
├── features.md
├── language
│   ├── colors
│   │   └── palette.md
│   ├── fonts
│   └── logo
│       ├── crab-logo-dark.svg
│       ├── crab-logo.svg
│       └── crab.rs.png
├── LICENSE
├── problems.md
├── README.md
├── scripts
│   ├── build.sh
│   ├── ci
│   │   ├── install-deps.sh
│   │   └── run-tests.sh
│   ├── clean.sh
│   ├── docker
│   │   ├── build-multiarch.sh
│   │   ├── build.sh
│   │   ├── dev.sh
│   │   ├── push.sh
│   │   ├── run.sh
│   │   ├── shell.sh
│   │   └── test.sh
│   ├── docker-compose.yml
│   ├── fmt.sh
│   ├── generate-docs.sh
│   ├── install-local.sh
│   ├── lint.sh
│   ├── package-release.sh
│   ├── publish.sh
│   └── test.sh
├── templates
│   ├── crab-actix
│   │   ├── crab.toml.template
│   │   ├── README.md.template
│   │   └── src
│   ├── crab-lib
│   │   ├── crab.toml.template
│   │   ├── README.md.template
│   │   └── src
│   └── crab-new
│       ├── crab.toml.template
│       ├── README.md.template
│       └── src
└── tools
    ├── crab-fmt
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   ├── src
    │   └── target
    ├── crab-lint
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   ├── src
    │   └── target
    ├── crab-lsp
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   ├── src
    │   └── target
    └── treesitter-crab
        ├── binding.gyp
        ├── bindings
        ├── build.zig
        ├── build.zig.zon
        ├── Cargo.lock
        ├── Cargo.toml
        ├── CMakeLists.txt
        ├── go.mod
        ├── grammar.js
        ├── Makefile
        ├── package.json
        ├── Package.swift
        ├── pom.xml
        ├── pyproject.toml
        ├── setup.py
        ├── target
        └── tree-sitter.json

102 directories, 117 files
```
