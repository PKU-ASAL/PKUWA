use std::cell::RefCell;

pub use dlmalloc::{Domain, GlobalDlmalloc, PKEY_DISABLE_ACCESS, PKEY_DISABLE_WRITE};

#[macro_export]
macro_rules! pkucall {
    ($func:ident( $( $arg:expr ),* )) => {
        {
            let old_domain = GlobalDlmalloc::get_domain_id();
            let new_domain = find_domain($func as *const ());
            let prot = Domain::get_domain_prot(new_domain);
            Domain::switch_domain(new_domain, prot);
            let ret = $func( $( $arg ),* );
            Domain::restore_domain(new_domain, old_domain, PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE);
            ret
        }
    };
}

// #[macro_export]
// macro_rules! testcall {
//     (@_@) => {
//         {
//             Domain::switch_domain(1, 0);
//         }
//     };
//     ([#]) => {
//         {
//             let ptr = Domain::mmap(0, 4096, 3, 0x2 | 0x20);
//             Domain::domain_protect(ptr as usize, 4096);
//         }
//     };
// }

struct PKUCall {
    func: *const (),
    domain: usize,
}

thread_local!(static PKUREGISTCALL: RefCell<Vec<PKUCall>> = RefCell::new(Vec::new()));

impl PKUCall {
    pub fn new(func: *const (), domain: usize) -> Self {
        PKUCall {
            func: (func),
            domain: (domain),
        }
    }

    pub fn get_func(&self) -> *const () {
        self.func
    }

    pub fn get_domain(&self) -> usize {
        self.domain
    }
}

pub fn register_pku_call(func: *const (), domain: usize) {
    if domain != 0 {
        PKUREGISTCALL.with(|pku| {
            pku.borrow_mut().push(PKUCall::new(func, domain));
        });
    }
}

pub fn find_domain(func: *const ()) -> usize {
    PKUREGISTCALL.with(|pku| {
        let mut domain = 0;
        for iter in pku.borrow().iter() {
            if iter.get_func() as *const () == func {
                domain = iter.get_domain();
                break;
            }
        }
        return domain;
    })
}

#[cfg(target_arch = "wasm32")]
pub fn get_memory_footprint() -> usize {
    GlobalDlmalloc::get_memory_footprint()
}

#[cfg(target_arch = "wasm32")]
pub fn print_memory_footprint() {
    let ret = GlobalDlmalloc::memory_footprint_vector();
    for iter in ret {
        println!("{}", iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[global_allocator]
    static GLOBAL: GlobalDlmalloc = GlobalDlmalloc;

    fn func(s: &mut String) {
        let _ = Box::new(5);
        s.push_str("func");
    }

    fn rdpkru() -> i32 {
        let ecx = 0;
        return ecx;
    }

    #[test]
    fn it_works() {
        let domain = Domain::create_domain(0);
        register_pku_call(func as *const (), domain);
        let mut s = String::new();
        pkucall!(func(&mut s));
        println!("{s}");
        println!("rdpkru = 0x{:x}", rdpkru());
    }
}
