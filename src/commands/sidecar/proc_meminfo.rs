use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

lazy_static! {
    static ref FIELD_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("MemTotal:", "procmeminfo_mem_total_bytes");
        map.insert("MemFree:", "procmeminfo_mem_free_bytes");
        map.insert("MemAvailable:", "procmeminfo_mem_available_bytes");
        map.insert("Buffers:", "procmeminfo_buffers_bytes");
        map.insert("Cached:", "procmeminfo_cached_bytes");
        map.insert("SwapCached:", "procmeminfo_swap_cached_bytes");
        map.insert("Active:", "procmeminfo_active_bytes");
        map.insert("Inactive:", "procmeminfo_inactive_bytes");
        map.insert("Active(anon):", "procmeminfo_active_anon_bytes");
        map.insert("Inactive(anon):", "procmeminfo_inactive_anon_bytes");
        map.insert("Active(file):", "procmeminfo_active_file_bytes");
        map.insert("Inactive(file):", "procmeminfo_inactive_file_bytes");
        map.insert("Unevictable:", "procmeminfo_unevictable_bytes");
        map.insert("Mlocked:", "procmeminfo_mlocked_bytes");
        map.insert("SwapTotal:", "procmeminfo_swap_total_bytes");
        map.insert("SwapFree:", "procmeminfo_swap_free_bytes");
        map.insert("Dirty:", "procmeminfo_swap_dirty_bytes");
        map.insert("Writeback:", "procmeminfo_writeback_bytes");
        map.insert("AnonPages:", "procmeminfo_anon_pages_bytes");
        map.insert("Mapped:", "procmeminfo_mapped_bytes");
        map.insert("Shmem:", "procmeminfo_shmem_bytes");
        map.insert("KReclaimable:", "procmeminfo_kreclaimable_bytes");
        map.insert("Slab:", "procmeminfo_slab_bytes");
        map.insert("SReclaimable:", "procmeminfo_sreclaiamble_bytes");
        map.insert("SUnreclaim:", "procmeminfo_sunreclaim_bytes");
        map.insert("KernelStack:", "procmeminfo_kernel_stack_bytes");
        map.insert("PageTables:", "procmeminfo_page_tables_bytes");
        map.insert("NFS_Unstable:", "procmeminfo_nfs_unstable_bytes");
        map.insert("Bounce:", "procmeminfo_bounce_bytes");
        map.insert("WritebackTmp:", "procmeminfo_writeback_tmp_bytes");
        map.insert("CommitLimit:", "procmeminfo_commit_limit_bytes");
        map.insert("Committed_AS:", "procmeminfo_commited_as_bytes");
        map.insert("VmallocTotal:", "procmeminfo_vmalloc_total_bytes");
        map.insert("VmallocUsed:", "procmeminfo_vmalloc_used_bytes");
        map.insert("VmallocChunk:", "procmeminfo_vmalloc_chunk_bytes");
        map.insert("Percpu:", "procmeminfo_percpu_bytes");
        map.insert("HardwareCorrupted:", "procmeminfo_hardware_corrupted_bytes");
        map.insert("AnonHugePages:", "procmeminfo_anon_huge_pages_bytes");
        map.insert("ShmemHugePages:", "procmeminfo_shmem_huge_pages_bytes");
        map.insert("ShmemPmdMapped:", "procmeminfo_shmem_pmd_mapped_bytes");
        map.insert("FileHugePages:", "procmeminfo_file_huge_pages_bytes");
        map.insert("FilePmdMapped:", "procmeminfo_file_pmd_mapped_bytes");
        map.insert("CmaTotal:", "procmeminfo_cma_total_bytes");
        map.insert("CmaFree:", "procmeminfo_cma_free_bytes");
        map.insert("HugePages_Total:", "procmeminfo_huge_pages_total_bytes");
        map.insert("HugePages_Free:", "procmeminfo_huge_pages_free_bytes");
        map.insert("HugePages_Rsvd:", "procmeminfo_huge_pages_rsvd_bytes");
        map.insert("HugePages_Surp:", "procmeminfo_huge_pages_surp_bytes");
        map.insert("Hugetlb:", "procmeminfo_hugetlb_bytes");
        map.insert("DirectMap4k:", "procmeminfo_directmap4k_bytes");
        map.insert("DirectMap2M:", "procmeminfo_directmap2m_bytes");
        map.insert("DirectMap1G:", "procmeminfo_directmap1g_bytes");
        map
    };
}

pub fn get_proc_memifo() -> String {
    let file = File::open("/proc/meminfo").expect("cannot open /proc/meminfo");
    let lines = io::BufReader::new(file).lines();

    let mut result = String::new();

    for line in lines.map_while(Result::ok) {
        let mut iter = line.split_ascii_whitespace();
        let first = iter.next().expect("first value expected");
        let second = iter.next().expect("second value expected");
        if let Some(line_ok) = iter.next() {
            assert!(line_ok == "kB")
        }

        if FIELD_MAP.contains_key(first) {
            let kbytes: u64 = second.parse().unwrap();
            result.push_str(&format!("{} {}\n", FIELD_MAP[first], kbytes * 1024));
        }
    }
    result
}
