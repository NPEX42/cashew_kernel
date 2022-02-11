use alloc::string::String;
use pc_keyboard::KeyCode::*;

use crate::{pit, print, arch};

pub mod keyboard;

pub mod ps2;

pub fn init() {
    keyboard::initialize();
    ps2::PS2Controller::get()
        .reinit()
        .expect("[PS/2] - Initialization Failed...");
}


pub fn prompt(prompt: &str) -> String {
    let mut output = String::new();
    'prompt_loop: loop {
        if let Some(key) = keyboard::read_keycode() {
            match key {
                Backspace => {output.pop();}
                Enter => {break 'prompt_loop;}
                _ => {}
            }
        }

        if let Some(chr) = keyboard::read_char() {
            match chr {
                '\x1b' => {}
                '\x08' => {output.pop();}
                '\n' => {break 'prompt_loop;}
                _ => { output.push(chr) }
            }
        }

        //sprint!("output: {}\n", output);

        keyboard::clear();



        print!("{}{} \r", prompt, output);
        
        pit::sleep(1);
    }

    print!("{}{} \n", prompt, output);
    keyboard::clear();
    return output;
}

pub fn wait_for_key() {
    keyboard::clear();
    while let None = keyboard::read_char() {

        pit::sleep(1);
    }
    keyboard::clear();
}