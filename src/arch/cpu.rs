use alloc::string::String;
use raw_cpuid::CpuId;


#[repr(C)]
pub struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
}

fn cpuid() -> CpuId {
    CpuId::new()
}

pub fn vendor_info() -> Option<String> {
    if let Some(vf) = cpuid().get_vendor_info() {
        return Some(vf.as_str().into());
    } else {
        None
    }
}

pub fn supports_sse() -> bool {
    cpuid()
        .get_feature_info()
        .map_or(false, |finfo| finfo.has_sse())
}

pub fn supports_sse2() -> bool {
    cpuid()
        .get_feature_info()
        .map_or(false, |finfo| finfo.has_sse2())
}

pub fn supports_avx() -> bool {
    cpuid()
        .get_feature_info()
        .map_or(false, |finfo| finfo.has_avx())
}

pub fn cache_params() -> Option<raw_cpuid::CacheParametersIter> {
    cpuid().get_cache_parameters()
}

pub fn l1_cache_info() -> Option<raw_cpuid::CacheParameter> {
    cache_params()
    .and_then(|mut params| {params.nth(0)})
}
