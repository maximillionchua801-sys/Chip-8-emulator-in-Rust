mod chip_8;
mod loading;
use chip_8::Memory;
use loading::rom;
use minifb::{Window, WindowOptions,Key};

    const FONTSET_SIZE: u8 = 80;
    const FONTSET: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];
    const FONTSET_START_ADDRESS: usize = 0x50;

    const KEY_MAP: [minifb::Key; 16] = [
    Key::X, Key::Key1, Key::Key2, Key::Key3,
    Key::Q, Key::W, Key::E, Key::A,
    Key::S, Key::D, Key::Z, Key::C,
    Key::Key4, Key::R, Key::F, Key::V,
];
fn main() {
    //    for i in (0x200..0x220).step_by(2) {
    //     let opcode =
    //         ((mem1.mem[i] as u16) << 8) |
    //         mem1.mem[i + 1] as u16;
    // }
    let mut mem1 = Memory::new();


    //load fontset for sprites
    for (i, byte) in FONTSET.iter().enumerate() {
        mem1.mem[FONTSET_START_ADDRESS + i] = *byte;
    }

    const START_ADDRESS: usize = 0x200;
    //load rom
    let rom = rom::load("Tetris [Fran Dachille, 1991].ch8");

    println!("ROM size: {}", rom.data.len());

    if let Some(first_byte) = rom.data.get(0) {
        println!("First byte: {}", first_byte);
    }
    for (i, byte) in rom.data.iter().enumerate() {
        mem1.mem[START_ADDRESS + i] = *byte;
    }
   
   
   
   
    let mut window = Window::new("CHIP-8", 64 * 10, 32 * 10, WindowOptions::default())
        .expect("did not create new window");
    let mut buffer = vec![0u32; 64 * 32];

  
  
    while window.is_open() && !window.is_key_down(Key::Escape) {
        mem1.input(&window);
        mem1.cycle();
        

        for i in 0..mem1.graphics.len(){
            buffer[i] = if mem1.graphics[i] {
                0x00FFFFFF
            }
            else{
                0x00000000
            }
        }
        window
            .update_with_buffer(&buffer, 64, 32)
            .expect("Failed to update window");
    }
}
