/// Import the necessary modules from rust standard library and cargo dependencies
use winapi::um::psapi::{EnumProcessModules, GetModuleFileNameExW, EnumDeviceDrivers, GetDeviceDriverFileNameW};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::shared::minwindef::{HMODULE, DWORD, LPVOID};

use std::{env, process, ptr, mem};
use std::path::Path;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

/// List DLL and EXE modules of a process using `EnumProcessModules`
fn list_modules(process_id: u32) {
    unsafe {
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);
        if !process_handle.is_null() {
            let mut modules: [HMODULE; 1024] = [ptr::null_mut(); 1024];
            let mut cb_needed: DWORD = 0;

            // Enumerate modules of the process
            if EnumProcessModules(process_handle, modules.as_mut_ptr(), mem::size_of_val(&modules) as u32, &mut cb_needed) != 0 {
                let mut dll_modules = Vec::new();
                let mut exe_modules = Vec::new();

                // Get the file name of each module                
                for &module in &modules {
                    if !module.is_null() {
                        let mut filename = vec![0u16; 512];
                        let len = GetModuleFileNameExW(process_handle, module, filename.as_mut_ptr(), filename.len() as u32);
                        if len > 0 {
                            let valid_filename: Vec<u16> = filename.into_iter().take_while(|&c| c != 0).collect();
                            let filename_str = OsString::from_wide(&valid_filename).to_string_lossy().into_owned();

                            // Check the extension of the module file
                            let path = Path::new(&filename_str);
                            if let Some(extension) = path.extension() {
                                match extension.to_str() {
                                    Some("dll") => dll_modules.push(filename_str),
                                    Some("exe") => exe_modules.push(filename_str),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                // Print the DLL modules 
                println!("\nDLL Modules: ");
                for dll in dll_modules {
                    println!("{}", dll);
                }

                // Print the EXE modules
                println!("\nEXE Modules :");
                for exe in exe_modules {
                    println!("{}", exe);
                }
            }

            // Close the process handle
            CloseHandle(process_handle);
        } else {
            eprintln!("Failed to open process with PID {}", process_id);
        }
    }
}

/// List system drivers using `EnumDeviceDrivers`
fn list_sys_drivers() {
    unsafe {
        let mut drivers: [LPVOID; 1024] = [ptr::null_mut(); 1024];
        let mut cb_needed: DWORD = 0;

        // Enumerate system drivers
        if EnumDeviceDrivers(drivers.as_mut_ptr(), mem::size_of_val(&drivers) as u32, &mut cb_needed) != 0 {
            let driver_count = cb_needed as usize / mem::size_of::<LPVOID>();
            let mut sys_modules = Vec::new();

            // Get the file name of each driver
            for &driver in &drivers[..driver_count] {
                if !driver.is_null() {
                    let mut filename = vec![0u16; 512];
                    let len = GetDeviceDriverFileNameW(driver, filename.as_mut_ptr(), filename.len() as u32);
                    if len > 0 {
                        let valid_filename: Vec<u16> = filename.into_iter().take_while(|&c| c != 0).collect();
                        let filename_str = OsString::from_wide(&valid_filename).to_string_lossy().into_owned();
                        sys_modules.push(filename_str);
                    }
                }
            }

            // Print the SYS modules
            println!("\nSYS Drivers:");
            for sys in sys_modules {
                println!("{}", sys);
            }
        } else {
            eprintln!("Failed to enumerate system drivers.");
        }
    }
}


/// Main function to parse command line arguments and call the functions to list modules and drivers
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a process ID (PID) as an argument.");
        process::exit(1);
    }

    // Parse the PID from the command line arguments
    let pid: u32 = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid PID provided.");
            process::exit(1);
        }
    };

    // Call the functions to list modules and drivers
    list_modules(pid);
    list_sys_drivers();
}