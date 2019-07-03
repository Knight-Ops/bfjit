use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use libc;

fn write_relo_info(instruction_stream: &mut Vec<u8>, location: usize, value: i32) {
    for (idx, y) in value.to_le_bytes().iter().enumerate() {
        instruction_stream[location + idx] = *y;
    }
}

fn interpret_bf(bf: &str) {

    let mut instruction_stream = vec![
        0x55, // push rbp
        0x48, 0x89, 0xE5, // move rsp, rbp
        0x41, 0x54, // pushq r12
        0x41, 0x55, // pushq r13
        0x41, 0x56, // pushq r14
        0x49, 0x89, 0xFC, // movq r12, rdi
        0x49, 0x89, 0xF5, // movq r13, rsi
        0x49, 0x89, 0xD6, // movq r14, rdx
        0x48, 0x81, 0xEC, 0x38, 0x75, 0x00, 0x00, // subq rsp, 30008
        0x48, 0x8D, 0x3C, 0x24, // leaq rdi, [rsp]
        0xBE, 0x00, 0x00, 0x00, 0x00, // movl esi, 0
        0x48, 0xC7, 0xC2, 0x30, 0x75, 0x00, 0x00, // movq rdx, 30000
        0x41, 0xFF, 0xD4, // callq r12
        0x49, 0x89, 0xE4 // movq r12, rsp
    ];


    let mut epilogue = vec![
        0x48, 0x81, 0xC4, 0x38, 0x75, 0x00, 0x00,
        0x41, 0x5E,
        0x41, 0x5D,
        0x41, 0x5C,
        0x5D,
        0xC3
    ];

    let mut relro_list = vec![];

    for instr in bf.chars() {
        match instr {
            '>' => {
                let mut inc_instr = vec![0x49, 0xFF, 0xC4];
                instruction_stream.append(&mut inc_instr);
            },
            '<' => {
                let mut dec_instr = vec![0x49, 0xFF, 0xCC];
                instruction_stream.append(&mut dec_instr);
            },
            '+' => {
                let mut inc_ptr = vec![0x41, 0xFE, 0x04, 0x24];
                instruction_stream.append(&mut inc_ptr);
            },
            '-' => {
                let mut dec_ptr = vec![0x41, 0xFE, 0x0C, 0x24];
                instruction_stream.append(&mut dec_ptr);
            },
            '.' => {
                let mut call_putchar = vec![0x41, 0x0F, 0xB6, 0x3C, 0x24,
                0x41, 0xFF, 0xD5];
                instruction_stream.append(&mut call_putchar);
            },
            ',' => {
                let mut call_getchar = vec![0x41, 0xFF, 0xD6,
                0x41, 0x88, 0x04, 0x24];
                instruction_stream.append(&mut call_getchar);
            },
            '[' => {
                let mut make_label = vec![0x41, 0x80, 0x3C, 0x24, 0x00,
                0x0F, 0x84, 0x00, 0x00, 0x00, 0x00];
                instruction_stream.append(&mut make_label);
                relro_list.push(instruction_stream.len());
            },
            ']' => {
                let mut fix_label = vec![0x41, 0x80, 0x3C, 0x24, 0x00,
                0x0F, 0x85, 0x00, 0x00, 0x00, 0x00];
                instruction_stream.append(&mut fix_label);

                let relocation_site = relro_list.pop().unwrap();
                let relative_offset = instruction_stream.len() - relocation_site;

                let instr_stream_len = instruction_stream.len();
                write_relo_info(&mut instruction_stream, instr_stream_len - 4, -(relative_offset as i32));
                write_relo_info(&mut instruction_stream, relocation_site - 4, (relative_offset as i32));
                
            },
            _ => {}
        }
    }

    instruction_stream.append(&mut epilogue);

    type Jitfunc = *const fn(*const libc::c_void, *const libc::c_void, *const libc::c_void);

    unsafe {
        let mem = libc::mmap(0 as *mut libc::c_void, instruction_stream.len(), libc::PROT_WRITE | libc::PROT_EXEC, libc::MAP_ANON | libc::MAP_PRIVATE, -1, 0);
        libc::memcpy(mem, instruction_stream.as_ptr() as *const libc::c_void, instruction_stream.len());
        let f : Jitfunc = mem as *const fn(*const libc::c_void, *const libc::c_void, *const libc::c_void);
        (*f)(libc::memset as *const libc::c_void, libc::getchar as *const libc::c_void, libc::putchar as *const libc::c_void);
    }


    for byte in instruction_stream {
        print!("{:02X}", byte);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage : {} [bf program to run]", &args[0]);
        return;
    }

    let bf_program =
        fs::read_to_string(&args[1]).expect("Error while reading provided BF program!");

    interpret_bf(&bf_program);
    // println!("{}", bf_program);
}
