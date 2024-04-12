#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use libc::{free, malloc};

use crate::state::block::{
    block_arr_append, block_arr_blocks, block_arr_copy, block_arr_create, block_arr_destroy,
    block_arr_nblocks, block_create, block_str,
};
use crate::state::location::location_copy;
use crate::util::{dynamic_str, map, strbuilder_build, strbuilder_create, strbuilder_printf};
use crate::{block_arr, Block as block, Location as location, StrBuilder as strbuilder};

pub struct static_memory {
    pub blocks: *mut block_arr,
    pub pool: Box<map>,
}

pub unsafe fn static_memory_create() -> *mut static_memory {
    let mut sm: *mut static_memory =
        malloc(::core::mem::size_of::<static_memory>()) as *mut static_memory;
    assert!(!sm.is_null());
    std::ptr::write(
        sm,
        static_memory {
            blocks: block_arr_create(),
            pool: map::new(),
        },
    );
    sm
}

pub unsafe fn static_memory_destroy(mut sm: *mut static_memory) {
    block_arr_destroy((*sm).blocks);
}

pub unsafe fn static_memory_str(
    mut sm: *mut static_memory,
    mut indent: *mut libc::c_char,
) -> *mut libc::c_char {
    let mut b: *mut strbuilder = strbuilder_create();
    let mut n: libc::c_int = block_arr_nblocks((*sm).blocks);
    let mut arr: *mut *mut block = block_arr_blocks((*sm).blocks);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut block: *mut libc::c_char = block_str(*arr.offset(i as isize));
        strbuilder_printf(
            b,
            b"%s%d: %s\n\0" as *const u8 as *const libc::c_char,
            indent,
            i,
            block,
        );
        free(block as *mut libc::c_void);
        i += 1;
    }
    return strbuilder_build(b);
}

pub unsafe fn static_memory_copy(mut sm: *mut static_memory) -> *mut static_memory {
    let mut copy: *mut static_memory =
        malloc(::core::mem::size_of::<static_memory>()) as *mut static_memory;
    (*copy).blocks = block_arr_copy((*sm).blocks);
    (*copy).pool = pool_copy(&(*sm).pool);
    return copy;
}
unsafe fn pool_copy(mut p: &map) -> Box<map> {
    let mut pcopy = map::new();
    for (k, v) in p.pairs() {
        pcopy.set(
            dynamic_str(k),
            location_copy(v as *mut location) as *const libc::c_void,
        );
    }
    return pcopy;
}

pub unsafe fn static_memory_newblock(mut sm: *mut static_memory) -> libc::c_int {
    let mut address: libc::c_int = block_arr_append((*sm).blocks, block_create());
    let mut n: libc::c_int = block_arr_nblocks((*sm).blocks);
    assert!(n > 0);
    return address;
}

pub unsafe fn static_memory_getblock(
    mut sm: *mut static_memory,
    mut address: libc::c_int,
) -> *mut block {
    if address >= block_arr_nblocks((*sm).blocks) {
        return 0 as *mut block;
    }
    return *(block_arr_blocks((*sm).blocks)).offset(address as isize);
}

pub unsafe fn static_memory_stringpool(
    mut sm: *mut static_memory,
    mut lit: *mut libc::c_char,
    mut loc: *mut location,
) {
    (*sm)
        .pool
        .set(dynamic_str(lit), location_copy(loc) as *const libc::c_void);
}

pub unsafe fn static_memory_checkpool(
    mut sm: *mut static_memory,
    mut lit: *mut libc::c_char,
) -> *mut location {
    return (*sm).pool.get(lit) as *mut location;
}
