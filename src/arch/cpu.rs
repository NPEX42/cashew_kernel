use x86_64::VirtAddr;


/// Execute A Byte Slice As x86-64 Machine Code.
/// Safety:
///     - binary MUST be legal & valid x86/x64 Machine Code.
///     - The System MUST be left in a valid & consistent State
///     - The Calling Convention MUST be the C Standard Call.
pub unsafe fn execute_c(addr: VirtAddr) {
    let fn_ptr = (addr.as_ptr() as *const ()) as *const extern "C" fn();

    (*fn_ptr)()
}