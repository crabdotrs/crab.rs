if exists("b:current_syntax")
  finish
endif

syn keyword crabKeyword var final const async await
syn keyword crabKeyword if else for while do switch case default
syn keyword crabKeyword break continue return throw try catch finally
syn keyword crabKeyword class abstract sealed base interface extends implements with
syn keyword crabKeyword mixin extension on enum typedef
syn keyword crabKeyword import export part partOf library
syn keyword crabKeyword static get set factory override required
syn keyword crabKeyword this super new is as in
syn keyword crabType int double bool String void dynamic Never
syn keyword crabType List Map Set Future Stream Result Option
syn keyword crabConstant true false null

syn match crabIdentifier /\<[a-zA-Z_][a-zA-Z0-9_]*\>/
syn match crabNumber /\<\d\+\>/
syn match crabFloat /\<\d\+\.\d*\>/
syn match crabHex /\<0x[0-9a-fA-F]\+\>/

syn region crabString start=/"/ skip=/\\"/ end=/"/
syn region crabString start=/'/ skip=/\\'/ end=/'/
syn region crabString start=/"""/ end=/"""/
syn region crabString start=/'''/ end=/'''/

syn match crabComment /\/\/.*$/
syn region crabComment start=/\/\*/ end=/\*\//

syn match crabOperator /[+=\-*/%&|^~<>!?:]/
syn match crabOperator /\*\*/
syn match crabOperator /\/\//
syn match crabOperator /\*=/
syn match crabOperator /\/=/
syn match crabOperator /+=/
syn match crabOperator /-=/
syn match crabOperator /%=/
syn match crabOperator /&=/
syn match crabOperator /|=/
syn match crabOperator /^=/
syn match crabOperator /\?\?/
syn match crabOperator /\?\?=/
syn match crabOperator /=>/
syn match crabOperator /->/
syn match crabOperator /++/
syn match crabOperator /--/
syn match crabOperator /&&/
syn match crabOperator /||/
syn match crabOperator /<=/
syn match crabOperator />=/
syn match crabOperator /==/
syn match crabOperator /!=/
syn match crabOperator /</
syn match crabOperator />/
syn match crabOperator /<<</
syn match crabOperator />>>/

hi def link crabKeyword Keyword
hi def link crabType Type
hi def link crabConstant Constant
hi def link crabIdentifier Identifier
hi def link crabNumber Number
hi def link crabFloat Float
hi def link crabHex Number
hi def link crabString String
hi def link crabComment Comment
hi def link crabOperator Operator

let b:current_syntax = "crab"
