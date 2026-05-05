/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "crab",

  rules: {
    source_file: ($) => repeat($._top_level_item),

    _top_level_item: ($) =>
      choice(
        $.import_statement,
        $.export_statement,
        $.function_declaration,
        $.class_declaration,
        $.mixin_declaration,
        $.extension_declaration,
        $.variable_declaration,
        $.constant_declaration,
        $.type_alias,
        $.enum_declaration,
        $.cblock_declaration,
      ),

    import_statement: ($) =>
      seq(
        "import",
        choice($.string_literal, $.identifier),
        optional(seq("as", $.identifier)),
        optional(seq("show", commaSep1($.identifier))),
        optional(seq("hide", commaSep1($.identifier))),
        optional("deferred"),
        ";",
      ),

    export_statement: ($) =>
      seq("export", choice($.string_literal, $.identifier), ";"),

    cblock_declaration: ($) => seq("cblock", $.block),

    function_declaration: ($) =>
      seq(
        optional($.type),
        $.identifier,
        $.formal_parameters,
        optional("async"),
        optional("sync"),
        choice($.function_body, seq("=>", $._expression, ";")),
      ),

    function_body: ($) => $.block,

    class_declaration: ($) =>
      seq(
        repeat(choice("abstract", "sealed", "base", "final", "interface")),
        "class",
        $.identifier,
        optional(seq("<", commaSep1($.identifier), ">")),
        optional(seq("extends", $.type)),
        optional(seq("implements", commaSep1($.type))),
        optional(seq("with", commaSep1($.type))),
        $.class_body,
      ),

    class_body: ($) =>
      seq(
        "{",
        repeat(
          choice(
            $.field_declaration,
            $.constructor_declaration,
            $.method_declaration,
            $.getter_declaration,
            $.setter_declaration,
          ),
        ),
        "}",
      ),

    mixin_declaration: ($) =>
      seq("mixin", $.identifier, optional(seq("on", $.type)), $.class_body),

    extension_declaration: ($) =>
      seq("extension", optional($.identifier), "on", $.type, $.class_body),

    enum_declaration: ($) =>
      seq("enum", $.identifier, "{", commaSep1($.enum_value), "}"),

    enum_value: ($) =>
      seq($.identifier, optional(seq("(", $._expression, ")"))),

    field_declaration: ($) =>
      seq(
        optional("static"),
        optional("final"),
        $.type,
        $.identifier,
        optional(seq("=", $._expression)),
        ";",
      ),

    constructor_declaration: ($) =>
      seq(
        optional($.identifier),
        $.formal_parameters,
        optional(seq(":", seq("super", $.actual_parameters))),
        optional($.block),
        optional(";"),
      ),

    method_declaration: ($) =>
      seq(
        optional($.type),
        $.identifier,
        $.formal_parameters,
        optional("async"),
        optional("sync"),
        choice($.block, seq("=>", $._expression, optional(";"))),
      ),

    getter_declaration: ($) =>
      seq(
        optional($.type),
        "get",
        $.identifier,
        seq("=>", $._expression, optional(";")),
      ),

    setter_declaration: ($) =>
      seq(
        optional($.type),
        "set",
        $.identifier,
        seq("(", $.type, $.identifier, ")"),
        $.block,
      ),

    variable_declaration: ($) =>
      seq(
        choice("var", "final"),
        optional(seq($.identifier, ":", $.type)),
        $.identifier,
        optional(seq("=", $._expression)),
        ";",
      ),

    constant_declaration: ($) =>
      seq("const", $.type, $.identifier, "=", $._expression, ";"),

    type_alias: ($) => seq("typedef", $.identifier, "=", $.type, ";"),

    formal_parameters: ($) => seq("(", commaSep($.formal_parameter), ")"),

    formal_parameter: ($) =>
      seq(
        optional("required"),
        $.type,
        $.identifier,
        optional(seq("=", $._expression)),
      ),

    actual_parameters: ($) => seq("(", commaSep($._expression), ")"),

    block: ($) => seq("{", repeat($._statement), "}"),

    _statement: ($) =>
      choice(
        $.variable_declaration,
        $.constant_declaration,
        $.expression_statement,
        $.if_statement,
        $.for_statement,
        $.while_statement,
        $.do_statement,
        $.switch_statement,
        $.try_statement,
        $.return_statement,
        $.break_statement,
        $.continue_statement,
        $.throw_statement,
        $.block,
      ),

    expression_statement: ($) => seq($._expression, ";"),

    if_statement: ($) =>
      seq(
        "if",
        "(",
        $._expression,
        ")",
        $._statement,
        optional(seq("else", $._statement)),
      ),

    for_statement: ($) =>
      seq(
        "for",
        "(",
        optional(choice($.variable_declaration, $.expression_statement)),
        optional($._expression),
        ";",
        optional($._expression),
        ")",
        $._statement,
      ),

    while_statement: ($) => seq("while", "(", $._expression, ")", $._statement),

    do_statement: ($) =>
      seq("do", $._statement, "while", "(", $._expression, ")", ";"),

    switch_statement: ($) =>
      seq(
        "switch",
        "(",
        $._expression,
        ")",
        "{",
        repeat($.switch_case),
        optional($.default_case),
        "}",
      ),

    switch_case: ($) =>
      seq(
        "case",
        $._expression,
        optional(seq("when", $._expression)),
        ":",
        repeat($._statement),
      ),

    default_case: ($) => seq("default", ":", repeat($._statement)),

    try_statement: ($) =>
      seq("try", $.block, repeat($.catch_clause), optional($.finally_clause)),

    catch_clause: ($) =>
      seq("catch", "(", optional($.type), optional($.identifier), ")", $.block),

    finally_clause: ($) => seq("finally", $.block),

    return_statement: ($) => seq("return", optional($._expression), ";"),

    break_statement: ($) => seq("break", optional($.identifier), ";"),

    continue_statement: ($) => seq("continue", optional($.identifier), ";"),

    throw_statement: ($) => seq("throw", $._expression, ";"),

    _expression: ($) =>
      choice(
        $.assignment_expression,
        $.conditional_expression,
        $.binary_expression,
        $.unary_expression,
        $.postfix_expression,
        $.primary_expression,
        $.await_expression,
        $.is_expression,
        $.as_expression,
        $.null_assertion,
        $.null_aware_expression,
      ),

    assignment_expression: ($) =>
      prec.right(
        1,
        seq(
          $._expression,
          choice("=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "??="),
          $._expression,
        ),
      ),

    conditional_expression: ($) =>
      prec.right(2, seq($._expression, "?", $._expression, ":", $._expression)),

    binary_expression: ($) =>
      choice(
        prec.left(3, seq($._expression, "||", $._expression)),
        prec.left(4, seq($._expression, "&&", $._expression)),
        prec.left(5, seq($._expression, "|", $._expression)),
        prec.left(6, seq($._expression, "^", $._expression)),
        prec.left(7, seq($._expression, "&", $._expression)),
        prec.left(8, seq($._expression, choice("==", "!="), $._expression)),
        prec.left(
          9,
          seq($._expression, choice("<", ">", "<=", ">="), $._expression),
        ),
        prec.left(10, seq($._expression, choice("<<", ">>"), $._expression)),
        prec.left(11, seq($._expression, choice("+", "-"), $._expression)),
        prec.left(
          12,
          seq($._expression, choice("*", "/", "%", "~/"), $._expression),
        ),
      ),

    unary_expression: ($) =>
      prec(13, seq(choice("-", "!", "~", "++", "--"), $._expression)),

    postfix_expression: ($) =>
      prec(14, choice(seq($._expression, "++"), seq($._expression, "--"))),

    primary_expression: ($) =>
      choice(
        $.literal,
        $.identifier,
        $.this_expression,
        $.super_expression,
        $.new_expression,
        $.parenthesized_expression,
        $.list_literal,
        $.map_literal,
        $.function_expression,
        $.string_literal,
        $.string_interpolation,
      ),

    await_expression: ($) => seq("await", $._expression),

    is_expression: ($) => seq($._expression, "is", $.type),

    as_expression: ($) => seq($._expression, "as", $.type),

    null_assertion: ($) => seq($._expression, "!"),

    null_aware_expression: ($) =>
      seq(
        $._expression,
        choice("?.", seq("?.", $.identifier, $.actual_parameters)),
        choice(
          seq($.identifier, optional($.actual_parameters)),
          seq("[", $._expression, "]"),
        ),
      ),

    this_expression: ($) => "this",

    super_expression: ($) => seq("super", optional(seq(".", $.identifier))),

    new_expression: ($) =>
      seq(
        "new",
        $.identifier,
        optional(seq(".", $.identifier)),
        $.actual_parameters,
      ),

    parenthesized_expression: ($) => seq("(", $._expression, ")"),

    list_literal: ($) => seq("[", commaSep($._expression), "]"),

    map_literal: ($) =>
      seq("{", commaSep(seq($._expression, ":", $._expression)), "}"),

    function_expression: ($) =>
      seq($.formal_parameters, choice($.block, seq("=>", $._expression))),

    string_interpolation: ($) =>
      seq(
        '"',
        repeat(choice($.string_content, seq("${", $._expression, "}"))),
        '"',
      ),

    string_content: ($) => /[^"$\\]+/,

    string_literal: ($) =>
      choice(
        seq('"', /[^"\\]*/, '"'),
        seq("'", /[^'\\]*/, "'"),
        seq('"""', /[^"]*"""/),
        seq("'''", /[^']*'''/),
      ),

    literal: ($) => choice($.numeric_literal, $.boolean_literal, "null"),

    numeric_literal: ($) => choice(/\d+/, /\d+\.\d*/, /0x[0-9a-fA-F]+/),

    boolean_literal: ($) => choice("true", "false"),

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    type: ($) =>
      choice(
        "int",
        "double",
        "bool",
        "String",
        "void",
        "dynamic",
        "Never",
        seq($.identifier, optional(seq("<", commaSep1($.type), ">"))),
        seq($.type, "?"),
        seq("List", "<", $.type, ">"),
        seq("Map", "<", $.type, ",", $.type, ">"),
        seq("Set", "<", $.type, ">"),
        seq("Future", "<", $.type, ">"),
        seq("Stream", "<", $.type, ">"),
        seq("Result", "<", $.type, ",", $.type, ">"),
        seq("Option", "<", $.type, ">"),
      ),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)));
}

function commaSep(rule) {
  return optional(commaSep1(rule));
}
