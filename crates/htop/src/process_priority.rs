use std::io;

pub(crate) fn nice_value(pid: &str) -> Option<i32> {
    platform::nice_value(pid.parse().ok()?)
}

pub(crate) fn adjust(pid: &str, delta: i32) -> io::Result<i32> {
    let pid = pid
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid process id"))?;
    platform::adjust(pid, delta)
}

#[cfg(unix)]
mod platform {
    use std::io;

    pub(super) fn nice_value(pid: u32) -> Option<i32> {
        Some(unsafe { libc::getpriority(libc::PRIO_PROCESS, pid as libc::id_t) })
    }

    pub(super) fn adjust(pid: u32, delta: i32) -> io::Result<i32> {
        let current = nice_value(pid)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "process not found"))?;
        let target = current.saturating_add(delta).clamp(-20, 19);
        let result = unsafe {
            libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, target)
        };
        if result == 0 {
            Ok(target)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(windows)]
mod platform {
    use std::io;

    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Threading::{
            ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, GetPriorityClass,
            HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, OpenProcess,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, REALTIME_PRIORITY_CLASS,
            SetPriorityClass,
        },
    };

    const CLASSES: [(u32, i32); 5] = [
        (HIGH_PRIORITY_CLASS, -10),
        (ABOVE_NORMAL_PRIORITY_CLASS, -5),
        (NORMAL_PRIORITY_CLASS, 0),
        (BELOW_NORMAL_PRIORITY_CLASS, 10),
        (IDLE_PRIORITY_CLASS, 19),
    ];

    pub(super) fn nice_value(pid: u32) -> Option<i32> {
        let handle = ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
        let class = unsafe { GetPriorityClass(handle.0) };
        class_nice(class)
    }

    pub(super) fn adjust(pid: u32, delta: i32) -> io::Result<i32> {
        let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION;
        let handle = ProcessHandle::open(pid, access)
            .ok_or_else(io::Error::last_os_error)?;
        let current = unsafe { GetPriorityClass(handle.0) };
        if current == 0 {
            return Err(io::Error::last_os_error());
        }
        let current_index = CLASSES
            .iter()
            .position(|(class, _)| *class == current)
            .unwrap_or(2);
        let target_index = if delta < 0 {
            current_index.saturating_sub(1)
        } else {
            (current_index + 1).min(CLASSES.len() - 1)
        };
        let (class, nice) = CLASSES[target_index];
        if unsafe { SetPriorityClass(handle.0, class) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(nice)
        }
    }

    fn class_nice(class: u32) -> Option<i32> {
        if class == REALTIME_PRIORITY_CLASS {
            return Some(-20);
        }
        CLASSES
            .iter()
            .find(|(candidate, _)| *candidate == class)
            .map(|(_, nice)| *nice)
    }

    struct ProcessHandle(HANDLE);

    impl ProcessHandle {
        fn open(pid: u32, access: u32) -> Option<Self> {
            let handle = unsafe { OpenProcess(access, 0, pid) };
            if handle.is_null() {
                None
            } else {
                Some(Self(handle))
            }
        }
    }

    impl Drop for ProcessHandle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    use std::io;

    pub(super) fn nice_value(_pid: u32) -> Option<i32> {
        None
    }

    pub(super) fn adjust(_pid: u32, _delta: i32) -> io::Result<i32> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "process priority is unsupported on this platform",
        ))
    }
}