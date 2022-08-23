use std::{thread::sleep, time::Duration};

use log::debug;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

const KB_TO_MB: u64 = 1000;
const BIT_TO_GB: u64 = 1000000000;

pub struct NodeSystem {
    sys: System,
}

impl NodeSystem {
    // used in all functions to access system
    pub fn new() -> Self {
        let mut sys = System::new_all();

        NodeSystem { sys }
    }

    fn refresh_system(&mut self) {
        self.sys.refresh_all();
    }

    // ------ CPU ------

    /*
      Returns the CPU limit (in MilliCPU)
    */
    pub fn total_cpu(&mut self) -> u64 {
        self.refresh_system();

        let nb_of_core: u64 = self.sys.cpus().len() as u64;

        let amount_of_milli_cpu: u64 = nb_of_core * 1000;
        debug!("total cpu: {:?} MilliCPU", &amount_of_milli_cpu);

        amount_of_milli_cpu
    }

    /*
      Returns the CPU usage (in MilliCPU)
    */
    pub fn used_cpu(&mut self) -> u64 {
        // we need to refresh the cpus at least 2 times (with 200 miliseconds interval) to get an accurate cpu usage
        for _ in 0..3 {
            self.sys.refresh_cpu();
            sleep(Duration::from_millis(200));
        }

        let amount_of_milli_cpu: u64 = Self::total_cpu(self);
        let cpu_usage_in_pourcent: u64 = self.sys.global_cpu_info().cpu_usage().ceil() as u64;

        let cpu_usage: u64 = amount_of_milli_cpu * cpu_usage_in_pourcent / 100;
        debug!("used cpu: {:?} MilliCPU", &cpu_usage);

        cpu_usage
    }

    // ------ MEMORY ------

    /*
      Returns the limit memory space (in MB)
    */
    pub fn total_memory(&mut self) -> u64 {
        self.refresh_system();

        let total_memory: u64 = (self.sys.available_memory() + self.sys.used_memory()) / KB_TO_MB;
        debug!("total memory: {:?} MB", &total_memory);

        total_memory
    }

    /*
      Returns the used memory space (in MB)
    */
    pub fn used_memory(&mut self) -> u64 {
        self.refresh_system();

        let used_memory: u64 = self.sys.used_memory() / KB_TO_MB;
        debug!("used memory: {} MB", &used_memory);

        used_memory
    }

    // ------ DISK ------

    /*
      Returns the limit space of the main disk (in GB)
    */
    pub fn total_disk(&mut self) -> u64 {
        self.refresh_system();

        let mut main_disk_total_space: u64 = 0;

        for disk in self.sys.disks() {
            let current_total_space: u64 = disk.total_space();

            if current_total_space > main_disk_total_space {
                main_disk_total_space = current_total_space;
            }
        }

        main_disk_total_space = main_disk_total_space / BIT_TO_GB;

        debug!("total disk space: {} GB", main_disk_total_space);

        main_disk_total_space
    }

    /*
      Returns the used space of the main disk (in GB)
    */
    pub fn used_disk(&mut self) -> u64 {
        self.refresh_system();

        // get used disk from main disk
        let mut main_disk_availability: u64 = 0;
        let mut main_disk_total_space: u64 = 0;

        for disk in self.sys.disks() {
            let current_available_space: u64 = disk.available_space();
            let current_total_space: u64 = disk.total_space();

            if current_available_space > main_disk_availability {
                main_disk_availability = current_available_space;
            }

            if current_total_space > main_disk_total_space {
                main_disk_total_space = current_total_space;
            }
        }

        let used_disk: u64 = (main_disk_total_space - main_disk_availability) / BIT_TO_GB;
        debug!("used disk: {} GB", used_disk);

        used_disk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_system() -> NodeSystem {
        NodeSystem::new()
    }

    #[test]
    fn test_total_cpu() {
        let mut sys = init_system();

        assert!(sys.total_cpu() >= 1000) // minimum 1 core
    }

    #[test]
    fn test_used_cpu() {
        let mut sys = init_system();

        let total_cpu = sys.total_cpu();
        let used_cpu = sys.used_cpu();

        assert!(used_cpu <= total_cpu)
    }

    #[test]
    fn test_used_memory() {
        let mut sys = init_system();

        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();

        assert!(used_memory <= total_memory)
    }

    #[test]
    fn test_used_disk() {
        let mut sys = init_system();

        let total_disk = sys.total_disk();
        let used_disk = sys.used_disk();

        assert!(used_disk <= total_disk)
    }
}
