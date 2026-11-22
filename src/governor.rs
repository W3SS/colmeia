use crate::config::Config;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MemoryPressure {
    Normal,
    SoftLimit,
    HardLimit,
}

pub struct Governor {
    pub soft_mb: u64,
    pub hard_mb: u64,
}

impl Governor {
    pub fn new(cfg: &Config) -> Self {
        Self {
            soft_mb: cfg.memory_soft_mb,
            hard_mb: cfg.memory_hard_mb,
        }
    }

    /// Obtém o Resident Set Size (RSS) atual do processo em Megabytes
    pub fn current_rss_mb(&self) -> u64 {
        // No Linux, /proc/self/statm relata páginas residentes
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(resident_pages) = statm.split_whitespace().nth(1) {
                if let Ok(pages) = resident_pages.parse::<u64>() {
                    let page_size_kb = 4; // Padrão Linux x86/arm
                    return (pages * page_size_kb) / 1024;
                }
            }
        }

        #[cfg(unix)]
        {
            let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
            let res = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
            if res == 0 {
                let usage = unsafe { usage.assume_init() };
                #[cfg(target_os = "macos")]
                {
                    // No macOS ru_maxrss é em bytes
                    return (usage.ru_maxrss as u64) / (1024 * 1024);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    // No Linux ru_maxrss é em Kilobytes
                    return (usage.ru_maxrss as u64) / 1024;
                }
            }
        }

        0
    }

    pub fn check(&self) -> MemoryPressure {
        let rss = self.current_rss_mb();
        if rss >= self.hard_mb {
            MemoryPressure::HardLimit
        } else if rss >= self.soft_mb {
            MemoryPressure::SoftLimit
        } else {
            MemoryPressure::Normal
        }
    }
}
