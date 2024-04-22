use std::ffi::{CStr, CString};
use std::ptr;

use crate::ast::*;
use crate::parser::env::Env;
use crate::util::{dynamic_str, OwningCStr};

type BoxedBlock = *mut AstBlock;
type BoxedFunction = *mut AstFunction;
type BoxedType = *mut AstType;
type BoxedVariable = *mut AstVariable;
type BoxedCStr = *mut libc::c_char;

pub struct Declaration {
    pub name: *mut libc::c_char,
    pub t: *mut AstType,
}

pub struct DirectFunctionDeclarator {
    pub name: *mut libc::c_char,
    pub params: Vec<*mut AstVariable>,
}

pub struct FunctionDeclarator {
    pub ptr_valence: libc::c_int,
    pub decl: DirectFunctionDeclarator,
}

pub struct Declarator {
    pub ptr_valence: libc::c_int,
    pub name: *mut libc::c_char,
}

struct BlockStatement {
    r#abstract: *mut AstBlock,
    body: *mut AstBlock,
}

enum PostfixOp {
    ArrayAccess(Box<AstExpr>),
    Call(Vec<Box<AstExpr>>),
    Dot(*mut libc::c_char),
    Arrow(*mut libc::c_char),
    Inc,
    Dec,
}

/* XXX */
unsafe fn strip_quotes(s: *const libc::c_char) -> BoxedCStr {
    assert_eq!(*s, b'"' as libc::c_char);
    let len = libc::strlen(s) - 2 + 1;
    let t = libc::malloc(len) as *mut libc::c_char;
    libc::memcpy(
        t as *mut libc::c_void,
        s.offset(1) as *const libc::c_void,
        len - 1,
    );
    *t.add(len - 1) = 0;
    t
}

unsafe fn variable_array_from_decl_vec(decls: Vec<Declaration>) -> Vec<*mut AstVariable> {
    decls
        .into_iter()
        .map(|decl| ast_variable_create(OwningCStr::new(decl.name), decl.t))
        .collect()
}

unsafe fn ast_from_vec(decls: Vec<Box<AstExternDecl>>) -> Box<Ast> {
    Box::new(Ast { decls })
}

peg::parser! {
pub grammar c_parser(env: &Env) for str {
    // A few bits cribbed from <https://github.com/vickenty/lang-c/blob/master/grammar.rustpeg>

    // Lists of elements.
    rule list0<T>(ex: rule<T>) -> Vec<T> = e:ex() ** _ { e }
    rule list1<T>(ex: rule<T>) -> Vec<T> = e:ex() ++ _ { e }
    rule cs0<T>(ex: rule<T>) -> Vec<T> = e:ex() ** (_ "," _) { e }
    rule cs1<T>(ex: rule<T>) -> Vec<T> = e:ex() ++ (_ "," _) { e }

    // Whitespace
    rule _() = quiet!{(newline() / [ ' ' | '\t'])*}
    rule newline() = "\r"? "\n" $(directive()?)
    rule directive() = "#" [^ '\n']*

    // Keywords
    rule end_of_token() = !['_' | 'a'..='z' | 'A'..='Z' | '0'..='9']
    rule K<T>(t: rule<T>) -> T = quiet! { e:t() end_of_token() { e } }

    // Identifiers
    rule identifier() -> BoxedCStr =
        n:quiet! { $(['_' | 'a'..='z' | 'A'..='Z'] ['_' | 'a'..='z' | 'A'..='Z' | '0'..='9']*) } {?
            if !env.reserved.contains(n) {
                let cstr = CString::new(n.to_string()).unwrap();
                Ok(unsafe { dynamic_str(cstr.as_ptr()) })
            } else {
                Err("identifier")
            }
        } /
        expected!("identifier")

    // Constants
    rule dec() = ['0'..='9']
    rule oct() = ['0'..='7']
    rule hex() = ['0'..='9' | 'a'..='f' | 'A'..='F']

    pub rule constant() -> Box<AstExpr> =
        numeric_constant() / character_constant()

    rule numeric_constant() -> Box<AstExpr> =
        n:quiet! { integer_number() } { unsafe { ast_expr_constant_create(n) } } /
        expected!("number")

    rule integer_number() -> libc::c_int =
        n:$(['1'..='9'] dec()* / "0") end_of_token() {
            parse_int(n)
        }

    rule character_constant() -> Box<AstExpr> =
        c:quiet! { $("'" character()+ "'") } {
            unsafe {
                ast_expr_constant_create_char(parse_char(c))
            }
        } /
        expected!("character")

    rule character() = [^ '\'' | '\\' | '\n'] / "\\" [^ '\n']

    // String literals
    rule string_literal() -> Box<AstExpr> =
        a:$("\"" string_char()* "\"") {
            unsafe {
                let cstr = CString::new(a.to_string()).unwrap();
                ast_expr_literal_create(strip_quotes(cstr.as_ptr()))
            }
        }

    rule string_char() = [^ '"' | '\\' | '\n'] / "\\" [^ '\n']

    // 6.5.1 Primary expression
    rule primary_expression() -> Box<AstExpr> =
        a:identifier() {
            unsafe { ast_expr_identifier_create(a) }
        } /
        "$" {
            unsafe { ast_expr_arbarg_create() }
        } /
        constant() /
        string_literal() /
        "(" _ a:expression() _ ")" { a }

    // 6.5.2 Postfix operators
    rule postfix_expression() -> Box<AstExpr> =
        a:primary_expression() ops:postfix_op()* {
            unsafe {
                let mut a = a;
                for op in ops {
                    a = match op {
                        PostfixOp::ArrayAccess(i) =>
                            ast_expr_unary_create(
                                ast_expr_binary_create(a, AstBinaryOp::Addition, i),
                                AstUnaryOp::Dereference,
                            ),
                        PostfixOp::Call(args) =>
                            ast_expr_call_create(a, args),
                        PostfixOp::Dot(name) =>
                            ast_expr_member_create(a, OwningCStr::copy(CStr::from_ptr(name))),
                        PostfixOp::Arrow(name) =>
                            ast_expr_member_create(
                                ast_expr_unary_create(
                                    ast_expr_binary_create(
                                        a,
                                        AstBinaryOp::Addition,
                                        ast_expr_constant_create(0)
                                    ),
                                    AstUnaryOp::Dereference
                                ),
                                OwningCStr::copy(CStr::from_ptr(name)),
                            ),
                        PostfixOp::Inc => ast_expr_incdec_create(a, true, false),
                        PostfixOp::Dec => ast_expr_incdec_create(a, false, false),
                    };
                }
                a
            }
        } /
        allocation_expression()

    rule postfix_op() -> PostfixOp =
        _ "[" _ i:expression() _ "]" { PostfixOp::ArrayAccess(i) } /
        _ "(" _ ")" { PostfixOp::Call(vec![]) } /
        _ "(" _ args:argument_expression_list() _ ")" { PostfixOp::Call(args) } /
        _ "." _ m:identifier() { PostfixOp::Dot(m) } /
        _ "->" _ m:identifier() { PostfixOp::Arrow(m) } /
        _ "++" { PostfixOp::Inc } /
        _ "--" { PostfixOp::Dec }

    rule allocation_expression() -> Box<AstExpr> =
        ".malloc" _ "(" a:expression() ")" { unsafe { ast_expr_alloc_create(a) } } /
        ".free" _ "(" a:expression() ")" { unsafe { ast_expr_dealloc_create(a) } } /
        ".clump" _ "(" a:expression() ")" { unsafe { ast_expr_clump_create(a) } }

    rule argument_expression_list() -> Vec<Box<AstExpr>> =
        args:cs1(<assignment_expression()>) { args }

    rule isdeallocand_expression() -> Box<AstExpr> =
        postfix_expression() /
        "@" _ a:isdeallocand_expression() { unsafe { ast_expr_isdeallocand_create(a) } } /
        "$" _ a:isdeallocand_expression() { unsafe { ast_expr_isdereferencable_create(a) } }

    // 6.5.3 Unary operators
    rule unary_expression() -> Box<AstExpr> =
        isdeallocand_expression() /
        "++" _ a:unary_expression() { unsafe { ast_expr_incdec_create(a, true, true) } } /
        "--" _ a:unary_expression() { unsafe { ast_expr_incdec_create(a, false, true) } } /
        op:unary_operator() _ e:cast_expression() { unsafe { ast_expr_unary_create(e, op) } } /
        "sizeof" _ "(" _ ty:type_name() _ ")" { unsafe { ast_expr_constant_create(1) /* XXX */ } }

    rule unary_operator() -> AstUnaryOp =
        "&" !"&" { AstUnaryOp::Address } /
        "*" { AstUnaryOp::Dereference } /
        "+" { AstUnaryOp::Positive } /
        "-" { AstUnaryOp::Negative } /
        "~" { AstUnaryOp::OnesComplement } /
        "!" { AstUnaryOp::Bang }

    // 6.5.4 Cast expressions
    rule cast_expression() -> Box<AstExpr> = unary_expression()

    // 6.5.5 -- 6.5.14 Binary operators
    rule binary_expression() -> Box<AstExpr> = precedence! {
        x:(@) _ "||" _ y:@ { x }
        --
        x:(@) _ "&&" _ y:@ { x }
        --
        x:(@) _ "==" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Eq, y) } }
        x:(@) _ "!=" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Ne, y) } }
        --
        x:(@) _ "<" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Lt, y) } }
        x:(@) _ ">" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Gt, y) } }
        x:(@) _ "<=" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Le, y) } }
        x:(@) _ ">=" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Ge, y) } }
        --
        x:(@) _ "+" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Addition, y) } }
        x:(@) _ "-" _ y:@ { unsafe { ast_expr_binary_create(x, AstBinaryOp::Subtraction, y) } }
        --
        x:(@) _ "*" _ y:@ { x }
        x:(@) _ "/" _ y:@ { x }
        x:(@) _ "%" _ y:@ { x }
        --
        e:cast_expression() { e }
    }

    rule conditional_expression() -> Box<AstExpr> = binary_expression()

    // 6.5.16 Assignment operators
    rule assignment_expression() -> Box<AstExpr> =
        a:unary_expression() _ assignment_operator() _ b:assignment_expression() {
            unsafe { ast_expr_assignment_create(a, b) }
        } /
        conditional_expression()

    rule assignment_operator() =
        "=" / ">>=" / "<<=" / "+=" / "-=" / "*=" / "/=" / "%=" / "&=" / "^=" / "|="

    pub rule expression() -> Box<AstExpr> = assignment_expression();

    // 6.6 Constant expressions
    rule constant_expression() -> Box<AstExpr> = conditional_expression()

    // 6.7 Declarations
    rule declaration() -> Declaration =
        t:declaration_specifiers() _ ";" {
            unsafe {
                Declaration { name: ast_type_struct_tag(&*t), t }
            }
        } /
        t:declaration_specifiers() _ v:declarator() _ ";" {
            unsafe {
                let is_typedef = (*t).modifiers & MOD_TYPEDEF as libc::c_int != 0;

                let mut t = t;
                for _ in 0..v.ptr_valence {
                    t = ast_type_create_ptr(t);
                }
                if is_typedef {
                    env.add_typename(v.name);
                }
                Declaration { name: v.name, t }
            }
        }

    rule declaration_modifier() -> AstTypeModifier =
        storage_class_specifier() /
        type_qualifier()

    rule declaration_specifiers() -> BoxedType =
        type_specifier() /
        type_specifier() _ t:declaration_specifiers() { t } /
        m:declaration_modifier() _ s:declaration_specifiers() {
            unsafe {
                ast_type_mod_or(s, m);
            }
            s
        }

    rule storage_class_specifier() -> AstTypeModifier =
        K(<"typedef">) {
            // XXX TODO
        /* XXX: typedef is not modelled as a modifier, so our only
         * concern is to prepare the lexer to return the next string it
         * sees as a TYPE_NAME, so that in the type_specifier beneath
         * the fact that this is a typedef will be captured */
            //env.installclass = IC_TYPE;
            // XXX probably handle this in the enclosing rule instead
            MOD_TYPEDEF
        } /
        K(<"extern">) { MOD_EXTERN } /
        K(<"static">) { MOD_STATIC } /
        K(<"auto">) { MOD_AUTO } /
        K(<"register">) { MOD_REGISTER }

    rule type_specifier() -> BoxedType =
        K(<"void">) { unsafe { ast_type_create(AstTypeBase::Void, 0) } } /
        K(<"char">) { unsafe { ast_type_create(AstTypeBase::Char, 0) } } /
        K(<"int">) { unsafe { ast_type_create(AstTypeBase::Int, 0) } } /
        struct_or_union_specifier() /
        t:typedef_name() { unsafe { ast_type_create_userdef(t) } }

    rule typedef_name() -> OwningCStr = i:identifier() {?
        unsafe {
            if env.is_typename(i) {
                Ok(OwningCStr::new(i))
            } else {
                Err("<unused>")
            }
        }
    }

    // note: unions are unsupported in the original
    rule struct_or_union_specifier() -> BoxedType =
        "struct" _ tag:identifier() _ "{" _ fields:struct_declaration_list() _ "}" {
            unsafe { ast_type_create_struct(Some(OwningCStr::new(tag)), Some(Box::new(fields))) }
        } /
        "struct" _ "{" _ fields:struct_declaration_list() _ "}" {
            unsafe { ast_type_create_struct_anonym(fields) }
        } /
        "struct" _ tag:identifier() {
            unsafe { ast_type_create_struct_partial(OwningCStr::new(tag)) }
        }

    rule struct_declaration_list() -> Vec<BoxedVariable> =
        decls:list1(<struct_declaration()>) { decls }

    rule struct_declaration() -> BoxedVariable =
        d:declaration() {
            unsafe { ast_variable_create(OwningCStr::new(d.name), d.t) }
        }

    rule type_qualifier() -> AstTypeModifier =
        K(<"const">) { MOD_CONST } /
        K(<"volatile">) { MOD_VOLATILE }

    rule function_declarator() -> FunctionDeclarator =
        p:pointer() _ d:direct_function_declarator() {
            FunctionDeclarator { ptr_valence: p, decl: d }
        } /
        d:direct_function_declarator() {
            FunctionDeclarator { ptr_valence: 0, decl: d }
        }

    rule direct_function_declarator() -> DirectFunctionDeclarator =
        name:identifier() _ "(" _ params:parameter_type_list() _ ")" {
            unsafe {
                DirectFunctionDeclarator { name, params }
            }
        } /
        name:identifier() _ "(" _ ")" {
            DirectFunctionDeclarator { name, params: vec![] }
        }

    rule declarator() -> Declarator =
        p:pointer() _ name:direct_declarator() { Declarator { ptr_valence: p, name } } /
        name:direct_declarator() { Declarator { ptr_valence: 0, name } } /
        p:pointer() { Declarator { ptr_valence: p, name: ptr::null_mut() } }

    rule direct_declarator() -> BoxedCStr =
        // Note: declarator postfix syntax is ignored in the original
        name:identifier() direct_declarator_postfix()* { name }

    rule direct_declarator_postfix() =
        _ "[" _ constant_expression() _ "]" /
        _ "(" _ parameter_type_list() _ ")"

    rule pointer() -> libc::c_int =
        s:$("*"+) { s.len() as libc::c_int }

    rule parameter_type_list() -> Vec<BoxedVariable> = parameter_list()

    rule parameter_list() -> Vec<BoxedVariable> =
        decls:cs1(<parameter_declaration()>) { decls }

    rule parameter_declaration() -> BoxedVariable =
        t:declaration_specifiers() _ decl:declarator() {
            unsafe {
                let mut t = t;
                for _ in 0..decl.ptr_valence {
                    t = ast_type_create_ptr(t);
                }
                let name = if decl.name.is_null() { dynamic_str(&(0 as libc::c_char)) } else { decl.name };
                ast_variable_create(OwningCStr::new(name), t)
            }
        } /
        t:declaration_specifiers() {
            unsafe { ast_variable_create(OwningCStr::new(dynamic_str(&(0 as libc::c_char))), t) }
        }

    rule type_name() -> BoxedType =
        t:specifier_qualifier_list() _ abstract_declarator() { t } /
        specifier_qualifier_list()

    rule abstract_declarator() -> libc::c_int = pointer()

    rule statement() -> Box<AstStmt> =
        labelled_statement() /
        block:compound_statement() p:position!() {
            unsafe { ast_stmt_create_compound(env.lexloc(p), block) }
        } /
        expression_statement() /
        selection_statement() /
        iteration_statement() /
        jump_statement() /
        iteration_effect_statement() /
        v:compound_verification_statement() p:position!() {
            unsafe { ast_stmt_create_compound_v(env.lexloc(p), v) }
        }

    rule specifier_qualifier_list() -> BoxedType =
        q:type_specifier() _ specifier_qualifier_list() { q } /
        q:type_specifier() { q } /
        // Note: original is I guess UB? type confusion
        type_qualifier() _ t:specifier_qualifier_list() { t } /
        type_qualifier() {
            unsafe { ast_type_create(AstTypeBase::Int, 0) }
        }

    rule labelled_statement() -> Box<AstStmt> =
        label:identifier() _ ":" _ s:statement() p:position!() {
            unsafe { ast_stmt_create_labelled(env.lexloc(p), OwningCStr::new(label), s) }
        } /
        K(<"case">) _ constant_expression() _ ":" _ statement() p:position!() {
            unsafe { ast_stmt_create_nop(env.lexloc(p)) }
        } /
        K(<"default">) _ ":" _ statement() p:position!() {
            unsafe { ast_stmt_create_nop(env.lexloc(p)) }
        }

    rule compound_statement() -> BoxedBlock =
        "{" _ b:block() _ "}" { b }

    rule block() -> BoxedBlock =
        d:declaration_list() _ s:statement_list() {
            unsafe { ast_block_create(d, s) }
        } /
        d:declaration_list() {
            unsafe { ast_block_create(d, vec![]) }
        } /
        s:statement_list() {
            unsafe { ast_block_create(vec![], s) }
        } /
        /* empty */ {
            unsafe { ast_block_create(vec![], vec![]) }
        }

    rule iteration_effect_statement() -> Box<AstStmt> =
        "." _ s:for_iteration_statement(true) { s }

    rule compound_verification_statement() -> BoxedBlock =
        "~" _ "[" _ "]" {
            unsafe { ast_block_create(vec![], vec![]) }
        } /
        "~" _ "[" _ b:block() _ "]" {
            b
        }

    rule declaration_list() -> Vec<BoxedVariable> =
        decls:list1(<declaration()>) { unsafe { variable_array_from_decl_vec(decls) } }

    rule statement_list() -> Vec<Box<AstStmt>> =
        stmts:list1(<statement()>) { stmts }

    rule expression_statement() -> Box<AstStmt> =
        ";" p:position!() { unsafe { ast_stmt_create_nop(env.lexloc(p)) } } /
        e:expression() _ ";" p:position!() { unsafe { ast_stmt_create_expr(env.lexloc(p), e) } }

    rule selection_statement() -> Box<AstStmt> =
        K(<"if">) _ "(" _ cond:expression() _ ")" _ then:statement() _ K(<"else">) _ alt:statement() p:position!() {
            unsafe {
                let neg_cond = ast_expr_unary_create(ast_expr_copy(&*cond), AstUnaryOp::Bang);
                let else_stmt = ast_stmt_create_sel(env.lexloc(p), false, neg_cond, alt, None);
                ast_stmt_create_sel(env.lexloc(p), false, cond, then, Some(else_stmt))
            }
        } /
        K(<"if">) _ "(" _ cond:expression() _ ")" _ then:statement() p:position!() {
            unsafe { ast_stmt_create_sel(env.lexloc(p), false, cond, then, None) }
        } /
        K(<"switch">) _ "(" _ v:expression() _ ")" _ cases:statement() p:position!() {
            unsafe { ast_stmt_create_nop(env.lexloc(p)) }
        }

    rule optional_compound_verification() -> BoxedBlock =
        vs:compound_verification_statement()? {
            vs.unwrap_or_else(|| {
                unsafe { ast_block_create(vec![], vec![]) }
            })
        }

    rule for_iteration_statement(as_iteration_e: bool) -> Box<AstStmt> =
        K(<"for">) _ "(" _ init:expression_statement() _ cond:expression_statement() _ iter:expression() _ ")" _
        verif:optional_compound_verification() _ body:statement() p:position!() {
            unsafe { ast_stmt_create_iter(env.lexloc(p), init, cond, iter, verif, body, as_iteration_e) }
        }

    rule iteration_statement() -> Box<AstStmt> = for_iteration_statement(false)

    rule jump_statement() -> Box<AstStmt> =
        K(<"return">) _ expr:expression() _ ";" p:position!() {
            unsafe { ast_stmt_create_jump(env.lexloc(p), AstJumpKind::Return, Some(expr)) }
        }

    rule block_statement() -> BlockStatement =
        ";" {
            BlockStatement { r#abstract: ptr::null_mut(), body: ptr::null_mut() }
        } /
        c:compound_statement() {
            BlockStatement { r#abstract: ptr::null_mut(), body: c }
        } /
        v:compound_verification_statement() _ ";" {
            BlockStatement { r#abstract: v, body: ptr::null_mut() }
        } /
        v:compound_verification_statement() _ c:compound_statement() {
            BlockStatement { r#abstract: v, body: c }
        }

    rule function_definition() -> BoxedFunction =
        K(<"axiom">) _ t:declaration_specifiers() _ decl:function_declarator() _ body:block_statement() {
            unsafe {
                let mut t = t;
                for _ in 0..decl.ptr_valence {
                    t = ast_type_create_ptr(t);
                }
                ast_function_create(
                    true,
                    t,
                    OwningCStr::new(decl.decl.name),
                    decl.decl.params,
                    if body.r#abstract.is_null() {
                        ast_block_create(vec![], vec![])
                    } else {
                        body.r#abstract
                    },
                    body.body
                )
            }
        } /
        t:declaration_specifiers() _ decl:function_declarator() _ body:block_statement() {
            unsafe {
                let mut t = t;
                for _ in 0..decl.ptr_valence {
                    t = ast_type_create_ptr(t);
                }
                ast_function_create(
                    false,
                    t,
                    OwningCStr::new(decl.decl.name),
                    decl.decl.params,
                    if body.r#abstract.is_null() {
                        ast_block_create(vec![], vec![])
                    } else {
                        body.r#abstract
                    },
                    body.body
                )
            }
        } /
        // Note: this was in the original. surely shouldn't be allowed, and implicit return type
        // should be int if it is allowed.
        decl:function_declarator() _ body:block_statement() {
            unsafe {
                ast_function_create(
                    false,
                    ast_type_create(AstTypeBase::Void, 0),
                    OwningCStr::new(decl.decl.name),
                    decl.decl.params,
                    if body.r#abstract.is_null() {
                        ast_block_create(vec![], vec![])
                    } else {
                        body.r#abstract
                    },
                    body.body
                )
            }
        }

    rule external_declaration() -> Box<AstExternDecl> =
        f:function_definition() { unsafe { ast_functiondecl_create(f) } } /
        d:declaration() { unsafe { ast_decl_create(OwningCStr::new(dynamic_str(d.name)), d.t) } }

    pub rule translation_unit() -> Box<Ast> =
        directive()? _ decl:list1(<external_declaration()>) _ { unsafe { ast_from_vec(decl) } }
}
}
