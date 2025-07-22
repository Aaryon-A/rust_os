#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::vec;
use rust_os::task::executor::Executor;
use rust_os::task::Task;
use rust_os::task::keyboard;

entry_point!(kernel_main);

mod vga_buffer;
mod serial;

// static HELLO: &[u8] = b"Hello Rust!";

#[unsafe(no_mangle)] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    
    // let vga_buffer = 0xb8000 as *mut u8;
    //
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();

    println!("Hello World{}", "!");
    //panic!("Some panic message");
    
    rust_os::init();
    
    // Breakpoint Exception
    // x86_64::instructions::interrupts::int3();
    
    // Double Fault Exception - General Case
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }
    
    // Double Fault Exception - Kernel Stack Overflow
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // 
    // stack_overflow();

    // Page Fault Error
    // let ptr = 0xdeadbeaf as *mut u8;
    // unsafe {*ptr = 42;}

    // Write Error
    /*let ptr = 0x2055f1 as *mut u8;

    // read from a code page
    unsafe { let x = *ptr; }
    println!("read worked");

    // write to a code page
    unsafe { *ptr = 42; }
    println!("write worked");*/

    // use x86_64::registers::control::Cr3;
    //
    // let (level_4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    /*use rust_os::memory::active_level_4_table;
    use x86_64::{VirtAddr, structures::paging::PageTable};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe {active_level_4_table(phys_mem_offset)};

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);

            // get the physical address from the entry and convert it
            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            // print non-empty entries of the level 3 table
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", i, entry);
                }
            }
        }
    }*/

    // use rust_os::memory::translate_addr;
    //
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let addresses = [0xb8000, 0x201008, 0x0100_0020_1a10, boot_info.physical_memory_offset];
    //
    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = unsafe {translate_addr(virt, phys_mem_offset)};
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    /*use rust_os::memory;
    use rust_os::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::{Translate, Page}, VirtAddr};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    // let mut frame_allocator = memory::EmptyFrameAllocator;

    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_map)};

    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};*/

    use rust_os::allocator;
    use rust_os::memory::{self, BootInfoFrameAllocator};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_map)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // let heap_value = Box::new(41);
    // println!("heap_value at {:p}", heap_value);
    //
    // let mut vec = Vec::new();
    // for i in 0..500 {
    //     vec.push(i);
    // }
    // println!("vec at {:p}", vec.as_slice());
    //
    // let reference_counted = Rc::new(vec![1, 2, 3]);
    // let cloned_reference = reference_counted.clone();
    // println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    // drop(reference_counted);
    // println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();
    
    println!("It did not crash!");
    rust_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}