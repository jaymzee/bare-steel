#![no_std]
#![no_main]

use blog_os::{
    print_test_name, print_test_passed, print_test_failed_because,
    exit_qemu, QemuExitCode,
};
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    should_fail();
    print_test_failed_because("test did not panic");
    exit_qemu(QemuExitCode::Failed);
}

fn should_fail() {
    print_test_name("should_panic::should_fail");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_test_passed();
    exit_qemu(QemuExitCode::Success);
}
