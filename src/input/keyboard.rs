use crate::{arch::*, locked::Locked};
use conquer_once::spin::OnceCell;
use pc_keyboard::{layouts::Uk105Key, *};

const KEYBOARD_PORT: u16 = 0x60;

pub fn initialize() {}

static LAST_KEY: OnceCell<Locked<Option<DecodedKey>>> = OnceCell::uninit();

static KEYBOARD: OnceCell<Locked<Keyboard<layouts::Uk105Key, ScancodeSet1>>> = OnceCell::uninit();

pub(crate) fn keypress() {
    let mut kb = KEYBOARD
        .get_or_init(|| Locked::new(Keyboard::new(Uk105Key, ScancodeSet1, HandleControl::Ignore)))
        .lock();
    let data = inb(KEYBOARD_PORT);
    if let Ok(Some(event)) = kb.add_byte(data) {
        if let Some(key) = kb.process_keyevent(event) {
            set_last_key(key);
        }
    }
}

fn set_last_key(key: DecodedKey) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        LAST_KEY.init_once(|| Locked::new(None));
        *LAST_KEY.get().unwrap().lock() = Some(key);
    });
}

fn get_last_key() -> Option<DecodedKey> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        LAST_KEY.init_once(|| Locked::new(None));
        *LAST_KEY.get().unwrap().lock()
    })
}

pub fn read_char() -> Option<char> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(key) = get_last_key() {
            match key {
                DecodedKey::Unicode(chr) => Some(chr),
                DecodedKey::RawKey(_) => None,
            }
        } else {
            None
        }
    })
}

pub fn read_keycode() -> Option<KeyCode> {
    if let Some(key) = get_last_key() {
        match key {
            DecodedKey::RawKey(kc) => Some(kc),
            _ => None,
        }
    } else {
        None
    }
}

pub fn clear() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        LAST_KEY.init_once(|| Locked::new(None));
        *LAST_KEY.get().unwrap().lock() = None;
    });
}
