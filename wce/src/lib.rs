use lazy_static::lazy_static;
use libc::{size_t, sysconf, SYS_pkey_alloc, SYS_pkey_free, SYS_pkey_mprotect, _SC_PAGESIZE};
use std::arch::asm;
use std::mem;
use std::os::raw::c_void;
use std::sync::Mutex;

const NUM_MPROTECT_RANGES: usize = 4096;
const NUM_DOMAINS: usize = 16;
const PAGESIZEPKU: usize = 4096;
const PKEY_DISABLE_ACCESS: u32 = 0x1;
const PKEY_DISABLE_WRITE: u32 = 0x2;
const EINVAL:u32 =  22;
const NUM_REGISTERED_PKUCALLS: usize = 64;

struct SMprotect {
    addr: *mut u8,
    len: usize,
    prot: i32,
    pkey: PKUKey,
    used: bool,
    name: String,
    mmap_flags: i32,
    mmap_fd: i32,
}

struct PKUCall<'a>{
    did: i32,
    entry: Option<&'a fn()> // ???
}
struct PKUData {
    initialized: bool,
    stacksize: usize,
    userhandler: Option<fn(*mut u8)>,
    domains: [PKUKey; NUM_DOMAINS],
    s_mprotect: [SMprotect; NUM_MPROTECT_RANGES],
    ranges_max_used: usize,
}


struct PKUKey {
    pkey: u16,
    perm: u32,
    used: bool,
}
lazy_static! {
    static ref G_LAZY_FREE: Mutex<bool> = Mutex::new(false);
    static ref G_MALLOC_NUMBER: Mutex<u32> = Mutex::new(0);
    static ref G_FREE_NUMBER: Mutex<u32> = Mutex::new(0);
    static ref G_EXTRA_MEMORY: Mutex<u32> = Mutex::new(0);
    static ref CURRENT_DID: Mutex<u32> = Mutex::new(0);
    static ref GS_MMAP_MEMORY: Mutex<usize> = Mutex::new(0);
    static ref G_INITIALIZED: Mutex<u8> = Mutex::new(0);
    static ref KEYS: Mutex<[PKUKey; NUM_DOMAINS]> = Mutex::new([PKUKey{
        pkey: 0,
        perm: 0,
        used: false
    }; NUM_DOMAINS]);
    static ref REGISTERED_PKUCALLS: Mutex<[PKUCall<'static>; NUM_REGISTERED_PKUCALLS]> = Mutex::new([PKUCall{
        did: 0,
        entry: None
    }; NUM_REGISTERED_PKUCALLS]);
    static ref G_DATA: Mutex<PKUData> = Mutex::new(PKUData {
        initialized: false,
        stacksize: 0,
        userhandler: None,
        domains: [PKUKey{
            pkey: 0,
            perm: 0,
            used: false 
        }; NUM_DOMAINS],
        s_mprotect: [SMprotect {
            addr: 0,
            len: 0,
            prot: 0,
            pkey: 0,
            used: false,
            name: None,
            mmap_flags: 0,
            mmap_fd: 0
        }; NUM_MPROTECT_RANGES],
        ranges_max_used: 16
    });
    static ref MMAP_ADDR: Mutex<*mut c_void> = 0 as Mutex<*mut c_void>;
}

pub fn get_g_lazy_free() -> &'static Mutex<bool> {
    &G_LAZY_FREE
}

pub fn get_g_initialized() -> &'static Mutex<u8> {
    &G_INITIALIZED
}

pub fn get_mmap_addr() -> &'static Mutex<*mut c_void> {
    &MMAP_ADDR
}

pub fn get_g_data() -> &'static Mutex<PKUData> {
    &G_DATA
}

pub fn get_g_malloc_number() -> &'static Mutex<u32> {
    &G_MALLOC_NUMBER
}

pub fn get_g_free_number() -> &'static Mutex<u32> {
    &G_FREE_NUMBER
}

pub fn get_g_extra_memory() -> &'static Mutex<u32> {
    &G_EXTRA_MEMORY
}

pub fn get_current_did() -> &'static Mutex<u32> {
    &CURRENT_DID
}

pub fn get_gs_mmap_memory() -> &'static Mutex<usize> {
    &GS_MMAP_MEMORY
}

//pub fn get_mmap_addr() -> &'static Mutex<*mut u8>{
//    &MMAP_ADDR
//}

pub fn get_keys() -> &'static Mutex<[PKUKey; NUM_DOMAINS]> {
    &KEYS
}

pub fn get_registered_pkucalls() -> &'static Mutex<[PKUCall<'static>; NUM_REGISTERED_PKUCALLS]> {
    &REGISTERED_PKUCALLS
}


fn domainexists(did: i32) -> bool {
    let mut a: bool = true;
    if did < 0 || did >= NUM_MPROTECT_RANGES {
        a = false;
    }
    a
}

fn doinit(flags: i32) -> i32 {
    let mut g_data = get_g_data().lock().unwrap();
    if g_data.initialized {
        println!("DoInit: PKU already initialized");
        -1
    }
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size == -1 {
        println!("DoInit: sysconf(_SC_PAGESIZE) failed");
        -1
    }
    if page_size != PAGESIZEPKU {
        println!(
            "DoInit: pagesize does not match. It should be {} but it is {}",
            PAGESIZEPKU, page_size
        );
        -1
    }
    g_data.initialized = true;
    0
}

fn pkudeinit() -> i32 {
    0
}

fn pkuinit(flags: i32) -> i32 {
    let mut ret: i32 = -1;
    let mut do_init_finished: i32 = 0;
    let mut g_initialized = get_g_initialized().lock().unwrap();
    if g_initialized != 0 {
        Ok(0)
    }
    ret = doinit(flags);
    if ret == -1 {
        ret
    }
    do_init_finished = 1;
    g_initialized = 1;
    ret
}

fn pkudomainfree(domain: i32) -> i32 {
    let mut g_data = get_g_data().lock().unwrap();
    if g_data.initialized == 0 {
        println!("PKUDomainFree: PKU not initialized");
        -1
    }
    if domainexists(domain) == false {
        println!("PKUDomainFree: domain {} does not exist", domain);
        -1
    }
    for did in 0..NUM_DOMAINS {
        if g_data.domains[did].used {
            println!(
                "PKUDomainFree: domain {} is still in use, cannot free domains",
                did,
            );
            -1
        }
    }

        for rid in 0..NUM_MPROTECT_RANGES {
            if g_data.ranges[rid].used {
                match g_data.userhandler{
                    Some(handler) => {
                        handler(g_data.ranges[rid].addr);
                    }
                    None => {
                        println!("PKUDomainFree: range {} addr {} len {} ({}) does not have a handler, cannot free pkeys", rid, g_data.ranges[rid].addr, g_data.ranges[rid].len, g_data.ranges[rid].name);
                    }
                } 
            }
        }


        let mut dom = &mut g_data.domains[domain];
        dom.used = false;
        dom.pkey = 0;
        dom.perm = 0;

    0
}

fn pkupkeyalloc(flags: u32, access_rights: u32) -> i32 {
    if access_rights & !(PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE) != 0 {
        println!("PKUPkeyAlloc: invalid access_rights");
        -1
    }
    let pka = pkucreatedomain(flags);
    pka
}

fn pkucreatedomain(flags: u32) -> i32{
    let mut buf: [u8; 12] = [0x01, 0x21];
    let error = WASICALL(buf, sizeof(buf));
    if error != 0 {
        println!("pku_create_domain failed")
        -1
    }
    let a = &buf;
    if a[2] >= 16 {
        0
    } else {
        let mut keys = get_keys().lock().unwrap();
        keys[a[2]].pkey = a[2];
        keys[a[2]].used = 1;
        buf[2]
    }

}

fn pkupkeyfree(pkey: i32) -> i32 {
    let mut ret = -1;
    let g_data = get_g_data().lock().unwrap();
    for rid in 0..NUM_DOMAINS {
        if g_data.ranges[rid].used {
            println!(
                "PKUPkeyFree: range {} addr {} len {} ({}) is still in use, cannot free pkeys",
                rid, g_data.ranges[rid].addr, g_data.ranges[rid].len, g_data.ranges[rid].name
            );
            -1
        }
    }
        for did in 0..NUM_DOMAINS {
            if g_data.domains[did].used {
                let domain = &mut g_data.domains[did];
                if domain.used {
                    domain.used = false;
                    println!("PKUPkeyFree: domain {} freed", did);
                }
            }
        }
    let g_lazy_free = get_g_lazy_free().lock().unwrap();
    if g_lazy_free != 0 {
        ret = 0;
    } else {
        ret = pkudomainfree(pkey);
    }

    ret
}

fn domain_protect(addr: u32, length: usize, pkey: u32) -> i32 {
    let mut buf: [u8; 12] = [0x01, 0x20];
    int error = WASICALL(buf, sizeof(buf));
    if error != 0 {
        println!("pku_create_domain failed")
        -1
    }
    let a = &buf;
    if a[2] >= 16 {
        0
    } else {
        keys[buf[2]].pkey = buf[2];
        keys[buf[2]].used = 1;
        a[2]
    }
}
#[inline(never)]
fn rdpkru() -> i32 {
    let ecx: i32;
    unsafe {
        asm!("mov %ecx, $0", "=r"(ecx));
    }
    ecx
}

#[inline(never)]
fn wrpkru(pkru: i32) -> i32 {
    let eax = pkru;
    unsafe {
        asm!("mov $0, %eax", "r"(eax));
    }
    eax
}
fn read_pkru() -> i32{
    let mut buf: [u8; 12] = [0x01, 0x21];
    int error = WASICALL(buf, sizeof(buf));
    if error != 0 {
        println!("pku_create_domain failed")
        -1
    }
    let mut pkru: i32 = 0;
    for i in 3..7{
        pkru = pkru << 8;
        pkru = pkru + buf[i];
    }
    pkru
}

fn write_pkru(pkru: u32)->i32{
    let mut buf: [u8; 12] = [0x0f, 0x01, 0xEF];
    let mut temp = pkru;
    for i in 3..7{
        buf[i] = temp & 0xff;
        temp = temp >> 8;
    }
    int error = WASICALL(buf, sizeof(buf));
    if error != 0 {
        println!("write_pkru failed")
        -1
    }
    0
}

pub fn setpkey(pkey: u16, prot: u32){
    let mut pkey_shift = pkey * 2;
    let mut new_pkru_bits = 0;

    if prot & PKEY_DISABLE_ACCESS{
        new_pkru_bits = new_pkru_bits | PKEY_DISABLE_ACCESS;
    }
    if prot & PKEY_DISABLE_WRITE{
        new_pkru_bits = new_pkru_bits | PKEY_DISABLE_WRITE;
    }
    new_pkru_bits = new_pkru_bits << pkey_shift;
    let mut old_pkru = rdpkru();
    
    if old_pkru == 0 {
        old_pkru = 0x55555554;
    }
    old_pkru = old_pkru & !((PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE)  << pkey_shift);
    wrpkru(old_pkru | new_pkru_bits);
    0
}


fn pku_domain_assign_key(did: i32, pkey: i32, flags: i32, access_rights: i32){
    let mut curdid = get_current_did().lock().unwrap();
    if !domainexists(curdid){
        println!("pku_domain_assign_key: get_current_did not exists");
        -EINVAL
    }
    if !domainexists(did){
        println!("pku_domain_assign_key: target domain did not exists");
        -EINVAL
    }
    if access_rights & !(PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE){
        println!("pku_domain_assign_key: invalid access_rights");
        -EINVAL
    }
    setpkey(KEYS[did].pkey, KEYS[did].perm);
    0
}

fn pku_pkey_mprotect(addr: Option<u32>, len: size_t, prot: i32, pkey: i32){
    let mut ret = pku_mprotect(addr, len, prot);
    ret
}

fn pku_mprotect(addr: *mut c_void, len: usize, prot:i32)-> i32{
    unsafe{
        let mmap_addr = get_mmap_addr().lock().unwrap();
        if addr.is_null() && mmap_addr.is_null(){ 
            mmap_addr= pku_mmap(addr, len, prot, 0x2 | 0x20, -1, 0);
        
        }
        if !mmap_addr.is_null(){
            domain_protect(mmap_addr, len, 0);
        }
    }
    0
}

fn pku_mmap(addr: *mut c_void, length: u64, prot: i32, flags: i32, fd: i32, offset: i32) -> *mut u8{
    let mut buf: [u8; 12] = [0x01, 0x2B];
    unsafe{
    let mut temp: u8= *addr;
    for i in 3..0{
        buf[i + 2] = temp & 0xff;
        temp = temp >> 8;
    }
    temp = length;
    for i in 3..0{
        buf[i + 6] = temp & 0xff;
        temp = temp >> 8;
    }
    buf[10] = prot;
    buf[11] = flags;

    let mut error = WASICALL(buf, sizeof(buf));
    if error != 0 {
        println!("pku_mmap failed")
    }

    temp = 0;

    for i in 2..6{
        temp = temp << 8;
        temp = temp + buf[i];
    }
    let mut len: u64 = 0;
    for i in 6..10 {
        len = len << 8;
        len = len + buf[i];
    }
    if len != length{
        temp = 0;
        println!("pku_mmap: length does not match");
    }
    let mut gs_mmap_memory = get_gs_mmap_memory().lock().unwrap();
    gs_mmap_memory += len;
    temp as *mut u8
    }
}

fn set_current_did(did: i32){
    let mut curdid = get_current_did().lock().unwrap();
    curdid = did;
    0
}

fn pkuswitch(pku_callid: i32){
    let mut registered_pkucalls = get_registered_pkucalls().lock().unwrap();
    let did = registered_pkucalls[pku_callid].did;
    let mut keys = get_keys().lock().unwrap();
    setpkey(keys[did].pkey, keys[did].perm);
    set_current_did(did);
    0
}

fn pku_restore(did: i32){
    let mut keys = get_keys().lock().unwrap();
    setpkey(keys[get_current_did()].pkey, PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE);
    set_current_did(did);
    0
}

//fn get_memory_size() -> u64{
//    let ret = GS_MmapMemory + g_ExtraMemory + MemorySize();
//}

// allocate a chunck of memory in wasm linear memory and return the pointer
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

