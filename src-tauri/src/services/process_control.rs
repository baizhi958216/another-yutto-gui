/// Cross-platform process control for pause/resume functionality
pub struct ProcessController;

impl ProcessController {
    #[cfg(windows)]
    fn collect_process_tree_pids(root_pid: u32) -> Result<std::collections::HashSet<u32>, String> {
        use std::collections::{HashMap, HashSet};
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        };

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create process snapshot".to_string());
            }

            let mut entry = PROCESSENTRY32 {
                dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
                ..std::mem::zeroed()
            };

            let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
            if Process32First(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    let parent_pid = entry.th32ParentProcessID;
                    children.entry(parent_pid).or_default().push(pid);

                    if Process32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);

            let mut result = HashSet::new();
            let mut stack = vec![root_pid];
            while let Some(pid) = stack.pop() {
                if result.insert(pid) {
                    if let Some(kids) = children.get(&pid) {
                        for &kid in kids {
                            stack.push(kid);
                        }
                    }
                }
            }

            Ok(result)
        }
    }

    #[cfg(windows)]
    fn suspend_threads_for_pids(pids: &std::collections::HashSet<u32>) -> Result<usize, String> {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD,
            THREADENTRY32,
        };
        use windows_sys::Win32::System::Threading::{OpenThread, SuspendThread, THREAD_SUSPEND_RESUME};

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create thread snapshot".to_string());
            }

            let mut thread_entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..std::mem::zeroed()
            };

            let mut suspended = 0usize;
            if Thread32First(snapshot, &mut thread_entry) != 0 {
                loop {
                    if pids.contains(&thread_entry.th32OwnerProcessID) {
                        let thread_handle = OpenThread(
                            THREAD_SUSPEND_RESUME,
                            0,
                            thread_entry.th32ThreadID,
                        );
                        if !thread_handle.is_null() {
                            let result = SuspendThread(thread_handle);
                            CloseHandle(thread_handle);
                            if result != u32::MAX {
                                suspended += 1;
                            }
                        }
                    }

                    if Thread32Next(snapshot, &mut thread_entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            Ok(suspended)
        }
    }

    #[cfg(windows)]
    fn resume_threads_for_pids(pids: &std::collections::HashSet<u32>) -> Result<usize, String> {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD,
            THREADENTRY32,
        };
        use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create thread snapshot".to_string());
            }

            let mut thread_entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..std::mem::zeroed()
            };

            let mut resumed = 0usize;
            if Thread32First(snapshot, &mut thread_entry) != 0 {
                loop {
                    if pids.contains(&thread_entry.th32OwnerProcessID) {
                        let thread_handle = OpenThread(
                            THREAD_SUSPEND_RESUME,
                            0,
                            thread_entry.th32ThreadID,
                        );
                        if !thread_handle.is_null() {
                            let result = ResumeThread(thread_handle);
                            CloseHandle(thread_handle);
                            if result != u32::MAX {
                                resumed += 1;
                            }
                        }
                    }

                    if Thread32Next(snapshot, &mut thread_entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            Ok(resumed)
        }
    }

    /// Pause a process by sending SIGSTOP signal (Unix) or suspending all threads (Windows)
    #[cfg(unix)]
    pub fn pause_process(pid: u32) -> Result<(), String> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGSTOP)
            .map_err(|e| format!("Failed to pause process: {}", e))
    }

    /// Resume a process by sending SIGCONT signal (Unix) or resuming all threads (Windows)
    #[cfg(unix)]
    pub fn resume_process(pid: u32) -> Result<(), String> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGCONT)
            .map_err(|e| format!("Failed to resume process: {}", e))
    }

    /// Pause a process by suspending all its threads (Windows)
    #[cfg(windows)]
    pub fn pause_process(pid: u32) -> Result<(), String> {
        let pids = Self::collect_process_tree_pids(pid)?;
        let suspended = Self::suspend_threads_for_pids(&pids)?;
        if suspended == 0 {
            Err("Failed to suspend process threads".to_string())
        } else {
            Ok(())
        }
    }

    /// Resume a process by resuming all its threads (Windows)
    #[cfg(windows)]
    pub fn resume_process(pid: u32) -> Result<(), String> {
        let pids = Self::collect_process_tree_pids(pid)?;
        let resumed = Self::resume_threads_for_pids(&pids)?;
        if resumed == 0 {
            Err("Failed to resume process threads".to_string())
        } else {
            Ok(())
        }
    }

    /// Check if a process exists and is running
    #[cfg(unix)]
    pub fn is_process_alive(pid: u32) -> bool {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        // Signal 0 (None) doesn't actually send a signal, just checks if process exists
        kill(Pid::from_raw(pid as i32), None).is_ok()
    }

    /// Check if a process exists and is running (Windows)
    #[cfg(windows)]
    pub fn is_process_alive(pid: u32) -> bool {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
        use windows_sys::Win32::Foundation::CloseHandle;

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if !handle.is_null() {
                CloseHandle(handle);
                true
            } else {
                false
            }
        }
    }

    /// Kill a process (used for cancel operation)
    #[cfg(unix)]
    pub fn kill_process(pid: u32) -> Result<(), String> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
            .map_err(|e| format!("Failed to kill process: {}", e))
    }

    /// Kill a process (used for cancel operation) (Windows)
    #[cfg(windows)]
    pub fn kill_process(pid: u32) -> Result<(), String> {
        use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        use windows_sys::Win32::Foundation::CloseHandle;

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle.is_null() {
                return Err("Failed to open process for termination".to_string());
            }

            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result == 0 {
                Err("Failed to terminate process".to_string())
            } else {
                Ok(())
            }
        }
    }
}
