use pku::*;

#[global_allocator]
static GLOBAL: GlobalDlmalloc = GlobalDlmalloc;

fn func() {
    let _ = Box::new(5);
}

fn main() {
    let domain = Domain::create_domain(0);
    pku::register_pku_call(func as *const (), domain);
    pku::pkucall!(func());
    println!("rdpkru = 0x{:x}", Domain::rdpkru());
}
