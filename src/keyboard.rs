use pc_keyboard::DecodedKey;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::println;

const KEYBUFFER_SIZE: usize = 128;

pub static KEYBUFFER: Mutex<KeyCodeIter> =
    Mutex::new(KeyCodeIter {
        buffer: [DecodedKey::Unicode('\0'); KEYBUFFER_SIZE],
        head: 0,
        tail: 0
    });

#[derive(Clone, Debug)]
pub struct KeyCodeIter {
    buffer: [DecodedKey; KEYBUFFER_SIZE],
    head: usize,
    tail: usize,
}

// TODO: self.tail and self.head can overflow
impl Iterator for KeyCodeIter {
    type Item = DecodedKey;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head <= self.tail {
            return None;
        } else {
            let ret = self.buffer[self.tail % KEYBUFFER_SIZE];
            self.tail += 1;
            self.head %= KEYBUFFER_SIZE;
            return Some(ret);
        }
    }
}

impl KeyCodeIter {
    pub fn push(&mut self, key: DecodedKey) {
        println!("PUSH: {}-{} {:?}", self.head, self.tail, key);
        self.buffer[self.head % KEYBUFFER_SIZE] = key;
        self.head += 1;
    }
}
