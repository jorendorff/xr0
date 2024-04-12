#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use libc::{calloc, free, malloc, realloc, strcmp};

use crate::ast::{
    ast_expr_constant_create, ast_expr_copy, ast_expr_destroy, ast_expr_equal,
    ast_expr_identifier_create, ast_expr_literal_create, ast_expr_member_create, ast_expr_str,
    ast_type_create_voidptr, ast_type_struct_complete, ast_type_struct_members,
    ast_variable_arr_copy, ast_variable_arr_destroy, ast_variable_arr_n, ast_variable_arr_v,
    ast_variable_name, ast_variable_type,
};
use crate::object::{
    object_abstractcopy, object_as_value, object_assign, object_copy, object_destroy,
    object_value_create,
};
use crate::state::location::{
    location_copy, location_destroy, location_references, location_referencesheap, location_str,
    location_transfigure,
};
use crate::state::state::{state_getext, state_vconst};
use crate::util::{
    dynamic_str, map, strbuilder_build, strbuilder_create, strbuilder_printf, strbuilder_putc,
};
use crate::{
    ast_type, ast_variable, ast_variable_arr, AstExpr as ast_expr, Location as location,
    Object as object, State as state, StrBuilder as strbuilder,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct value {
    pub type_0: value_type,
    pub c2rust_unnamed: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub ptr: C2RustUnnamed_3,
    pub n: *mut number,
    pub s: *mut libc::c_char,
    pub _struct: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub members: *mut ast_variable_arr,
    pub m: *mut map,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct number {
    pub type_0: number_type,
    pub c2rust_unnamed: C2RustUnnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ranges: *mut number_range_arr,
    pub computation: *mut ast_expr,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct number_range_arr {
    pub n: libc::c_int,
    pub range: *mut *mut number_range,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct number_range {
    pub lower: *mut number_value,
    pub upper: *mut number_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct number_value {
    pub type_0: number_value_type,
    pub c2rust_unnamed: C2RustUnnamed_2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub constant: libc::c_int,
    pub max: bool,
}
pub type number_value_type = libc::c_uint;
pub const NUMBER_VALUE_LIMIT: number_value_type = 1;
pub const NUMBER_VALUE_CONSTANT: number_value_type = 0;
pub type number_type = libc::c_uint;
pub const NUMBER_COMPUTED: number_type = 1;
pub const NUMBER_RANGES: number_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub isindefinite: bool,
    pub c2rust_unnamed: C2RustUnnamed_4,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_4 {
    pub loc: *mut location,
    pub n: *mut number,
}
pub type value_type = libc::c_uint;
pub const VALUE_STRUCT: value_type = 4;
pub const VALUE_LITERAL: value_type = 3;
pub const VALUE_INT: value_type = 2;
pub const VALUE_PTR: value_type = 1;
pub const VALUE_SYNC: value_type = 0;

pub unsafe fn value_ptr_create(mut loc: *mut location) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_PTR;
    (*v).c2rust_unnamed.ptr.isindefinite = false;
    (*v).c2rust_unnamed.ptr.c2rust_unnamed.loc = loc;
    return v;
}

pub unsafe fn value_ptr_indefinite_create() -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_PTR;
    (*v).c2rust_unnamed.ptr.isindefinite = 1 as libc::c_int != 0;
    (*v).c2rust_unnamed.ptr.c2rust_unnamed.n = number_indefinite_create();
    return v;
}
unsafe fn ptr_referencesheap(mut v: *mut value, mut s: *mut state) -> bool {
    return !(*v).c2rust_unnamed.ptr.isindefinite
        && location_referencesheap((*v).c2rust_unnamed.ptr.c2rust_unnamed.loc, s) as libc::c_int
            != 0;
}

pub unsafe fn value_ptr_copy(mut old: *mut value) -> *mut value {
    let mut new: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if new.is_null() {
        panic!();
    }
    (*new).type_0 = VALUE_PTR;
    (*new).c2rust_unnamed.ptr.isindefinite = (*old).c2rust_unnamed.ptr.isindefinite;
    if (*old).c2rust_unnamed.ptr.isindefinite {
        (*new).c2rust_unnamed.ptr.c2rust_unnamed.n =
            number_copy((*old).c2rust_unnamed.ptr.c2rust_unnamed.n);
    } else {
        (*new).c2rust_unnamed.ptr.c2rust_unnamed.loc =
            location_copy((*old).c2rust_unnamed.ptr.c2rust_unnamed.loc);
    }
    return new;
}

pub unsafe fn value_ptr_sprint(mut v: *mut value, mut b: *mut strbuilder) {
    let mut s: *mut libc::c_char = if (*v).c2rust_unnamed.ptr.isindefinite as libc::c_int != 0 {
        number_str((*v).c2rust_unnamed.ptr.c2rust_unnamed.n)
    } else {
        location_str((*v).c2rust_unnamed.ptr.c2rust_unnamed.loc)
    };
    strbuilder_printf(b, b"ptr:%s\0" as *const u8 as *const libc::c_char, s);
    free(s as *mut libc::c_void);
}

pub unsafe fn value_int_create(mut val: libc::c_int) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_INT;
    (*v).c2rust_unnamed.n = number_single_create(val);
    return v;
}

pub unsafe fn value_literal_create(mut lit: *mut libc::c_char) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_LITERAL;
    (*v).c2rust_unnamed.s = dynamic_str(lit);
    return v;
}

pub unsafe fn value_transfigure(
    mut v: *mut value,
    mut compare: *mut state,
    mut islval: bool,
) -> *mut value {
    match (*v).type_0 as libc::c_uint {
        0 | 3 => {
            return if islval as libc::c_int != 0 {
                0 as *mut value
            } else {
                v
            }
        }
        4 => {
            panic!();
        }
        2 => {}
        1 => return location_transfigure(value_as_location(v), compare),
        _ => {
            panic!();
        }
    }
    return if islval as libc::c_int != 0 {
        0 as *mut value
    } else {
        state_vconst(
            compare,
            ast_type_create_voidptr(),
            0 as *mut libc::c_char,
            false,
        )
    };
}

pub unsafe fn value_int_ne_create(mut not_val: libc::c_int) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_INT;
    (*v).c2rust_unnamed.n = number_ne_create(not_val);
    return v;
}

pub unsafe fn value_int_range_create(mut lw: libc::c_int, mut excl_up: libc::c_int) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_INT;
    (*v).c2rust_unnamed.n = number_with_range_create(lw, excl_up);
    return v;
}

pub unsafe fn value_int_indefinite_create() -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_INT;
    (*v).c2rust_unnamed.n = number_indefinite_create();
    return v;
}

pub unsafe fn value_int_lw(mut v: *mut value) -> libc::c_int {
    return number_range_lw((*v).c2rust_unnamed.n);
}

pub unsafe fn value_int_up(mut v: *mut value) -> libc::c_int {
    return number_range_up((*v).c2rust_unnamed.n);
}

pub unsafe fn value_sync_create(mut e: *mut ast_expr) -> *mut value {
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_SYNC;
    (*v).c2rust_unnamed.n = number_computed_create(e);
    return v;
}

pub unsafe fn value_sync_copy(mut old: *mut value) -> *mut value {
    if !((*old).type_0 as libc::c_uint == VALUE_SYNC as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    let mut new: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if new.is_null() {
        panic!();
    }
    (*new).type_0 = VALUE_SYNC;
    (*new).c2rust_unnamed.n = number_copy((*old).c2rust_unnamed.n);
    return new;
}

pub unsafe fn value_int_copy(mut old: *mut value) -> *mut value {
    if !((*old).type_0 as libc::c_uint == VALUE_INT as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    let mut new: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if new.is_null() {
        panic!();
    }
    (*new).type_0 = VALUE_INT;
    (*new).c2rust_unnamed.n = number_copy((*old).c2rust_unnamed.n);
    return new;
}

pub unsafe fn value_struct_create(mut t: *mut ast_type) -> *mut value {
    let mut members: *mut ast_variable_arr = ast_variable_arr_copy(ast_type_struct_members(t));
    if members.is_null() {
        panic!();
    }
    let mut v: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = VALUE_STRUCT;
    (*v).c2rust_unnamed._struct.members = members;
    (*v).c2rust_unnamed._struct.m = Box::into_raw(frommembers(members));
    return v;
}

pub unsafe fn value_struct_indefinite_create(
    mut t: *mut ast_type,
    mut s: *mut state,
    mut comment: *mut libc::c_char,
    mut persist: bool,
) -> *mut value {
    t = ast_type_struct_complete(t, state_getext(s));
    if (ast_type_struct_members(t)).is_null() {
        panic!();
    }
    let mut v: *mut value = value_struct_create(t);
    let mut n: libc::c_int = ast_variable_arr_n((*v).c2rust_unnamed._struct.members);
    let mut var: *mut *mut ast_variable = ast_variable_arr_v((*v).c2rust_unnamed._struct.members);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut field: *mut libc::c_char = ast_variable_name(*var.offset(i as isize));
        let mut obj: *mut object = (*(*v).c2rust_unnamed._struct.m).get(field) as *mut object;
        let mut b: *mut strbuilder = strbuilder_create();
        strbuilder_printf(
            b,
            b"%s.%s\0" as *const u8 as *const libc::c_char,
            comment,
            field,
        );
        object_assign(
            obj,
            state_vconst(
                s,
                ast_variable_type(*var.offset(i as isize)),
                strbuilder_build(b),
                persist,
            ),
        );
        i += 1;
    }
    return v;
}

pub unsafe fn value_pf_augment(mut old: *mut value, mut root: *mut ast_expr) -> *mut value {
    if !value_isstruct(old) {
        panic!();
    }
    let mut v: *mut value = value_copy(old);
    let mut n: libc::c_int = ast_variable_arr_n((*v).c2rust_unnamed._struct.members);
    let mut var: *mut *mut ast_variable = ast_variable_arr_v((*v).c2rust_unnamed._struct.members);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut field: *mut libc::c_char = ast_variable_name(*var.offset(i as isize));
        let mut obj: *mut object = (*(*v).c2rust_unnamed._struct.m).get(field) as *mut object;
        let mut obj_value: *mut value = object_as_value(obj);
        if !obj_value.is_null() {
            if value_issync(obj_value) {
                object_assign(
                    obj,
                    value_sync_create(ast_expr_member_create(
                        ast_expr_copy(root),
                        dynamic_str(field),
                    )),
                );
            }
        }
        i += 1;
    }
    return v;
}

pub unsafe fn value_isstruct(mut v: *mut value) -> bool {
    return (*v).type_0 as libc::c_uint == VALUE_STRUCT as libc::c_int as libc::c_uint;
}
unsafe fn frommembers(mut members: *mut ast_variable_arr) -> Box<map> {
    let mut m = map::new();
    let mut n: libc::c_int = ast_variable_arr_n(members);
    let mut v: *mut *mut ast_variable = ast_variable_arr_v(members);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        m.set(
            dynamic_str(ast_variable_name(*v.offset(i as isize))),
            object_value_create(ast_expr_constant_create(0 as libc::c_int), 0 as *mut value)
                as *const libc::c_void,
        );
        i += 1;
    }
    return m;
}
unsafe fn destroymembers(m: &map) {
    for p in m.values() {
        object_destroy(p as *mut object);
    }
}

pub unsafe fn value_struct_copy(mut old: *mut value) -> *mut value {
    let mut new: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if new.is_null() {
        panic!();
    }
    (*new).type_0 = VALUE_STRUCT;
    (*new).c2rust_unnamed._struct.members =
        ast_variable_arr_copy((*old).c2rust_unnamed._struct.members);
    (*new).c2rust_unnamed._struct.m = Box::into_raw(copymembers(&*(*old).c2rust_unnamed._struct.m));
    return new;
}
unsafe fn copymembers(mut old: &map) -> Box<map> {
    let mut new = map::new();
    for (k, v) in old.pairs() {
        new.set(
            dynamic_str(k),
            object_copy(v as *mut object) as *const libc::c_void,
        );
    }
    return new;
}

pub unsafe fn value_struct_abstractcopy(mut old: *mut value, mut s: *mut state) -> *mut value {
    let mut new: *mut value = malloc(::core::mem::size_of::<value>()) as *mut value;
    if new.is_null() {
        panic!();
    }
    (*new).type_0 = VALUE_STRUCT;
    (*new).c2rust_unnamed._struct.members =
        ast_variable_arr_copy((*old).c2rust_unnamed._struct.members);
    (*new).c2rust_unnamed._struct.m =
        Box::into_raw(abstractcopymembers(&*(*old).c2rust_unnamed._struct.m, s));
    return new;
}
unsafe fn abstractcopymembers(old: &map, mut s: *mut state) -> Box<map> {
    let mut new = map::new();
    for (k, v) in old.pairs() {
        new.set(
            dynamic_str(k),
            object_abstractcopy(v as *mut object, s) as *const libc::c_void,
        );
    }
    return new;
}

pub unsafe fn value_struct_membertype(
    mut v: *mut value,
    mut member: *mut libc::c_char,
) -> *mut ast_type {
    let mut members: *mut ast_variable_arr = (*v).c2rust_unnamed._struct.members;
    let mut n: libc::c_int = ast_variable_arr_n(members);
    let mut var: *mut *mut ast_variable = ast_variable_arr_v(members);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        if strcmp(member, ast_variable_name(*var.offset(i as isize))) == 0 as libc::c_int {
            return ast_variable_type(*var.offset(i as isize));
        }
        i += 1;
    }
    0 as *mut ast_type
}

pub unsafe fn value_struct_member(mut v: *mut value, mut member: *mut libc::c_char) -> *mut object {
    return (*(*v).c2rust_unnamed._struct.m).get(member) as *mut object;
}
unsafe fn struct_referencesheap(mut v: *mut value, mut s: *mut state) -> bool {
    let mut m: &map = &*(*v).c2rust_unnamed._struct.m;
    for p in m.values() {
        let mut val: *mut value = object_as_value(p as *mut object);
        if !val.is_null() && value_referencesheap(val, s) as libc::c_int != 0 {
            return 1 as libc::c_int != 0;
        }
    }
    false
}

pub unsafe fn value_struct_sprint(mut v: *mut value, mut b: *mut strbuilder) {
    strbuilder_printf(b, b"struct:{\0" as *const u8 as *const libc::c_char);
    let mut members: *mut ast_variable_arr = (*v).c2rust_unnamed._struct.members;
    let mut n: libc::c_int = ast_variable_arr_n(members);
    let mut var: *mut *mut ast_variable = ast_variable_arr_v(members);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut f: *mut libc::c_char = ast_variable_name(*var.offset(i as isize));
        let mut val: *mut value =
            object_as_value((*(*v).c2rust_unnamed._struct.m).get(f) as *mut object);
        let mut val_str: *mut libc::c_char = if !val.is_null() {
            value_str(val)
        } else {
            dynamic_str(b"\0" as *const u8 as *const libc::c_char)
        };
        strbuilder_printf(
            b,
            b".%s = <%s>%s\0" as *const u8 as *const libc::c_char,
            f,
            val_str,
            if (i + 1 as libc::c_int) < n {
                b", \0" as *const u8 as *const libc::c_char
            } else {
                b"\0" as *const u8 as *const libc::c_char
            },
        );
        free(val_str as *mut libc::c_void);
        i += 1;
    }
    strbuilder_printf(b, b"}\0" as *const u8 as *const libc::c_char);
}

pub unsafe fn value_int_sprint(mut v: *mut value, mut b: *mut strbuilder) {
    strbuilder_printf(
        b,
        b"int:%s\0" as *const u8 as *const libc::c_char,
        number_str((*v).c2rust_unnamed.n),
    );
}

pub unsafe fn value_sync_sprint(mut v: *mut value, mut b: *mut strbuilder) {
    strbuilder_printf(
        b,
        b"comp:%s\0" as *const u8 as *const libc::c_char,
        number_str((*v).c2rust_unnamed.n),
    );
}

pub unsafe fn value_copy(mut v: *mut value) -> *mut value {
    match (*v).type_0 as libc::c_uint {
        0 => value_sync_copy(v),
        1 => value_ptr_copy(v),
        2 => value_int_copy(v),
        3 => value_literal_create((*v).c2rust_unnamed.s),
        4 => value_struct_copy(v),
        _ => panic!(),
    }
}

pub unsafe fn value_abstractcopy(mut v: *mut value, mut s: *mut state) -> *mut value {
    if !value_referencesheap(v, s) {
        return 0 as *mut value;
    }
    match (*v).type_0 as libc::c_uint {
        1 => value_copy(v),
        4 => value_struct_abstractcopy(v, s),
        _ => panic!(),
    }
}

pub unsafe fn value_destroy(mut v: *mut value) {
    match (*v).type_0 as libc::c_uint {
        0 => {
            number_destroy((*v).c2rust_unnamed.n);
        }
        1 => {
            if (*v).c2rust_unnamed.ptr.isindefinite {
                number_destroy((*v).c2rust_unnamed.ptr.c2rust_unnamed.n);
            } else if !((*v).c2rust_unnamed.ptr.c2rust_unnamed.loc).is_null() {
                location_destroy((*v).c2rust_unnamed.ptr.c2rust_unnamed.loc);
            }
        }
        2 => {
            number_destroy((*v).c2rust_unnamed.n);
        }
        3 => {
            free((*v).c2rust_unnamed.s as *mut libc::c_void);
        }
        4 => {
            ast_variable_arr_destroy((*v).c2rust_unnamed._struct.members);
            destroymembers(&*(*v).c2rust_unnamed._struct.m);
            Box::from_raw((*v).c2rust_unnamed._struct.m).destroy();
        }
        _ => panic!(),
    }
    free(v as *mut libc::c_void);
}

pub unsafe fn value_str(mut v: *mut value) -> *mut libc::c_char {
    let mut b: *mut strbuilder = strbuilder_create();
    match (*v).type_0 as libc::c_uint {
        0 => {
            value_sync_sprint(v, b);
        }
        1 => {
            value_ptr_sprint(v, b);
        }
        2 => {
            value_int_sprint(v, b);
        }
        3 => {
            strbuilder_printf(
                b,
                b"\"%s\"\0" as *const u8 as *const libc::c_char,
                (*v).c2rust_unnamed.s,
            );
        }
        4 => {
            value_struct_sprint(v, b);
        }
        _ => panic!(),
    }
    return strbuilder_build(b);
}

pub unsafe fn value_islocation(mut v: *mut value) -> bool {
    if v.is_null() {
        panic!();
    }
    return (*v).type_0 as libc::c_uint == VALUE_PTR as libc::c_int as libc::c_uint
        && !(*v).c2rust_unnamed.ptr.isindefinite;
}

pub unsafe fn value_as_location(mut v: *mut value) -> *mut location {
    if !value_islocation(v) {
        panic!();
    }
    return (*v).c2rust_unnamed.ptr.c2rust_unnamed.loc;
}

pub unsafe fn value_referencesheap(mut v: *mut value, mut s: *mut state) -> bool {
    match (*v).type_0 as libc::c_uint {
        1 => return ptr_referencesheap(v, s),
        4 => return struct_referencesheap(v, s),
        _ => return false,
    };
}

pub unsafe fn value_as_constant(mut v: *mut value) -> libc::c_int {
    if !((*v).type_0 as libc::c_uint == VALUE_INT as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    return number_as_constant((*v).c2rust_unnamed.n);
}

pub unsafe fn value_isconstant(mut v: *mut value) -> bool {
    if (*v).type_0 as libc::c_uint != VALUE_INT as libc::c_int as libc::c_uint {
        return false;
    }
    return number_isconstant((*v).c2rust_unnamed.n);
}

pub unsafe fn value_issync(mut v: *mut value) -> bool {
    if (*v).type_0 as libc::c_uint != VALUE_SYNC as libc::c_int as libc::c_uint {
        return false;
    }
    return number_issync((*v).c2rust_unnamed.n);
}

pub unsafe fn value_as_sync(mut v: *mut value) -> *mut ast_expr {
    if !((*v).type_0 as libc::c_uint == VALUE_SYNC as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    return number_as_sync((*v).c2rust_unnamed.n);
}

pub unsafe fn value_to_expr(mut v: *mut value) -> *mut ast_expr {
    match (*v).type_0 as libc::c_uint {
        1 => ast_expr_identifier_create(value_str(v)),
        3 => ast_expr_copy(value_as_literal(v)),
        0 => ast_expr_copy(value_as_sync(v)),
        2 => number_to_expr((*v).c2rust_unnamed.n),
        _ => panic!(),
    }
}

pub unsafe fn value_isliteral(mut v: *mut value) -> bool {
    if (*v).type_0 as libc::c_uint != VALUE_LITERAL as libc::c_int as libc::c_uint {
        return false;
    }
    return 1 as libc::c_int != 0;
}

pub unsafe fn value_as_literal(mut v: *mut value) -> *mut ast_expr {
    if !((*v).type_0 as libc::c_uint == VALUE_LITERAL as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    return ast_expr_literal_create((*v).c2rust_unnamed.s);
}

pub unsafe fn value_type(mut v: *mut value) -> value_type {
    return (*v).type_0;
}

pub unsafe fn value_references(
    mut v: *mut value,
    mut loc: *mut location,
    mut s: *mut state,
) -> bool {
    match (*v).type_0 as libc::c_uint {
        1 => {
            return !(*v).c2rust_unnamed.ptr.isindefinite
                && location_references((*v).c2rust_unnamed.ptr.c2rust_unnamed.loc, loc, s)
                    as libc::c_int
                    != 0;
        }
        4 => return struct_references(v, loc, s),
        _ => return false,
    };
}

unsafe fn struct_references(mut v: *mut value, mut loc: *mut location, mut s: *mut state) -> bool {
    let mut m: &map = &*(*v).c2rust_unnamed._struct.m;
    for p in m.values() {
        let mut val: *mut value = object_as_value(p as *mut object);
        if !val.is_null() && value_references(val, loc, s) as libc::c_int != 0 {
            return true;
        }
    }
    false
}

pub unsafe fn values_comparable(mut v1: *mut value, mut v2: *mut value) -> bool {
    (*v1).type_0 as libc::c_uint == (*v2).type_0 as libc::c_uint
}

pub unsafe fn value_equal(mut v1: *mut value, mut v2: *mut value) -> bool {
    if !((*v1).type_0 as libc::c_uint == (*v2).type_0 as libc::c_uint) {
        panic!();
    }
    match (*v1).type_0 as libc::c_uint {
        3 => strcmp((*v1).c2rust_unnamed.s, (*v2).c2rust_unnamed.s) == 0,
        2 | 0 => number_equal((*v1).c2rust_unnamed.n, (*v2).c2rust_unnamed.n),
        _ => panic!(),
    }
}

pub unsafe fn value_assume(mut v: *mut value, mut value: bool) -> bool {
    match (*v).type_0 as libc::c_uint {
        2 => number_assume((*v).c2rust_unnamed.n, value),
        1 => {
            if !(*v).c2rust_unnamed.ptr.isindefinite {
                panic!();
            }
            number_assume((*v).c2rust_unnamed.ptr.c2rust_unnamed.n, value)
        }
        _ => panic!(),
    }
}

pub unsafe fn number_ranges_create(mut ranges: *mut number_range_arr) -> *mut number {
    let mut num: *mut number = calloc(1, ::core::mem::size_of::<number>()) as *mut number;
    (*num).type_0 = NUMBER_RANGES;
    (*num).c2rust_unnamed.ranges = ranges;
    return num;
}

pub unsafe fn number_single_create(mut val: libc::c_int) -> *mut number {
    return number_ranges_create(number_range_arr_single_create(val));
}

pub unsafe fn number_range_arr_single_create(mut val: libc::c_int) -> *mut number_range_arr {
    let mut arr: *mut number_range_arr = number_range_arr_create();
    number_range_arr_append(
        arr,
        number_range_create(
            number_value_constant_create(val),
            number_value_constant_create(val + 1 as libc::c_int),
        ),
    );
    return arr;
}

pub unsafe fn number_computed_create(mut e: *mut ast_expr) -> *mut number {
    let mut num: *mut number = calloc(1, ::core::mem::size_of::<number>()) as *mut number;
    (*num).type_0 = NUMBER_COMPUTED;
    (*num).c2rust_unnamed.computation = e;
    return num;
}

pub unsafe fn number_range_arr_ne_create(mut val: libc::c_int) -> *mut number_range_arr {
    let mut arr: *mut number_range_arr = number_range_arr_create();
    number_range_arr_append(
        arr,
        number_range_create(number_value_min_create(), number_value_constant_create(val)),
    );
    number_range_arr_append(
        arr,
        number_range_create(
            number_value_constant_create(val + 1 as libc::c_int),
            number_value_max_create(),
        ),
    );
    return arr;
}

pub unsafe fn number_ne_create(mut val: libc::c_int) -> *mut number {
    return number_ranges_create(number_range_arr_ne_create(val));
}

pub unsafe fn number_with_range_create(
    mut lw: libc::c_int,
    mut excl_up: libc::c_int,
) -> *mut number {
    let mut arr: *mut number_range_arr = number_range_arr_create();
    number_range_arr_append(
        arr,
        number_range_create(
            number_value_constant_create(lw),
            number_value_constant_create(excl_up),
        ),
    );
    return number_ranges_create(arr);
}

pub unsafe fn number_indefinite_create() -> *mut number {
    let mut arr: *mut number_range_arr = number_range_arr_create();
    number_range_arr_append(
        arr,
        number_range_create(number_value_min_create(), number_value_max_create()),
    );
    return number_ranges_create(arr);
}

pub unsafe fn number_range_lw(mut n: *mut number) -> libc::c_int {
    if !(number_range_arr_n((*n).c2rust_unnamed.ranges) == 1 as libc::c_int) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    let mut r: *mut number_range =
        *(number_range_arr_range((*n).c2rust_unnamed.ranges)).offset(0 as libc::c_int as isize);
    return number_value_as_constant(number_range_lower(r));
}

pub unsafe fn number_range_up(mut n: *mut number) -> libc::c_int {
    if !(number_range_arr_n((*n).c2rust_unnamed.ranges) == 1 as libc::c_int) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    let mut r: *mut number_range =
        *(number_range_arr_range((*n).c2rust_unnamed.ranges)).offset(0 as libc::c_int as isize);
    return number_value_as_constant(number_range_upper(r));
}

pub unsafe fn number_destroy(mut n: *mut number) {
    match (*n).type_0 as libc::c_uint {
        0 => {
            number_range_arr_destroy((*n).c2rust_unnamed.ranges);
        }
        1 => {
            ast_expr_destroy((*n).c2rust_unnamed.computation);
        }
        _ => panic!(),
    };
}

pub unsafe fn number_ranges_sprint(mut num: *mut number) -> *mut libc::c_char {
    if !((*num).type_0 as libc::c_uint == NUMBER_RANGES as libc::c_int as libc::c_uint)
        as libc::c_int as libc::c_long
        != 0
    {
        panic!();
    }
    let mut b: *mut strbuilder = strbuilder_create();
    let mut n: libc::c_int = number_range_arr_n((*num).c2rust_unnamed.ranges);
    let mut range: *mut *mut number_range = number_range_arr_range((*num).c2rust_unnamed.ranges);
    strbuilder_putc(b, '{' as i32 as libc::c_char);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut r: *mut libc::c_char = number_range_str(*range.offset(i as isize));
        strbuilder_printf(
            b,
            b"%s%s\0" as *const u8 as *const libc::c_char,
            r,
            if (i + 1 as libc::c_int) < n {
                b", \0" as *const u8 as *const libc::c_char
            } else {
                b"\0" as *const u8 as *const libc::c_char
            },
        );
        free(r as *mut libc::c_void);
        i += 1;
    }
    strbuilder_putc(b, '}' as i32 as libc::c_char);
    return strbuilder_build(b);
}

pub unsafe fn number_str(mut num: *mut number) -> *mut libc::c_char {
    match (*num).type_0 as libc::c_uint {
        0 => number_ranges_sprint(num),
        1 => ast_expr_str((*num).c2rust_unnamed.computation),
        _ => panic!(),
    }
}

pub unsafe fn number_equal(mut n1: *mut number, mut n2: *mut number) -> bool {
    if !((*n1).type_0 as libc::c_uint == (*n2).type_0 as libc::c_uint) {
        panic!();
    }
    match (*n1).type_0 as libc::c_uint {
        0 => number_ranges_equal(n1, n2),
        1 => ast_expr_equal(
            (*n1).c2rust_unnamed.computation,
            (*n2).c2rust_unnamed.computation,
        ),

        _ => panic!(),
    }
}

pub unsafe fn number_ranges_equal(mut n1: *mut number, mut n2: *mut number) -> bool {
    if !((*n1).type_0 as libc::c_uint == (*n2).type_0 as libc::c_uint
        && (*n1).type_0 as libc::c_uint == NUMBER_RANGES as libc::c_int as libc::c_uint)
        as libc::c_int as libc::c_long
        != 0
    {
        panic!();
    }
    let mut len: libc::c_int = number_range_arr_n((*n1).c2rust_unnamed.ranges);
    if len != number_range_arr_n((*n2).c2rust_unnamed.ranges) {
        return false;
    }
    let mut n1_r: *mut *mut number_range = number_range_arr_range((*n1).c2rust_unnamed.ranges);
    let mut n2_r: *mut *mut number_range = number_range_arr_range((*n2).c2rust_unnamed.ranges);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < len {
        if !number_range_equal(*n1_r.offset(i as isize), *n2_r.offset(i as isize)) {
            return false;
        }
        i += 1;
    }
    return 1 as libc::c_int != 0;
}
unsafe fn number_assume(mut n: *mut number, mut value: bool) -> bool {
    if !((*n).type_0 as libc::c_uint == NUMBER_RANGES as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    if !number_range_arr_canbe((*n).c2rust_unnamed.ranges, value) {
        return false;
    }
    (*n).c2rust_unnamed.ranges = number_range_assumed_value(value);
    return 1 as libc::c_int != 0;
}
unsafe fn number_range_assumed_value(mut value: bool) -> *mut number_range_arr {
    if value {
        return number_range_arr_ne_create(0 as libc::c_int);
    } else {
        return number_range_arr_single_create(0 as libc::c_int);
    };
}

pub unsafe fn number_isconstant(mut n: *mut number) -> bool {
    if !((*n).type_0 as libc::c_uint == NUMBER_RANGES as libc::c_int as libc::c_uint) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    return number_range_arr_n((*n).c2rust_unnamed.ranges) == 1 as libc::c_int
        && number_range_issingle(
            *(number_range_arr_range((*n).c2rust_unnamed.ranges)).offset(0 as libc::c_int as isize),
        ) as libc::c_int
            != 0;
}

pub unsafe fn number_as_constant(mut n: *mut number) -> libc::c_int {
    if !((*n).type_0 as libc::c_uint == NUMBER_RANGES as libc::c_int as libc::c_uint
        && number_range_arr_n((*n).c2rust_unnamed.ranges) == 1 as libc::c_int) as libc::c_int
        as libc::c_long
        != 0
    {
        panic!();
    }
    return number_range_as_constant(
        *(number_range_arr_range((*n).c2rust_unnamed.ranges)).offset(0 as libc::c_int as isize),
    );
}

pub unsafe fn number_issync(mut n: *mut number) -> bool {
    return (*n).type_0 as libc::c_uint == NUMBER_COMPUTED as libc::c_int as libc::c_uint;
}

pub unsafe fn number_as_sync(mut n: *mut number) -> *mut ast_expr {
    if !((*n).type_0 as libc::c_uint == NUMBER_COMPUTED as libc::c_int as libc::c_uint)
        as libc::c_int as libc::c_long
        != 0
    {
        panic!();
    }
    return (*n).c2rust_unnamed.computation;
}

pub unsafe fn number_to_expr(mut n: *mut number) -> *mut ast_expr {
    match (*n).type_0 as libc::c_uint {
        0 => number_ranges_to_expr((*n).c2rust_unnamed.ranges),
        1 => ast_expr_copy(number_as_sync(n)),
        _ => panic!(),
    }
}

pub unsafe fn number_copy(mut num: *mut number) -> *mut number {
    match (*num).type_0 as libc::c_uint {
        0 => number_ranges_create(number_range_arr_copy((*num).c2rust_unnamed.ranges)),
        1 => number_computed_create(ast_expr_copy((*num).c2rust_unnamed.computation)),
        _ => panic!(),
    }
}

pub unsafe fn number_range_arr_create() -> *mut number_range_arr {
    let mut arr: *mut number_range_arr =
        calloc(1, ::core::mem::size_of::<number_range_arr>()) as *mut number_range_arr;
    if arr.is_null() {
        panic!();
    }
    return arr;
}

pub unsafe fn number_range_arr_destroy(mut arr: *mut number_range_arr) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*arr).n {
        number_range_destroy(*((*arr).range).offset(i as isize));
        i += 1;
    }
    free((*arr).range as *mut libc::c_void);
    free(arr as *mut libc::c_void);
}

pub unsafe fn number_range_arr_n(mut arr: *mut number_range_arr) -> libc::c_int {
    return (*arr).n;
}

pub unsafe fn number_range_arr_range(mut arr: *mut number_range_arr) -> *mut *mut number_range {
    return (*arr).range;
}

pub unsafe fn number_range_arr_append(
    mut arr: *mut number_range_arr,
    mut r: *mut number_range,
) -> libc::c_int {
    (*arr).n += 1;
    (*arr).range = realloc(
        (*arr).range as *mut libc::c_void,
        (::core::mem::size_of::<*mut number_range>()).wrapping_mul((*arr).n as usize),
    ) as *mut *mut number_range;
    if ((*arr).range).is_null() {
        panic!();
    }
    let mut loc: libc::c_int = (*arr).n - 1 as libc::c_int;
    let ref mut fresh0 = *((*arr).range).offset(loc as isize);
    *fresh0 = r;
    return loc;
}

pub unsafe fn number_range_arr_copy(mut old: *mut number_range_arr) -> *mut number_range_arr {
    let mut new: *mut number_range_arr = number_range_arr_create();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*old).n {
        number_range_arr_append(new, number_range_copy(*((*old).range).offset(i as isize)));
        i += 1;
    }
    new
}

pub unsafe fn number_ranges_to_expr(mut arr: *mut number_range_arr) -> *mut ast_expr {
    if !(number_range_arr_n(arr) == 1 as libc::c_int) {
        panic!();
    }
    ast_expr_constant_create(number_range_as_constant(
        *((*arr).range).offset(0 as libc::c_int as isize),
    ))
}
unsafe fn number_range_arr_canbe(mut arr: *mut number_range_arr, mut value: bool) -> bool {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*arr).n {
        if number_range_canbe(*((*arr).range).offset(i as isize), value) {
            return 1 as libc::c_int != 0;
        }
        i += 1;
    }
    return false;
}

pub unsafe fn number_range_create(
    mut lw: *mut number_value,
    mut up: *mut number_value,
) -> *mut number_range {
    let mut r: *mut number_range =
        malloc(::core::mem::size_of::<number_range>()) as *mut number_range;
    (*r).lower = lw;
    (*r).upper = up;
    return r;
}

pub unsafe fn number_range_destroy(mut r: *mut number_range) {
    number_value_destroy((*r).lower);
    number_value_destroy((*r).upper);
    free(r as *mut libc::c_void);
}

pub unsafe fn number_range_lower(mut r: *mut number_range) -> *mut number_value {
    return (*r).lower;
}

pub unsafe fn number_range_upper(mut r: *mut number_range) -> *mut number_value {
    return (*r).upper;
}

pub unsafe fn number_range_str(mut r: *mut number_range) -> *mut libc::c_char {
    let mut b: *mut strbuilder = strbuilder_create();
    if number_range_issingle(r) {
        strbuilder_printf(
            b,
            b"%s\0" as *const u8 as *const libc::c_char,
            number_value_str((*r).lower),
        );
    } else {
        strbuilder_printf(
            b,
            b"%s:%s\0" as *const u8 as *const libc::c_char,
            number_value_str((*r).lower),
            number_value_str((*r).upper),
        );
    }
    return strbuilder_build(b);
}

pub unsafe fn number_range_copy(mut r: *mut number_range) -> *mut number_range {
    return number_range_create(number_value_copy((*r).lower), number_value_copy((*r).upper));
}
unsafe fn number_range_canbe(mut r: *mut number_range, mut value: bool) -> bool {
    if value {
        if number_value_equal((*r).lower, (*r).upper) {
            return false;
        }
        return number_value_le_constant((*r).lower, -(1 as libc::c_int)) as libc::c_int != 0
            || constant_le_number_value(1 as libc::c_int, (*r).lower) as libc::c_int != 0;
    } else {
        return number_value_le_constant((*r).lower, 0 as libc::c_int) as libc::c_int != 0
            && constant_le_number_value(1 as libc::c_int, (*r).upper) as libc::c_int != 0;
    };
}

pub unsafe fn number_range_issingle(mut r: *mut number_range) -> bool {
    return number_values_aresingle((*r).lower, (*r).upper);
}

pub unsafe fn number_range_equal(mut r1: *mut number_range, mut r2: *mut number_range) -> bool {
    return number_value_equal((*r1).lower, (*r2).lower) as libc::c_int != 0
        && number_value_equal((*r1).upper, (*r2).upper) as libc::c_int != 0;
}

pub unsafe fn number_range_as_constant(mut r: *mut number_range) -> libc::c_int {
    if !number_range_issingle(r) {
        panic!();
    }
    return number_value_as_constant((*r).lower);
}

pub unsafe fn number_value_constant_create(mut constant: libc::c_int) -> *mut number_value {
    let mut v: *mut number_value =
        malloc(::core::mem::size_of::<number_value>()) as *mut number_value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = NUMBER_VALUE_CONSTANT;
    (*v).c2rust_unnamed.constant = constant;
    return v;
}

pub unsafe fn number_value_limit_create(mut max: bool) -> *mut number_value {
    let mut v: *mut number_value =
        malloc(::core::mem::size_of::<number_value>()) as *mut number_value;
    if v.is_null() {
        panic!();
    }
    (*v).type_0 = NUMBER_VALUE_LIMIT;
    (*v).c2rust_unnamed.max = max;
    return v;
}

pub unsafe fn number_value_min_create() -> *mut number_value {
    return number_value_limit_create(false);
}

pub unsafe fn number_value_max_create() -> *mut number_value {
    return number_value_limit_create(1 as libc::c_int != 0);
}

pub unsafe fn number_value_destroy(mut v: *mut number_value) {
    free(v as *mut libc::c_void);
}

pub unsafe fn number_value_str(mut v: *mut number_value) -> *mut libc::c_char {
    let mut b: *mut strbuilder = strbuilder_create();
    match (*v).type_0 as libc::c_uint {
        0 => {
            strbuilder_printf(
                b,
                b"%d\0" as *const u8 as *const libc::c_char,
                (*v).c2rust_unnamed.constant,
            );
        }
        1 => {
            strbuilder_printf(
                b,
                b"%s\0" as *const u8 as *const libc::c_char,
                if (*v).c2rust_unnamed.max as libc::c_int != 0 {
                    b"MAX\0" as *const u8 as *const libc::c_char
                } else {
                    b"MIN\0" as *const u8 as *const libc::c_char
                },
            );
        }
        _ => panic!(),
    }
    return strbuilder_build(b);
}

pub unsafe fn number_value_copy(mut v: *mut number_value) -> *mut number_value {
    match (*v).type_0 as libc::c_uint {
        0 => number_value_constant_create((*v).c2rust_unnamed.constant),
        1 => number_value_limit_create((*v).c2rust_unnamed.max),
        _ => panic!(),
    }
}

pub unsafe fn number_values_aresingle(
    mut v1: *mut number_value,
    mut v2: *mut number_value,
) -> bool {
    if (*v1).type_0 as libc::c_uint != (*v2).type_0 as libc::c_uint {
        return false;
    }
    match (*v1).type_0 as libc::c_uint {
        0 => (*v1).c2rust_unnamed.constant == (*v2).c2rust_unnamed.constant - 1 as libc::c_int,
        1 => (*v1).c2rust_unnamed.max as libc::c_int == (*v2).c2rust_unnamed.max as libc::c_int,
        _ => panic!(),
    }
}

pub unsafe fn number_value_difference(
    mut v1: *mut number_value,
    mut v2: *mut number_value,
) -> libc::c_int {
    if !((*v1).type_0 as libc::c_uint == (*v2).type_0 as libc::c_uint) {
        panic!();
    }
    match (*v1).type_0 as libc::c_uint {
        0 => (*v1).c2rust_unnamed.constant - (*v2).c2rust_unnamed.constant,
        _ => panic!(),
    }
}

pub unsafe fn number_value_equal(mut v1: *mut number_value, mut v2: *mut number_value) -> bool {
    if (*v1).type_0 as libc::c_uint != (*v2).type_0 as libc::c_uint {
        return false;
    }
    match (*v1).type_0 as libc::c_uint {
        0 => number_value_difference(v1, v2) == 0 as libc::c_int,
        1 => (*v1).c2rust_unnamed.max as libc::c_int == (*v2).c2rust_unnamed.max as libc::c_int,
        _ => panic!(),
    }
}

pub unsafe fn number_value_as_constant(mut v: *mut number_value) -> libc::c_int {
    if !((*v).type_0 as libc::c_uint == NUMBER_VALUE_CONSTANT as libc::c_int as libc::c_uint) {
        panic!();
    }
    return (*v).c2rust_unnamed.constant;
}
unsafe fn number_value_le_constant(mut v: *mut number_value, mut constant: libc::c_int) -> bool {
    match (*v).type_0 as libc::c_uint {
        0 => (*v).c2rust_unnamed.constant <= constant,
        1 => !(*v).c2rust_unnamed.max,
        _ => panic!(),
    }
}
unsafe fn constant_le_number_value(mut constant: libc::c_int, mut v: *mut number_value) -> bool {
    match (*v).type_0 as libc::c_uint {
        0 => constant <= (*v).c2rust_unnamed.constant,
        1 => (*v).c2rust_unnamed.max,
        _ => panic!(),
    }
}
