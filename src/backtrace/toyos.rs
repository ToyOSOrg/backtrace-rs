use core::ffi::c_void;
use core::ptr::addr_of_mut;

pub enum Frame {
    Raw(*mut uw::_Unwind_Context),
    Cloned {
        ip: *mut c_void,
        sp: *mut c_void,
        symbol_address: *mut c_void,
    },
}

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    pub fn ip(&self) -> *mut c_void {
        let ctx = match *self {
            Frame::Raw(ctx) => ctx,
            Frame::Cloned { ip, .. } => return ip,
        };
        unsafe { uw::_Unwind_GetIP(ctx) as *mut c_void }
    }

    pub fn sp(&self) -> *mut c_void {
        match *self {
            Frame::Raw(_ctx) => core::ptr::null_mut(),
            Frame::Cloned { sp, .. } => sp,
        }
    }

    pub fn symbol_address(&self) -> *mut c_void {
        if let Frame::Cloned { symbol_address, .. } = *self {
            return symbol_address;
        }
        unsafe { uw::_Unwind_FindEnclosingFunction(self.ip()) }
    }

    pub fn module_base_address(&self) -> Option<*mut c_void> {
        None
    }
}

impl Clone for Frame {
    fn clone(&self) -> Frame {
        Frame::Cloned {
            ip: self.ip(),
            sp: self.sp(),
            symbol_address: self.symbol_address(),
        }
    }
}

struct Bomb {
    enabled: bool,
}

impl Drop for Bomb {
    fn drop(&mut self) {
        if self.enabled {
            panic!("cannot panic during the backtrace function");
        }
    }
}

#[inline(always)]
pub unsafe fn trace(mut cb: &mut dyn FnMut(&super::Frame) -> bool) {
    unsafe {
        uw::_Unwind_Backtrace(trace_fn, addr_of_mut!(cb).cast());
    }

    extern "C" fn trace_fn(
        ctx: *mut uw::_Unwind_Context,
        arg: *mut c_void,
    ) -> uw::_Unwind_Reason_Code {
        let cb = unsafe { &mut *arg.cast::<&mut dyn FnMut(&super::Frame) -> bool>() };
        let cx = super::Frame {
            inner: Frame::Raw(ctx),
        };

        let mut bomb = Bomb { enabled: true };
        let keep_going = cb(&cx);
        bomb.enabled = false;

        if keep_going {
            uw::_URC_NO_REASON
        } else {
            uw::_URC_FAILURE
        }
    }
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod uw {
    pub use self::_Unwind_Reason_Code::*;

    use core::ffi::c_void;

    #[repr(C)]
    pub enum _Unwind_Reason_Code {
        _URC_NO_REASON = 0,
        _URC_FOREIGN_EXCEPTION_CAUGHT = 1,
        _URC_FATAL_PHASE2_ERROR = 2,
        _URC_FATAL_PHASE1_ERROR = 3,
        _URC_NORMAL_STOP = 4,
        _URC_END_OF_STACK = 5,
        _URC_HANDLER_FOUND = 6,
        _URC_INSTALL_CONTEXT = 7,
        _URC_CONTINUE_UNWIND = 8,
        _URC_FAILURE = 9,
    }

    pub enum _Unwind_Context {}

    pub type _Unwind_Trace_Fn =
        extern "C" fn(ctx: *mut _Unwind_Context, arg: *mut c_void) -> _Unwind_Reason_Code;

    unsafe extern "C" {
        pub fn _Unwind_Backtrace(
            trace: _Unwind_Trace_Fn,
            trace_argument: *mut c_void,
        ) -> _Unwind_Reason_Code;
        pub fn _Unwind_GetIP(ctx: *mut _Unwind_Context) -> usize;
        pub fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void;
    }
}
