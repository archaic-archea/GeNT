# Lockdown
When the kernel encounters an error it does not know how to recover from, it enters a lockdown mode, this mode prevents execution of user programs, but on screen, or through other output devices, such as a serial device, it will output a message.  
The message should look like this: 
"Warning: a kernel panic has occurred, and cannot be recovered from. Error code: {error_code}, Kernel version: {kernel_version}, Stack trace: {stack_trace}, Debug log location: {device_id}:{path_to_log}, Press any button to complete shutdown"
The kernel will auto-shutdown after 30 seconds.
Upon shutdown the kernel should complete all standard safety procedures to avoid potential damage.