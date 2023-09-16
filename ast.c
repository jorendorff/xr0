#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include "ast.h"
#include "lex.h"
#include "util.h"

static struct ast_expr *
ast_expr_create()
{
	return malloc(sizeof(struct ast_expr));
}

struct ast_expr *
ast_expr_create_identifier(char *s)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_IDENTIFIER;
	expr->u.string = s;
	return expr;
}

char *
ast_expr_as_identifier(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_IDENTIFIER);
	return expr->u.string;
}

static void
ast_expr_destroy_identifier(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_IDENTIFIER);
	free(expr->u.string);
}

struct ast_expr *
ast_expr_create_constant(int k)
{
	/* TODO: generalise for all constant cases */
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_CONSTANT;
	expr->u.constant = k;
	return expr;
}

int
ast_expr_as_constant(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_CONSTANT);
	return expr->u.constant;
}

struct ast_expr *
ast_expr_create_literal(char *s)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_STRING_LITERAL;
	expr->u.string = s;
	return expr;
}

static void
ast_expr_destroy_literal(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_STRING_LITERAL);
	free(expr->u.string);
}

struct ast_expr *
ast_expr_create_bracketed(struct ast_expr *root)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_BRACKETED;
	expr->root = root;
	return expr;
}

static void
ast_expr_bracketed_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root);
	strbuilder_printf(b, "(%s)", root);
	free(root);
}


struct ast_expr *
ast_expr_create_iteration()
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_ITERATION;
	return expr;
}

struct ast_expr *
ast_expr_create_access(struct ast_expr *root, struct ast_expr *index)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_ACCESS;
	expr->root = root;
	expr->u.access.index = index;
	return expr;
}

struct ast_expr *
ast_expr_access_root(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ACCESS);
	return expr->root;
}

struct ast_expr *
ast_expr_access_index(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ACCESS);
	return expr->u.access.index;
}

static void
ast_expr_access_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root);
	char *index = ast_expr_str(expr->u.access.index);
	strbuilder_printf(b, "%s[%s]", root, index);
	free(root);
	free(index);
}

static void
ast_expr_destroy_access(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ACCESS);
	ast_expr_destroy(expr->root);
	ast_expr_destroy(expr->u.access.index);
}

struct ast_expr *
ast_expr_create_call(struct ast_expr *root, int narg, struct ast_expr **arg)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_CALL;
	expr->root = root;
	expr->u.call.n = narg;
	expr->u.call.arg = arg;
	return expr;
}

struct ast_expr *
ast_expr_call_root(struct ast_expr *expr)
{
	return expr->root;
}

int
ast_expr_call_nargs(struct ast_expr *expr)
{
	return expr->u.call.n;
}

struct ast_expr **
ast_expr_call_args(struct ast_expr *expr)
{
	return expr->u.call.arg;
}

static void
ast_expr_call_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root);
	strbuilder_printf(b, "%s(", root);
	for (int i = 0; i < expr->u.call.n; i++) {
		char *arg = ast_expr_str(expr->u.call.arg[i]);
		strbuilder_printf(b, "%s%s", arg,
			(i + 1 < expr->u.call.n) ? ", " : "");
		free(arg);
	}
	strbuilder_printf(b, ")");
	free(root);
}

static void
ast_expr_destroy_call(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_CALL);
	ast_expr_destroy(expr->root);
	for (int i = 0; i < expr->u.call.n; i++) {
		ast_expr_destroy(expr->u.call.arg[i]);
	}
	free(expr->u.call.arg);
}

static struct ast_expr *
ast_expr_copy_call(struct ast_expr *expr)
{
	struct ast_expr **arg = malloc(sizeof(struct ast_expr *) * expr->u.call.n);
	for (int i = 0; i < expr->u.call.n; i++) {
		arg[i] = ast_expr_copy(expr->u.call.arg[i]);
	}
	return ast_expr_create_call(
		ast_expr_copy(expr->root),
		expr->u.call.n,
		arg
	);
}

struct ast_expr *
ast_expr_create_incdec(struct ast_expr *root, bool inc, bool pre)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_INCDEC;
	expr->root = root;
	expr->u.incdec.inc = inc;
	expr->u.incdec.pre = pre;
	return expr;
}


static void
ast_expr_destroy_incdec(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_INCDEC);
	ast_expr_destroy(expr->root);
}

static void
ast_expr_incdec_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root);
	char *op = expr->u.incdec.inc ? "++" : "--";
	if (expr->u.incdec.pre) {
		strbuilder_printf(b, "%s%s", op, root);
	} else {
		strbuilder_printf(b, "%s%s", root, op);
	}
	free(root);
}

struct ast_expr *
ast_expr_create_unary(struct ast_expr *root, enum ast_unary_operator op)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_UNARY;
	expr->root = root;
	expr->u.unary_op = op;
	return expr;
}

static void
ast_expr_destroy_unary(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_UNARY);
	ast_expr_destroy(expr->root);
}

enum ast_unary_operator
ast_expr_unary_op(struct ast_expr *expr)
{
	return expr->u.unary_op;
}

struct ast_expr *
ast_expr_unary_operand(struct ast_expr *expr)
{
	return expr->root;
}

static void
ast_expr_unary_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	const char opchar[] = {
		[UNARY_OP_ADDRESS]		= '&',
		[UNARY_OP_DEREFERENCE]		= '*',
		[UNARY_OP_POSITIVE]		= '+',
		[UNARY_OP_NEGATIVE]		= '-',
		[UNARY_OP_ONES_COMPLEMENT]	= '~',
		[UNARY_OP_BANG]			= '!',
	};
	char *root = ast_expr_str(expr->root);
	strbuilder_printf(b, "%c%s", opchar[expr->u.unary_op], root);
	free(root);
}

struct ast_expr *
ast_expr_create_binary(struct ast_expr *e1, enum ast_binary_operator op,
		struct ast_expr *e2)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_BINARY;
	expr->u.binary.e1 = e1;
	expr->u.binary.op = op;
	expr->u.binary.e2 = e2;
	return expr;
}

struct ast_expr *
ast_expr_binary_e1(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_BINARY);
	return expr->u.binary.e1;
}

struct ast_expr *
ast_expr_binary_e2(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_BINARY);
	return expr->u.binary.e2;
}

enum ast_binary_operator
ast_expr_binary_op(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_BINARY);
	return expr->u.binary.op;
}

static void
ast_expr_destroy_binary(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_BINARY);
	ast_expr_destroy(expr->u.binary.e1);
	ast_expr_destroy(expr->u.binary.e2);
}

static void
ast_expr_binary_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	const char opchar[] = {
		[BINARY_OP_ADDITION]		= '+',
	};
	char *e1 = ast_expr_str(expr->u.binary.e1);
	char *e2 = ast_expr_str(expr->u.binary.e2);
	strbuilder_printf(b, "%s%c%s", e1, opchar[expr->u.binary.op], e2);
	free(e2);
	free(e1);
}

struct ast_expr *
ast_expr_create_chain(struct ast_expr *root, enum ast_chain_operator op,
		struct ast_expr *justification, struct ast_expr *last)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_CHAIN;
	expr->root = root;
	expr->u.chain.op = op;
	expr->u.chain.justification = justification;
	expr->u.chain.last = last;
	return expr;
}

static void
ast_expr_destroy_chain(struct ast_expr *expr)
{
	ast_expr_destroy(expr->root);
	/* TODO: ast_expr_destroy(expr->u.chain.justification);*/
	ast_expr_destroy(expr->u.chain.last);
}

static void
ast_expr_chain_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	const char *opstr[] = {
		[CHAIN_OP_EQV]		= "===",
		[CHAIN_OP_IMPL]		= "==>",
		[CHAIN_OP_FLLW]		= "<==",

		[CHAIN_OP_EQ]		= "==",
		[CHAIN_OP_NE]		= "!=",

		[CHAIN_OP_LT]		= "<",
		[CHAIN_OP_GT]		= ">",
		[CHAIN_OP_LE]		= "<=",
		[CHAIN_OP_GE]		= ">=",
	};
	char *root = ast_expr_str(expr->root),
	     *last = ast_expr_str(expr->u.chain.last);
	strbuilder_printf(b, "%s %s %s", root, opstr[expr->u.chain.op], last);
	free(last);
	free(root);
}

struct ast_expr *
ast_expr_create_assignment(struct ast_expr *root, struct ast_expr *value)
{
	struct ast_expr *expr = ast_expr_create();
	expr->kind = EXPR_ASSIGNMENT;
	expr->root = root;
	expr->u.assignment_value = value;
	return expr;
}

struct ast_expr *
ast_expr_assignment_lval(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ASSIGNMENT);
	return expr->root;
}

struct ast_expr *
ast_expr_assignment_rval(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ASSIGNMENT);
	return expr->u.assignment_value;
}

static void
ast_expr_destroy_assignment(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ASSIGNMENT);
	ast_expr_destroy(expr->root);
	ast_expr_destroy(expr->u.assignment_value);
}

static void
ast_expr_assignment_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root),
	     *value = ast_expr_str(expr->u.assignment_value);
	strbuilder_printf(b, "%s = %s", root, value);
	free(value);
	free(root);
}

struct ast_expr *
ast_expr_create_memory(enum effect_kind kind, struct ast_expr *expr)
{
	struct ast_expr *new = ast_expr_create();
	new->kind = EXPR_MEMORY;
	new->root = expr;
	new->u.memory.kind = kind;
	return new;
}

struct ast_expr *
ast_expr_memory_root(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_MEMORY);
	return expr->root;
}

bool
ast_expr_memory_isalloc(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_MEMORY);
	return expr->u.memory.kind == EFFECT_ALLOC;
}

bool
ast_expr_memory_isunalloc(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_MEMORY);
	return expr->u.memory.kind == EFFECT_DEALLOC;
}

bool
ast_expr_memory_isundefined(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_MEMORY);
	return expr->u.memory.kind == EFFECT_UNDEFINED;
}

enum effect_kind
ast_expr_memory_kind(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_MEMORY);
	return expr->u.memory.kind;
}

struct effect_mapping {
	enum effect_kind kind;
	const char* string;
};

struct effect_mapping effect_string_map[] = {
	{EFFECT_ALLOC, "alloc"},
	{EFFECT_DEALLOC, "dealloc"},
	{EFFECT_UNDEFINED, "undefined"}
};

const char*
get_effect_string(enum effect_kind kind)
{
	int len = sizeof(effect_string_map) / sizeof(effect_string_map[0]);
	for (int i = 0; i < len; i++) {
		if (effect_string_map[i].kind == kind) {
			return effect_string_map[i].string;
		}
	} 
	return "unknown";
}

static void
ast_expr_memory_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	assert(ast_expr_kind(expr) == EXPR_MEMORY);

	if (ast_expr_memory_kind(expr) == EFFECT_UNDEFINED) {
		strbuilder_printf(b, "undefined");
		return;
	}

	char *root = ast_expr_str(expr->root);

	strbuilder_printf(
		b, "%s %s",
		get_effect_string(ast_expr_memory_kind(expr)),
		root
	);
	free(root);
}

struct ast_expr *
ast_expr_create_assertion(struct ast_expr *assertand)
{
	struct ast_expr *new = ast_expr_create();
	new->kind = EXPR_ASSERTION;
	new->root = assertand;
	return new;
}

struct ast_expr *
ast_expr_assertion_assertand(struct ast_expr *expr)
{
	assert(expr->kind == EXPR_ASSERTION);
	return expr->root;
}

static void
ast_expr_assertion_str_build(struct ast_expr *expr, struct strbuilder *b)
{
	char *root = ast_expr_str(expr->root);
	strbuilder_printf(b, "@%s", root);
	free(root);
}

void
ast_expr_destroy(struct ast_expr *expr)
{
	switch (expr->kind) {
	case EXPR_IDENTIFIER:
		ast_expr_destroy_identifier(expr);
		break;
	case EXPR_STRING_LITERAL:
		ast_expr_destroy_literal(expr);
		break;
	case EXPR_BRACKETED:
		ast_expr_destroy(expr->root);
		break;
	case EXPR_ACCESS:
		ast_expr_destroy_access(expr);
		break;
	case EXPR_CALL:
		ast_expr_destroy_call(expr);
		break;
	case EXPR_INCDEC:
		ast_expr_destroy_incdec(expr);
		break;
	case EXPR_UNARY:
		ast_expr_destroy_unary(expr);
		break;
	case EXPR_BINARY:
		ast_expr_destroy_binary(expr);
		break;
	case EXPR_CHAIN:
		ast_expr_destroy_chain(expr);
		break;
	case EXPR_ASSIGNMENT:
		ast_expr_destroy_assignment(expr);
		break;
	case EXPR_CONSTANT:
		break;
	case EXPR_MEMORY:
		expr->root ? ast_expr_destroy(expr->root) : 0;
		break;
	case EXPR_ASSERTION:
		ast_expr_destroy(expr->root);
		break;
	default:
		fprintf(stderr, "unknown ast_expr_kind %d\n", expr->kind);
		exit(EXIT_FAILURE);
	}
	free(expr);
}

char *
ast_expr_str(struct ast_expr *expr)
{
	struct strbuilder *b = strbuilder_create();
	switch (expr->kind) {
	case EXPR_IDENTIFIER:
		strbuilder_printf(b, expr->u.string);
		break;
	case EXPR_CONSTANT:
		strbuilder_printf(b, "%d", expr->u.constant);
		break;
	case EXPR_STRING_LITERAL:
		strbuilder_printf(b, "\"%s\"", expr->u.string);
		break;
	case EXPR_BRACKETED:
		ast_expr_bracketed_str_build(expr, b);
		break;
	case EXPR_ACCESS:
		ast_expr_access_str_build(expr, b);
		break;
	case EXPR_CALL:
		ast_expr_call_str_build(expr, b);
		break;
	case EXPR_INCDEC:
		ast_expr_incdec_str_build(expr, b);
		break;
	case EXPR_UNARY:
		ast_expr_unary_str_build(expr, b);
		break;
	case EXPR_BINARY:
		ast_expr_binary_str_build(expr, b);
		break;
	case EXPR_CHAIN:
		ast_expr_chain_str_build(expr, b);
		break;
	case EXPR_ASSIGNMENT:
		ast_expr_assignment_str_build(expr, b);
		break;
	case EXPR_MEMORY:
		ast_expr_memory_str_build(expr, b);
		break;
	case EXPR_ASSERTION:
		ast_expr_assertion_str_build(expr, b);
		break;
	default:
		assert(false);
	}
	return strbuilder_build(b);
}

struct ast_expr *
ast_expr_copy(struct ast_expr *expr)
{
	assert(expr);
	switch (expr->kind) {
	case EXPR_IDENTIFIER:
		return ast_expr_create_identifier(dynamic_str(expr->u.string));
	case EXPR_CONSTANT:
		return ast_expr_create_constant(expr->u.constant);
	case EXPR_STRING_LITERAL:
		return ast_expr_create_literal(dynamic_str(expr->u.string));
	case EXPR_BRACKETED:
		return ast_expr_create_bracketed(ast_expr_copy(expr->root));
	case EXPR_ACCESS:
		return ast_expr_create_access(
			ast_expr_copy(expr->root),
			ast_expr_copy(expr->u.access.index)
		);
	case EXPR_CALL:
		return ast_expr_copy_call(expr);
	case EXPR_INCDEC:
		return ast_expr_create_incdec(
			ast_expr_copy(expr->root),
			expr->u.incdec.inc,
			expr->u.incdec.pre
		);
	case EXPR_UNARY:
		return ast_expr_create_unary(
			ast_expr_copy(expr->root),
			expr->u.unary_op
		);
	case EXPR_BINARY:
		return ast_expr_create_binary(
			ast_expr_copy(expr->u.binary.e1),
			expr->u.binary.op,
			ast_expr_copy(expr->u.binary.e2)
		);
	case EXPR_CHAIN:
		return ast_expr_create_chain(
			ast_expr_copy(expr->root),
			expr->u.chain.op,
			NULL, /* XXX */
			ast_expr_copy(expr->u.chain.last)
		);
	case EXPR_ASSIGNMENT:
		return ast_expr_create_assignment(
			ast_expr_copy(expr->root),
			ast_expr_copy(expr->u.assignment_value)
		);
	case EXPR_MEMORY:
		return ast_expr_create_memory(
			expr->u.memory.kind,
			expr->root ? ast_expr_copy(expr->root) : NULL
		);
	case EXPR_ASSERTION:
		return ast_expr_create_assertion(
			ast_expr_copy(expr->root)
		);
	default:

		fprintf(stderr, "cannot copy `%s'\n", ast_expr_str(expr));
		assert(false);
	}
}

enum ast_expr_kind
ast_expr_kind(struct ast_expr *expr)
{
	return expr->kind;
}

bool
ast_expr_equal(struct ast_expr *e1, struct ast_expr *e2)
{
	if (!e1 || !e2) {
		return false;
	}
	if (e1->kind != e2->kind) {
		return false;	
	}
	switch (e1->kind) {
	case EXPR_CONSTANT:
		return e1->u.constant == e2->u.constant;
	case EXPR_IDENTIFIER:
		return strcmp(ast_expr_as_identifier(e1), ast_expr_as_identifier(e2)) == 0;
	case EXPR_ASSIGNMENT:
		return ast_expr_equal(e1->root, e2->root)
			&& ast_expr_equal(e1->u.assignment_value, e2->u.assignment_value); 
	case EXPR_CHAIN:
		return ast_expr_equal(e1->root, e2->root)
			&& e1->u.chain.op == e2->u.chain.op
			&& ast_expr_equal(e1->u.chain.last, e2->u.chain.last);
	case EXPR_BINARY:
		return ast_expr_binary_op(e1) == ast_expr_binary_op(e2) &&
			ast_expr_equal(ast_expr_binary_e1(e1), ast_expr_binary_e1(e2)) && 
			ast_expr_equal(ast_expr_binary_e2(e1), ast_expr_binary_e2(e2));
	default:
		fprintf(stderr, "cannot compare e1: `%s' and e2: %s\n",
			ast_expr_str(e1), ast_expr_str(e2));
		assert(false);
	}
}

struct ast_block *
ast_block_create(struct ast_variable **decl, int ndecl, 
	struct ast_stmt **stmt, int nstmt)
{
	struct ast_block *b = malloc(sizeof(struct ast_block));
	b->decl = decl;
	b->ndecl = ndecl;
	b->stmt = stmt;
	b->nstmt = nstmt;
	return b;
}

void
ast_block_destroy(struct ast_block *b)
{
	for (int i = 0; i < b->ndecl; i++) {
		ast_variable_destroy(b->decl[i]);
	}
	free(b->decl);
	for (int i = 0; i < b->nstmt; i++) {
		ast_stmt_destroy(b->stmt[i]);
	}
	free(b->stmt);
	free(b);
}

static struct ast_variable **
copy_var_arr(int len, struct ast_variable **);

static struct ast_stmt **
copy_stmt_arr(int len, struct ast_stmt **);

struct ast_block *
ast_block_copy(struct ast_block *b)
{
	assert(b);
	return ast_block_create(
		copy_var_arr(b->ndecl, b->decl),
		b->ndecl,
		copy_stmt_arr(b->nstmt, b->stmt),
		b->nstmt
	);
}

static struct ast_variable **
copy_var_arr(int len, struct ast_variable **var)
{
	assert(len == 0 || var);
	if (len == 0) {
		return NULL;
	}
	struct ast_variable **new = malloc(sizeof(struct ast_variable *) * len); 
	for (int i = 0; i < len; i++) {
		new[i] = ast_variable_copy(var[i]);
	}
	return new;
}

static struct ast_stmt **
copy_stmt_arr(int len, struct ast_stmt **stmt)
{
	assert(len == 0 || stmt);
	if (len == 0) {
		return NULL;
	}
	struct ast_stmt **new = malloc(sizeof(struct ast_stmt *) * len); 
	for (int i = 0; i < len; i++) {
		new[i] = ast_stmt_copy(stmt[i]);
	}
	return new;
}

char *
ast_block_str(struct ast_block *b)
{
	struct strbuilder *sb = strbuilder_create();
	for (int i = 0; i < b->ndecl; i++) {
		char *s = ast_variable_str(b->decl[i]);
		strbuilder_printf(sb, "%s;\n", s);
		free(s);
	}
	for (int i = 0; i < b->nstmt; i++) {
		char *s = ast_stmt_str(b->stmt[i]);
		strbuilder_printf(sb, "%s\n", s);
		free(s);
	}
	return strbuilder_build(sb);
}

int
ast_block_ndecls(struct ast_block *b)
{
	return b->ndecl;
}

struct ast_variable **
ast_block_decls(struct ast_block *b)
{
	assert(b->ndecl > 0 || !b->decl);
	return b->decl;
}

int
ast_block_nstmts(struct ast_block *b)
{
	return b->nstmt;
}

struct ast_stmt **
ast_block_stmts(struct ast_block *b)
{
	assert(b->nstmt > 0 || !b->stmt);
	return b->stmt;
}

static struct ast_stmt *
ast_stmt_create(struct lexememarker *loc)
{
	struct ast_stmt *stmt = calloc(1, sizeof(struct ast_stmt));
	stmt->loc = loc;
	return stmt;
}

struct ast_stmt *
ast_stmt_create_nop(struct lexememarker *loc)
{
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_NOP;
	return stmt;
}

static void
ast_stmt_nop_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	strbuilder_printf(b, ";");
}

struct ast_stmt *
ast_stmt_create_expr(struct lexememarker *loc, struct ast_expr *expr)
{
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_EXPR;
	stmt->u.expr = expr;
	return stmt;
}

static void
ast_stmt_expr_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_EXPR);
	char *s = ast_expr_str(stmt->u.expr);
	strbuilder_printf(b, "%s;", s);
	free(s);
}

struct ast_stmt *
ast_stmt_create_compound(struct lexememarker *loc, struct ast_block *b)
{
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_COMPOUND;
	stmt->u.compound = b;
	return stmt;
}

struct ast_block *
ast_stmt_as_block(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_COMPOUND);
	return stmt->u.compound;
}

static void
ast_stmt_compound_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_COMPOUND || stmt->kind == STMT_COMPOUND_V);
	char *s = ast_block_str(stmt->u.compound);
	strbuilder_printf(b, s);
	free(s);
}

struct ast_stmt *
ast_stmt_create_compound_v(struct lexememarker *loc, struct ast_block *b)
{
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_COMPOUND_V;
	stmt->u.compound = b;
	return stmt;
}

struct ast_stmt *
ast_stmt_create_jump(struct lexememarker *loc, enum ast_jump_kind kind,
		struct ast_expr *rv)
{
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_JUMP;
	stmt->u.jump.kind = JUMP_RETURN;
	stmt->u.jump.rv = rv;
	return stmt;
}

struct ast_expr *
ast_stmt_jump_rv(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_JUMP && stmt->u.jump.kind == JUMP_RETURN);
	return stmt->u.jump.rv;
}

static void
ast_stmt_destroy_jump(struct ast_stmt *stmt)
{
	struct ast_expr *rv = stmt->u.jump.rv;
	if (!rv) {
		return;
	}
	assert(stmt->u.jump.kind == JUMP_RETURN);
	ast_expr_destroy(rv);
}

struct ast_stmt *
ast_stmt_create_sel(struct lexememarker *loc, bool isswitch, struct ast_expr *cond,
		struct ast_stmt *body, struct ast_stmt *nest)
{
	assert(!isswitch); /* XXX */
	assert(cond);
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_SELECTION;
	stmt->u.selection.isswitch = isswitch;
	stmt->u.selection.cond = cond;
	stmt->u.selection.body = body;
	stmt->u.selection.nest = nest;
	return stmt;
}

struct ast_expr *
ast_stmt_sel_cond(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_SELECTION);
	return stmt->u.selection.cond;
}

struct ast_stmt *
ast_stmt_sel_body(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_SELECTION);
	return stmt->u.selection.body;
}

struct ast_stmt *
ast_stmt_sel_nest(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_SELECTION);
	return stmt->u.selection.nest;
}

static void
ast_stmt_sel_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_SELECTION);
	char *cond	= ast_expr_str(stmt->u.selection.cond),
	     *body	= ast_stmt_str(stmt->u.selection.body);

	/* XXX: we only support simple IF for now */
	strbuilder_printf(
		b,
		"if (%s) { %s }",
		cond, body
	);

	struct ast_stmt *nest_stmt = stmt->u.selection.nest;
	if (nest_stmt) {
		char *nest = ast_stmt_str(nest_stmt);
		strbuilder_printf(
			b,
			" else %s",
			nest
		);
		free(nest);
	}

	free(cond); free(body);
}

struct ast_stmt *
ast_stmt_create_iter(struct lexememarker *loc,
		struct ast_stmt *init, struct ast_stmt *cond,
		struct ast_expr *iter, struct ast_block *abstract,
		struct ast_stmt *body)
{
	assert(init && cond && iter && abstract && body);
	struct ast_stmt *stmt = ast_stmt_create(loc);
	stmt->kind = STMT_ITERATION;
	stmt->u.iteration.init = init;
	stmt->u.iteration.cond = cond;
	stmt->u.iteration.iter = iter;
	stmt->u.iteration.body = body;
	stmt->u.iteration.abstract = abstract;
	return stmt;
}

struct ast_stmt *
ast_stmt_iter_init(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	return stmt->u.iteration.init;
}

struct ast_stmt *
ast_stmt_iter_cond(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	return stmt->u.iteration.cond;
}

struct ast_expr *
ast_stmt_iter_iter(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	return stmt->u.iteration.iter;
}

struct ast_block *
ast_stmt_iter_abstract(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	return stmt->u.iteration.abstract;
}

struct ast_stmt *
ast_stmt_iter_body(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	return stmt->u.iteration.body;
}

struct ast_expr *
ast_stmt_iter_lower_bound(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	struct ast_stmt *init = stmt->u.iteration.init;
	assert(init->kind == STMT_EXPR);
	struct ast_expr *expr = init->u.expr;
	assert(expr->kind == EXPR_ASSIGNMENT);
	return expr->u.assignment_value;
}

struct ast_expr *
ast_stmt_iter_upper_bound(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_ITERATION);
	struct ast_stmt *cond = stmt->u.iteration.cond;
	assert(cond->kind == STMT_EXPR);
	struct ast_expr *expr = cond->u.expr;
	assert(expr->kind == EXPR_CHAIN);
	return expr->u.chain.last;
}

static struct ast_stmt *
ast_stmt_copy_iter(struct ast_stmt *stmt)
{
	stmt->kind = STMT_ITERATION;
	struct ast_stmt *copy = ast_stmt_copy(stmt);
	stmt->kind = STMT_ITERATION_E;
	return ast_stmt_create_iter_e(copy);
}

static void
ast_stmt_iter_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_ITERATION);
	char *init = ast_stmt_str(stmt->u.iteration.init),
	     *cond = ast_stmt_str(stmt->u.iteration.cond),
	     *body = ast_stmt_str(stmt->u.iteration.body),
	     *iter = ast_expr_str(stmt->u.iteration.iter);

	char *abs = stmt->u.iteration.abstract ?
		ast_block_str(stmt->u.iteration.abstract) : "";

	strbuilder_printf(
		b,
		"for (%s %s %s) [%s] { %s }",
		init, cond, iter, abs, body
	);

	free(init); free(cond); free(body); free(iter);
}

struct ast_stmt *
ast_stmt_create_iter_e(struct ast_stmt *stmt)
{
	/* TODO: determine where loc should go */
	assert(stmt->kind == STMT_ITERATION);
	stmt->kind = STMT_ITERATION_E;
	return stmt;
}

static void
ast_stmt_iter_e_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_ITERATION_E);
	stmt->kind = STMT_ITERATION;
	char *s = ast_stmt_str(stmt);
	stmt->kind = STMT_ITERATION_E;
	strbuilder_printf(b, ".%s", s);
	free(s);
}

static void
ast_stmt_jump_sprint(struct ast_stmt *stmt, struct strbuilder *b)
{
	assert(stmt->kind == STMT_JUMP);
	char *rv = ast_expr_str(stmt->u.jump.rv);

	strbuilder_printf(
		b,
		"return %s;\n",
		rv
	);

	free(rv);
}

static struct ast_expr *
ast_expr_copy_ifnotnull(struct ast_expr *expr)
{
	return expr ? ast_expr_copy(expr) : NULL;
}

void
ast_stmt_destroy(struct ast_stmt *stmt)
{
	switch (stmt->kind) {
	case STMT_NOP:
		break;
	case STMT_COMPOUND:
	case STMT_COMPOUND_V:
		ast_block_destroy(stmt->u.compound);
		break;
	case STMT_SELECTION:
		ast_expr_destroy(stmt->u.selection.cond);
		ast_stmt_destroy(stmt->u.selection.body);
		if (stmt->u.selection.nest) {
			ast_stmt_destroy(stmt->u.selection.nest);
		}
		break;
	case STMT_ITERATION:
	case STMT_ITERATION_E:
		ast_stmt_destroy(stmt->u.iteration.init);
		ast_stmt_destroy(stmt->u.iteration.cond);
		ast_stmt_destroy(stmt->u.iteration.body);
		ast_expr_destroy(stmt->u.iteration.iter);
		ast_block_destroy(stmt->u.iteration.abstract);
		break;
	case STMT_EXPR:
		ast_expr_destroy(stmt->u.expr);
		break;
	case STMT_JUMP:
		ast_stmt_destroy_jump(stmt);
		break;
	default:
		assert(false);
		break;
	}
	if (stmt->loc) {
		lexememarker_destroy(stmt->loc);
	}
	free(stmt);
}

struct ast_stmt *
ast_stmt_copy(struct ast_stmt *stmt)
{
	struct lexememarker *loc = stmt->loc
		? lexememarker_copy(stmt->loc)
		: NULL;
	switch (stmt->kind) {
	case STMT_NOP:
		return ast_stmt_create_nop(loc);
	case STMT_EXPR:
		return ast_stmt_create_expr(loc, ast_expr_copy(stmt->u.expr));
	case STMT_COMPOUND:
		return ast_stmt_create_compound(
			loc, ast_block_copy(stmt->u.compound)
		);
	case STMT_COMPOUND_V:
		return ast_stmt_create_compound_v(
			loc, ast_block_copy(stmt->u.compound)
		);
	case STMT_SELECTION:
		return ast_stmt_create_sel(
			loc,
			stmt->u.selection.isswitch,
			ast_expr_copy(stmt->u.selection.cond),
			ast_stmt_copy(stmt->u.selection.body),
			stmt->u.selection.nest
				? ast_stmt_copy(stmt->u.selection.nest)
				: NULL
		);
	case STMT_ITERATION:
		return ast_stmt_create_iter(
			loc,
			ast_stmt_copy(stmt->u.iteration.init),
			ast_stmt_copy(stmt->u.iteration.cond),
			ast_expr_copy(stmt->u.iteration.iter),
			ast_block_copy(stmt->u.iteration.abstract),
			ast_stmt_copy(stmt->u.iteration.body)
		);
	case STMT_ITERATION_E:
		return ast_stmt_copy_iter(stmt);
	case STMT_JUMP:
		return ast_stmt_create_jump(
			loc, stmt->u.jump.kind,
			ast_expr_copy_ifnotnull(stmt->u.jump.rv)
		);
	default:
		fprintf(stderr, "wrong stmt kind: %d\n", stmt->kind);
		assert(false);
	}
}

char *
ast_stmt_str(struct ast_stmt *stmt)
{
	struct strbuilder *b = strbuilder_create();
	switch (stmt->kind) {
	case STMT_NOP:
		ast_stmt_nop_sprint(stmt, b);
		break;
	case STMT_EXPR:
		ast_stmt_expr_sprint(stmt, b);
		break;
	case STMT_COMPOUND:
		ast_stmt_compound_sprint(stmt, b);
		break;
	case STMT_COMPOUND_V:
		ast_stmt_compound_sprint(stmt, b);
		break;
	case STMT_SELECTION:
		ast_stmt_sel_sprint(stmt, b);
		break;
	case STMT_ITERATION:
		ast_stmt_iter_sprint(stmt, b);
		break;
	case STMT_ITERATION_E:
		ast_stmt_iter_e_sprint(stmt, b);
		break;
	case STMT_JUMP:
		ast_stmt_jump_sprint(stmt, b);
		break;
	default:
		fprintf(stderr, "wrong stmt kind: %d\n", stmt->kind);
		assert(false);
	}
	return strbuilder_build(b);
}

bool
ast_stmt_equal(struct ast_stmt *s1, struct ast_stmt *s2)
{
	if (!s1 || !s2) {
		return false;
	}
	if (ast_stmt_kind(s1) != ast_stmt_kind(s2)) {
		return false;
	}
	switch (ast_stmt_kind(s1)) {
	case STMT_EXPR:
		return ast_expr_equal(ast_stmt_as_expr(s1), ast_stmt_as_expr(s2));
	default:
		fprintf(stderr, "wrong stmt kind: %d\n", ast_stmt_kind(s1));
		assert(false);
	}
}

enum ast_stmt_kind
ast_stmt_kind(struct ast_stmt *stmt)
{
	return stmt->kind;
}

struct ast_block *
ast_stmt_as_v_block(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_COMPOUND_V);
	return stmt->u.compound;
}

struct ast_expr *
ast_stmt_as_expr(struct ast_stmt *stmt)
{
	assert(stmt->kind == STMT_EXPR);
	return stmt->u.expr;
}

struct ast_type *
ast_type_create(enum ast_type_base base, enum ast_type_modifier mod)
{
	struct ast_type *t = malloc(sizeof(struct ast_type));
	assert(t);
	t->base = base;
	t->mod = mod;
	return t;
}

struct ast_type *
ast_type_create_ptr(struct ast_type *ref)
{
	assert(ref);
	struct ast_type *t = ast_type_create(TYPE_POINTER, 0);
	t->u.ptr_type = ref;
	return t;
}

struct ast_type *
ast_type_create_arr(struct ast_type *base, int length)
{
	assert(base);
	struct ast_type *t = ast_type_create(TYPE_ARRAY, 0);
	t->u.arr.type = base;
	t->u.arr.length = length;
	return t;
}

struct ast_type *
ast_type_create_typedef(struct ast_type *base, char *name)
{
	assert(base);
	struct ast_type *t = ast_type_create(TYPE_TYPEDEF, 0);
	t->u._typedef.type = base;
	t->u._typedef.name = name;
	return t;
}

void
ast_type_destroy(struct ast_type *t)
{
	switch (t->base) {
	case TYPE_TYPEDEF:
		assert(t->u._typedef.type);
		ast_type_destroy(t->u._typedef.type);
		assert(t->u._typedef.name);
		free(t->u._typedef.name);
		break;
	case TYPE_POINTER:
		assert(t->u.ptr_type);
		ast_type_destroy(t->u.ptr_type);
		break;
	case TYPE_ARRAY:
		assert(t->u.arr.type);
		ast_type_destroy(t->u.arr.type);
	default:
		break;
	}
	free(t);
}

struct ast_type *
ast_type_copy(struct ast_type *t)
{
	assert(t);
	switch (t->base) {
	case TYPE_TYPEDEF:
		return ast_type_create_typedef(
			ast_type_copy(t->u._typedef.type),
			dynamic_str(t->u._typedef.name)
		);
	case TYPE_POINTER:
		return ast_type_create_ptr(
			ast_type_copy(t->u.ptr_type)
		);
	case TYPE_ARRAY:
		return ast_type_create_arr(
			ast_type_copy(t->u.arr.type),
			t->u.arr.length
		);
	default:
		return ast_type_create(t->base, t->mod);
	}
}

static void
ast_type_str_build_ptr(struct strbuilder *b, struct ast_type *t);

static void
ast_type_str_build_arr(struct strbuilder *b, struct ast_type *t);

static void
ast_type_str_build_typedef(struct strbuilder *b, struct ast_type *t);

char *
ast_type_str(struct ast_type *t)
{
	assert(t);
	/* XXX */
	const char *modstr[] = {
		[MOD_EXTERN]	= "extern",
		[MOD_STATIC]	= "static",
		[MOD_AUTO]	= "auto",
		[MOD_REGISTER]	= "register",

		[MOD_CONST]	= "const",
		[MOD_VOLATILE]	= "volatile",
	};
	const int modlen = 6;
	const char *basestr[] = {
		[TYPE_VOID]	= "void",
		[TYPE_CHAR]	= "char",
		[TYPE_SHORT]	= "short",
		[TYPE_INT]	= "int",
		[TYPE_LONG]	= "long",
		[TYPE_FLOAT]	= "float",
		[TYPE_DOUBLE]	= "double",
		[TYPE_SIGNED]	= "signed",
		[TYPE_UNSIGNED]	= "unsigned",
	};
	const int baselen = 9;
	struct strbuilder *b = strbuilder_create();
	int nmods = 0;
	for (int i = 0; i < modlen; i++) {
		if (1 << i & t->mod) {
			nmods++;
		}
	}
	for (int i = 0; i < modlen; i++) {
		int mod = 1 << i;
		if (mod & t->mod) {
			char *space = --nmods ? " " : "";
			strbuilder_printf(b, "%s%s", modstr[mod], space);
		}
	}
	switch (t->base) {
	case TYPE_TYPEDEF:
		ast_type_str_build_typedef(b, t);
		break;
	case TYPE_POINTER:
		ast_type_str_build_ptr(b, t);
		break;
	case TYPE_ARRAY:
		ast_type_str_build_arr(b, t);
		break;
	default:
		strbuilder_printf(b, basestr[t->base]);
		break;
	}
	return strbuilder_build(b);
}

static void
ast_type_str_build_ptr(struct strbuilder *b, struct ast_type *t)
{
	char *base = ast_type_str(t->u.ptr_type);
	bool space = t->u.ptr_type->base != TYPE_POINTER;
	strbuilder_printf(b, "%s%s*", base, space ? " " : "");
	free(base);
}

static void
ast_type_str_build_arr(struct strbuilder *b, struct ast_type *t)
{
	char *base = ast_type_str(t->u.arr.type);
	strbuilder_printf(b, "%s[%d]", base, t->u.arr.length);
	free(base);
}

static void
ast_type_str_build_typedef(struct strbuilder *b, struct ast_type *t)
{
	char *base = ast_type_str(t->u._typedef.type);
	strbuilder_printf(b, "typedef %s %s", base, t->u._typedef.type);
	free(base);
}

enum ast_type_base
ast_type_base(struct ast_type *t)
{
	return t->base;
}

struct ast_type *
ast_type_ptr_type(struct ast_type *t)
{
	assert(t->base == TYPE_POINTER);
	return t->u.ptr_type;
}

struct ast_variable *
ast_variable_create(char *name, struct ast_type *type)
{
	struct ast_variable *v = malloc(sizeof(struct ast_variable));
	v->name = name;
	v->type = type;
	return v;
}

void
ast_variable_destroy(struct ast_variable *v)
{
	ast_type_destroy(v->type);
	free(v->name);
	free(v);
}

struct ast_variable *
ast_variable_copy(struct ast_variable *v)
{
	assert(v);
	return ast_variable_create(
		dynamic_str(v->name), ast_type_copy(v->type)
	);
}

struct ast_variable **
ast_variables_copy(int n, struct ast_variable **v)
{
	assert(v);
	struct ast_variable **new = calloc(n, sizeof(struct variable *));
	for (int i = 0; i < n; i++) {
		new[i] = ast_variable_copy(v[i]);
	}
	return new;
}


char *
ast_variable_str(struct ast_variable *v)
{
	struct strbuilder *b = strbuilder_create();
	char *t = ast_type_str(v->type);
	
	strbuilder_printf(b, "%s %s", t, v->name);
	free(t);
	return strbuilder_build(b);
}

char *
ast_variable_name(struct ast_variable *v)
{
	return v->name;
}

struct ast_type *
ast_variable_type(struct ast_variable *v)
{
	return v->type;
}

struct ast_function *
ast_function_create(
	bool isaxiom,
	struct ast_type *ret,
	char *name, 
	int nparam,
	struct ast_variable **param,
	struct ast_block *abstract, 
	struct ast_block *body)
{
	struct ast_function *f = malloc(sizeof(struct ast_function));
	f->isaxiom = isaxiom;
	f->ret = ret;
	f->name = name;
	f->nparam = nparam;
	f->param = param;
	assert(abstract);
	f->abstract = abstract;
	f->body = body;
	return f;
}

void
ast_function_destroy(struct ast_function *f)
{
	ast_type_destroy(f->ret);
	for (int i = 0; i < f->nparam; i++) {
		ast_variable_destroy(f->param[i]);
	}
	ast_block_destroy(f->abstract);
	if (f->body) {
		ast_block_destroy(f->body);
	}
	free(f->param);
	free(f->name);
	free(f);
}

char *
ast_function_str(struct ast_function *f)
{
	struct strbuilder *b = strbuilder_create();
	strbuilder_printf(b, "func");
	if (f->isaxiom) {
		strbuilder_printf(b, " <axiom>");
	}
	strbuilder_printf(b, " `%s'", f->name);
	char *ret = ast_type_str(f->ret);
	strbuilder_printf(b, " returns %s ", ret);
	free(ret);
	strbuilder_printf(b, "takes [");
	for (int i = 0; i < f->nparam; i++) {
		char *v = ast_variable_str(f->param[i]);
		char *space = (i + 1 < f->nparam) ? ", " : "";
		strbuilder_printf(b, "%s%s", v, space);
		free(v);
	}
	strbuilder_printf(b, "] has abstract:\n%s", ast_block_str(f->abstract));
	return strbuilder_build(b);
}

struct ast_function *
ast_function_copy(struct ast_function *f)
{
	assert(f);
	struct ast_variable **param = malloc(sizeof(struct ast_variable *) * f->nparam);
	for (int i = 0; i < f->nparam; i++) {
		param[i] = ast_variable_copy(f->param[i]);
	}
	return ast_function_create(
		f->isaxiom,
		ast_type_copy(f->ret),
		dynamic_str(f->name),
		f->nparam,
		param,
		ast_block_copy(f->abstract),
		f->body ? ast_block_copy(f->body) : NULL
	);
}

bool
ast_function_isaxiom(struct ast_function *f)
{
	return f->isaxiom;
}

struct ast_type *
ast_function_type(struct ast_function *f)
{
	return f->ret;
}

struct ast_block *
ast_function_body(struct ast_function *f)
{
	assert(f->body);
	return f->body;
}

struct ast_block *
ast_function_abstract(struct ast_function *f)
{
	assert(f->abstract);
	return f->abstract;
}

int
ast_function_nparams(struct ast_function *f)
{
	return f->nparam;
}

struct ast_variable **
ast_function_params(struct ast_function *f)
{
	return f->param;
}

struct ast_externdecl *
ast_functiondecl_create(struct ast_function *f)
{
	struct ast_externdecl *decl = malloc(sizeof(struct ast_externdecl));
	decl->kind = EXTERN_FUNCTION;
	decl->u.function = f;
	return decl;
}

struct ast_externdecl *
ast_variabledecl_create(struct ast_variable *v)
{
	struct ast_externdecl *decl = malloc(sizeof(struct ast_externdecl));
	decl->kind = EXTERN_VARIABLE;
	decl->u.variable = v;
	return decl;
}

void
ast_externdecl_destroy(struct ast_externdecl *decl)
{
	if (decl->kind == EXTERN_FUNCTION) {
		ast_function_destroy(decl->u.function);
	} else if (decl->kind == EXTERN_VARIABLE) {
		ast_variable_destroy(decl->u.variable);
	}
	free(decl);
}

struct ast *
ast_create(struct ast_externdecl *decl)
{
	struct ast *node = calloc(1, sizeof(struct ast));
	return ast_append(node, decl);
}

void
ast_destroy(struct ast *node)
{
	for (int i = 0; i < node->n; i++) {
		ast_externdecl_destroy(node->decl[i]);
	}
	free(node->decl);
	free(node);
}

struct ast *
ast_append(struct ast *node, struct ast_externdecl *decl)
{
	node->decl = realloc(node->decl,
		sizeof(struct ast_externdecl *) * ++node->n);
	node->decl[node->n-1] = decl;
	return node;
}
