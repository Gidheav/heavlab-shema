//! bible-rt — Runtime Tuning & OS-level Performance
//!
//! Memory pinning, thread priorities, and process priority. Platform-specific
//! internals are `unsafe`; the public API is safe and never panics.

use thiserror::Error;

/// Failures from OS-level runtime tuning. Never panics; callers decide policy.
#[derive(Debug, Error)]
pub enum RtError {
    #[error("failed to pin process memory: {0}")]
    MemoryPin(String),
    #[error("failed to set thread priority: {0}")]
    ThreadPriority(String),
    #[error("failed to set process priority: {0}")]
    ProcessPriority(String),
}

/// Raise the process working-set floor so hot ASR/audio pages stay resident.
///
/// On POSIX this calls `mlockall(MCL_CURRENT | MCL_FUTURE)`.
/// On Windows this calls `SetProcessWorkingSetSizeEx` (successor to `SetProcessWorkingSetSize`).
pub fn pin_memory() -> Result<(), RtError> {
    #[cfg(unix)]
    {
        let ret = unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) };
        if ret != 0 {
            return Err(RtError::MemoryPin(os_error_message()));
        }
        return Ok(());
    }

    #[cfg(windows)]
    {
        return windows::pin_working_set();
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

/// Set the calling thread to real-time / time-critical audio priority.
///
/// Must be invoked from the audio capture thread itself.
/// On Windows this uses `SetThreadPriority(THREAD_PRIORITY_TIME_CRITICAL)`.
pub fn set_audio_thread_priority() -> Result<(), RtError> {
    #[cfg(unix)]
    {
        return unix::set_fifo_priority();
    }

    #[cfg(windows)]
    {
        return windows::set_audio_thread_priority();
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

/// Boost the process scheduling class so audio and ASR are less likely to be preempted.
///
/// On Windows this uses `SetPriorityClass(HIGH_PRIORITY_CLASS)`.
pub fn set_process_priority() -> Result<(), RtError> {
    #[cfg(unix)]
    {
        return unix::set_process_nice();
    }

    #[cfg(windows)]
    {
        return windows::set_process_priority();
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

/// Apply process-wide tuning (priority + memory pin). Call once at startup.
///
/// Thread priority is **not** set here; call [`set_audio_thread_priority`] from
/// the audio thread.
pub fn apply_process_tuning() -> Result<(), RtError> {
    set_process_priority()?;
    pin_memory()?;
    Ok(())
}

fn os_error_message() -> String {
    std::io::Error::last_os_error().to_string()
}

#[cfg(unix)]
mod unix {
    use super::{os_error_message, RtError};

    pub(super) fn set_fifo_priority() -> Result<(), RtError> {
        unsafe {
            let mut param: libc::sched_param = std::mem::zeroed();
            param.sched_priority = libc::sched_get_priority_max(libc::SCHED_FIFO);
            let ret = libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
            if ret != 0 {
                return Err(RtError::ThreadPriority(os_error_message()));
            }
        }
        Ok(())
    }

    pub(super) fn set_process_nice() -> Result<(), RtError> {
        // Negative nice requires CAP_SYS_NICE; surface the OS error if it fails.
        let ret = unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, -10) };
        if ret != 0 {
            return Err(RtError::ProcessPriority(os_error_message()));
        }
        Ok(())
    }
}

#[cfg(windows)]
mod windows {
    use super::{os_error_message, RtError};
    use windows_sys::Win32::System::Memory::{
        SetProcessWorkingSetSizeEx, QUOTA_LIMITS_HARDWS_MIN_ENABLE,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentThread, SetPriorityClass, SetThreadPriority,
        HIGH_PRIORITY_CLASS, THREAD_PRIORITY_TIME_CRITICAL,
    };

    /// Keep a large minimum working set so the OS is less likely to page out
    /// the ASR model and audio buffers during a live service.
    const MIN_WORKING_SET_BYTES: usize = 512 * 1024 * 1024;
    const MAX_WORKING_SET_BYTES: usize = 2usize * 1024 * 1024 * 1024;

    pub(super) fn pin_working_set() -> Result<(), RtError> {
        let handle = unsafe { GetCurrentProcess() };
        // windows-sys 0.59 exposes SetProcessWorkingSetSizeEx (the current Win32 API).
        // HARDWS_MIN_ENABLE keeps the minimum working set as a floor so pages stay resident.
        let ret = unsafe {
            SetProcessWorkingSetSizeEx(
                handle,
                MIN_WORKING_SET_BYTES,
                MAX_WORKING_SET_BYTES,
                QUOTA_LIMITS_HARDWS_MIN_ENABLE,
            )
        };
        if ret == 0 {
            return Err(RtError::MemoryPin(os_error_message()));
        }
        Ok(())
    }

    pub(super) fn set_audio_thread_priority() -> Result<(), RtError> {
        let handle = unsafe { GetCurrentThread() };
        let ret = unsafe { SetThreadPriority(handle, THREAD_PRIORITY_TIME_CRITICAL) };
        if ret == 0 {
            return Err(RtError::ThreadPriority(os_error_message()));
        }
        Ok(())
    }

    pub(super) fn set_process_priority() -> Result<(), RtError> {
        let handle = unsafe { GetCurrentProcess() };
        let ret = unsafe { SetPriorityClass(handle, HIGH_PRIORITY_CLASS) };
        if ret == 0 {
            return Err(RtError::ProcessPriority(os_error_message()));
        }
        Ok(())
    }
}
