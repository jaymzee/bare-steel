#![no_std]
#![no_main]

use blog_os::{print_test_name, print_test_passed, print_test_failed_because};
use blog_os::{exit_qemu, QemuExitCode};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

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
