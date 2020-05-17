//! Forward output debug printouts from the validation layer to stdout.
//!
//! WARNING: Logger needs to be removed before terminating the application.

use winapi::um::errhandlingapi::{AddVectoredExceptionHandler, RemoveVectoredExceptionHandler};
use winapi::um::winnt::{DBG_PRINTEXCEPTION_C, EXCEPTION_POINTERS, LONG};
use winapi::vc::excpt::{EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH};

extern "system" fn vectored_handler(exception: *mut EXCEPTION_POINTERS) -> LONG {
    unsafe {
        let rec = &(*(*exception).ExceptionRecord);
        let code = rec.ExceptionCode;

        match code {
            DBG_PRINTEXCEPTION_C => {
                let len = rec.ExceptionInformation[0];
                let data = rec.ExceptionInformation[1] as *const u8;

                if let Ok(string) =
                    std::ffi::CStr::from_bytes_with_nul(std::slice::from_raw_parts(data, len))
                {
                    println!("{}", string.to_string_lossy());
                }

                EXCEPTION_CONTINUE_EXECUTION
            }
            _ => EXCEPTION_CONTINUE_SEARCH,
        }
    }
}

pub fn debug_logger_add() -> *mut std::ffi::c_void {
    unsafe { AddVectoredExceptionHandler(0, Some(vectored_handler)) }
}

pub fn debug_logger_remove(handle: *mut std::ffi::c_void) {
    if !handle.is_null() {
        unsafe {
            RemoveVectoredExceptionHandler(handle);
        }
    }
}
