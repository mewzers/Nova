use crate::debug;
use crate::display::Display;
use crate::keypad::Keypad;
use crate::memory::{with_fontset, FONT_START, MEMORY_SIZE, PROGRAM_START};

// EN: Compatibility switches for ROM/interpreter behavior differences.
// FR: Interrupteurs de compatibilite pour les differences de comportement ROM/interpreteur.
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Default)]
pub struct CpuQuirks {
    pub shift_uses_vy: bool,
    pub jump_uses_vx: bool,
    pub draw_clips: bool,
    pub load_store_increment_i: bool,
    pub logic_clears_vf: bool,
}

// EN: Default serializer initializer for memory (fontset preloaded).
// FR: Initialiseur de serialisation par defaut pour la memoire (police prechargee).
fn default_memory() -> [u8; MEMORY_SIZE] {
    with_fontset()
}

// EN: CHIP-8 CPU core state.
// FR: Etat du coeur CPU CHIP-8.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct CPU {
    // EN: General purpose registers V0..VF.
    // FR: Registres generalistes V0..VF.
    pub v: [u8; 16],
    // EN: Index register.
    // FR: Registre index.
    pub i: u16,
    // EN: Program counter.
    // FR: Compteur ordinal (PC).
    pub pc: u16,
    // EN: Stack pointer.
    // FR: Pointeur de pile.
    pub sp: u8,
    // EN: Call stack (16 levels).
    // FR: Pile d appels (16 niveaux).
    pub stack: [u16; 16],
    // EN: Delay timer (60 Hz).
    // FR: Timer de delai (60 Hz).
    pub delay_timer: u8,
    // EN: Sound timer (60 Hz).
    // FR: Timer sonore (60 Hz).
    pub sound_timer: u8,
    // EN: Full memory space; skipped in persistence and rebuilt on load.
    // FR: Espace memoire complet; ignore en persistence et reconstruit au chargement.
    #[serde(skip, default = "default_memory")]
    pub memory: [u8; MEMORY_SIZE],
}

impl CPU {
    // EN: Create a new CPU instance with default state and fontset loaded.
    // FR: Cree une nouvelle instance de CPU avec l etat par defaut et la police chargee.
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START as u16,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            memory: with_fontset(),
        }
    }

    // EN: Hard reset all runtime registers/state and reload fontset.
    // FR: Reinitialise totalement les registres/etat runtime et recharge la police.
    pub fn hard_reset(&mut self) {
        self.v = [0; 16];
        self.i = 0;
        self.pc = PROGRAM_START as u16;
        self.sp = 0;
        self.stack = [0; 16];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.memory = with_fontset();
    }

    // EN: Load ROM bytes into memory at PROGRAM_START and return bytes copied.
    // FR: Charge les octets ROM en memoire a PROGRAM_START et retourne le nombre copie.
    pub fn load_program(&mut self, program: &[u8]) -> usize {
        let capacity = MEMORY_SIZE.saturating_sub(PROGRAM_START);
        let len = capacity.min(program.len());
        self.memory[PROGRAM_START..PROGRAM_START + len].copy_from_slice(&program[..len]);
        debug::log("cpu_program_loaded");
        len
    }

    // EN: Execute one CPU cycle (fetch + decode + execute).
    // FR: Execute un cycle CPU (fetch + decode + execute).
    pub fn cycle(&mut self, display: &mut Display, keypad: &Keypad, quirks: CpuQuirks) {
        let opcode = self.fetch();
        self.execute(opcode, display, keypad, quirks);
    }

    // EN: Tick delay and sound timers by one step.
    // FR: Incremente les timers delai et son d un pas.
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // EN: Fetch 16-bit opcode at PC and advance PC by 2.
    // FR: Lit l opcode 16 bits au PC et avance le PC de 2.
    fn fetch(&mut self) -> u16 {
        let pc = self.pc as usize;
        if pc + 1 >= self.memory.len() {
            return 0;
        }
        let hi = self.memory[pc] as u16;
        let lo = self.memory[pc + 1] as u16;
        self.pc = self.pc.wrapping_add(2);
        (hi << 8) | lo
    }

    // EN: Decode and execute one opcode.
    // FR: Decode et execute un opcode.
    fn execute(&mut self, opcode: u16, display: &mut Display, keypad: &Keypad, quirks: CpuQuirks) {
        let n1 = (opcode & 0xF000) >> 12;
        let n2 = (opcode & 0x0F00) >> 8;
        let n3 = (opcode & 0x00F0) >> 4;
        let n4 = opcode & 0x000F;

        // EN: Main opcode dispatch table.
        // FR: Table principale de dispatch des opcodes.
        match (n1, n2, n3, n4) {
            (0, 0, 0xE, 0) => {
                display.clear();
            }
            (0, 0, 0xE, 0xE) => {
                if self.sp == 0 {
                    return;
                }
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            (1, _, _, _) => {
                self.pc = opcode & 0x0FFF;
            }
            (2, _, _, _) => {
                if self.sp as usize >= self.stack.len() {
                    return;
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp.wrapping_add(1);
                self.pc = opcode & 0x0FFF;
            }
            (3, x, _, _) => {
                if self.v[x as usize] == (opcode & 0x00FF) as u8 {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (4, x, _, _) => {
                if self.v[x as usize] != (opcode & 0x00FF) as u8 {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (5, x, y, 0) => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (6, x, _, _) => {
                self.v[x as usize] = (opcode & 0x00FF) as u8;
            }
            (7, x, _, _) => {
                self.v[x as usize] = self.v[x as usize].wrapping_add((opcode & 0x00FF) as u8);
            }
            (8, x, y, 0) => {
                self.v[x as usize] = self.v[y as usize];
            }
            (8, x, y, 1) => {
                self.v[x as usize] |= self.v[y as usize];
                if quirks.logic_clears_vf {
                    self.v[0xF] = 0;
                }
            }
            (8, x, y, 2) => {
                self.v[x as usize] &= self.v[y as usize];
                if quirks.logic_clears_vf {
                    self.v[0xF] = 0;
                }
            }
            (8, x, y, 3) => {
                self.v[x as usize] ^= self.v[y as usize];
                if quirks.logic_clears_vf {
                    self.v[0xF] = 0;
                }
            }
            (8, x, y, 4) => {
                let (result, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[x as usize] = result;
                self.v[0xF] = carry as u8;
            }
            (8, x, y, 5) => {
                let (result, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[x as usize] = result;
                self.v[0xF] = (!borrow) as u8;
            }
            (8, x, y, 6) => {
                let source = if quirks.shift_uses_vy {
                    self.v[y as usize]
                } else {
                    self.v[x as usize]
                };
                self.v[0xF] = source & 0x1;
                self.v[x as usize] = source >> 1;
            }
            (8, x, y, 7) => {
                let (result, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[x as usize] = result;
                self.v[0xF] = (!borrow) as u8;
            }
            (8, x, y, 0xE) => {
                let source = if quirks.shift_uses_vy {
                    self.v[y as usize]
                } else {
                    self.v[x as usize]
                };
                self.v[0xF] = (source >> 7) & 0x1;
                self.v[x as usize] = source << 1;
            }
            (9, x, y, 0) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (0xA, _, _, _) => {
                self.i = opcode & 0x0FFF;
            }
            (0xB, x, _, _) => {
                let offset = if quirks.jump_uses_vx {
                    self.v[x as usize]
                } else {
                    self.v[0]
                };
                self.pc = (opcode & 0x0FFF) + offset as u16;
            }
            (0xC, x, _, _) => {
                let random: u8 = rand::random();
                self.v[x as usize] = random & (opcode & 0x00FF) as u8;
            }
            (0xD, x, y, n) => {
                let vx = self.v[x as usize] as usize;
                let vy = self.v[y as usize] as usize;
                self.v[0xF] = 0;

                for row in 0..n as usize {
                    let sprite_byte = self
                        .memory
                        .get(self.i as usize + row)
                        .copied()
                        .unwrap_or(0);
                    for bit in 0..8 {
                        if (sprite_byte & (0x80 >> bit)) == 0 {
                            continue;
                        }
                        if quirks.draw_clips && (vx + bit >= 64 || vy + row >= 32) {
                            continue;
                        }
                        let px = (vx + bit) % 64;
                        let py = (vy + row) % 32;
                        if display.set_pixel(px, py) {
                            self.v[0xF] = 1;
                        }
                    }
                }
            }
            (0xE, x, 9, 0xE) => {
                let key = (self.v[x as usize] & 0x0F) as usize;
                if keypad.is_pressed(key) {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (0xE, x, 0xA, 1) => {
                let key = (self.v[x as usize] & 0x0F) as usize;
                if !keypad.is_pressed(key) {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            (0xF, x, 0, 7) => {
                self.v[x as usize] = self.delay_timer;
            }
            (0xF, x, 0, 0xA) => {
                if let Some(key) = keypad.first_pressed() {
                    self.v[x as usize] = key;
                } else {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            (0xF, x, 1, 5) => {
                self.delay_timer = self.v[x as usize];
            }
            (0xF, x, 1, 8) => {
                self.sound_timer = self.v[x as usize];
            }
            (0xF, x, 1, 0xE) => {
                self.i = self.i.wrapping_add(self.v[x as usize] as u16);
            }
            (0xF, x, 2, 9) => {
                self.i = (FONT_START as u16) + (self.v[x as usize] as u16 * 5);
            }
            (0xF, x, 3, 3) => {
                let base = self.i as usize;
                if let Some(slice) = self.memory.get_mut(base..base + 3) {
                    slice[0] = self.v[x as usize] / 100;
                    slice[1] = (self.v[x as usize] / 10) % 10;
                    slice[2] = self.v[x as usize] % 10;
                }
            }
            (0xF, x, 5, 5) => {
                let base = self.i as usize;
                let max = (x as usize + 1).min(self.memory.len().saturating_sub(base));
                for idx in 0..max {
                    self.memory[base + idx] = self.v[idx];
                }
                if quirks.load_store_increment_i {
                    self.i = self.i.wrapping_add(x + 1);
                }
            }
            (0xF, x, 6, 5) => {
                let base = self.i as usize;
                let max = (x as usize + 1).min(self.memory.len().saturating_sub(base));
                for idx in 0..max {
                    self.v[idx] = self.memory[base + idx];
                }
                if quirks.load_store_increment_i {
                    self.i = self.i.wrapping_add(x + 1);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CPU, CpuQuirks};
    use crate::display::Display;
    use crate::keypad::Keypad;
    use crate::memory::{MEMORY_SIZE, PROGRAM_START};

    // EN: Build a CPU/display/keypad trio and load opcodes at PROGRAM_START.
    // FR: Construit un trio CPU/affichage/clavier et charge les opcodes a PROGRAM_START.
    fn setup_with_program(opcodes: &[u16]) -> (CPU, Display, Keypad) {
        let mut cpu = CPU::new();
        let mut program = Vec::with_capacity(opcodes.len() * 2);
        for op in opcodes {
            program.push((op >> 8) as u8);
            program.push((op & 0xFF) as u8);
        }
        cpu.load_program(&program);
        (cpu, Display::new(), Keypad::new())
    }

    #[test]
    fn op_6xnn_loads_immediate_into_register() {
        let (mut cpu, mut display, keypad) = setup_with_program(&[0x60AB]);
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        assert_eq!(cpu.v[0], 0xAB);
        assert_eq!(cpu.pc, (PROGRAM_START + 2) as u16);
    }

    #[test]
    fn op_7xnn_adds_with_wrap() {
        let (mut cpu, mut display, keypad) = setup_with_program(&[0x60FF, 0x7002]);
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        assert_eq!(cpu.v[0], 0x01);
    }

    #[test]
    fn op_8xy4_sets_carry_flag() {
        let (mut cpu, mut display, keypad) = setup_with_program(&[0x60F0, 0x6115, 0x8014]);
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        assert_eq!(cpu.v[0], 0x05);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn op_dxyn_draws_and_sets_collision() {
        let (mut cpu, mut display, keypad) = setup_with_program(&[
            0x6000, // V0 = 0
            0x6100, // V1 = 0
            0xA300, // I = 0x300
            0xD011, // draw 1 row at (V0, V1)
            0xD011, // draw same row again -> collision
        ]);
        cpu.memory[0x300] = 0b1000_0000;
        for _ in 0..4 {
            cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        }
        assert_eq!(cpu.v[0xF], 0);
        assert!(display.get_pixel(0, 0));

        cpu.cycle(&mut display, &keypad, CpuQuirks::default());
        assert_eq!(cpu.v[0xF], 1);
        assert!(!display.get_pixel(0, 0));
    }

    #[test]
    fn fx55_fx65_respect_increment_i_quirk() {
        let quirks_no_inc = CpuQuirks {
            load_store_increment_i: false,
            ..CpuQuirks::default()
        };
        let quirks_inc = CpuQuirks {
            load_store_increment_i: true,
            ..CpuQuirks::default()
        };

        // Fx55 without increment
        let (mut cpu, mut display, keypad) = setup_with_program(&[0x6001, 0x6102, 0xA300, 0xF155]);
        for _ in 0..4 {
            cpu.cycle(&mut display, &keypad, quirks_no_inc);
        }
        assert_eq!(cpu.memory[0x300], 1);
        assert_eq!(cpu.memory[0x301], 2);
        assert_eq!(cpu.i, 0x300);

        // Fx65 with increment
        let (mut cpu2, mut display2, keypad2) = setup_with_program(&[0xA300, 0xF165]);
        cpu2.memory[0x300] = 0xAA;
        cpu2.memory[0x301] = 0xBB;
        cpu2.cycle(&mut display2, &keypad2, quirks_inc);
        cpu2.cycle(&mut display2, &keypad2, quirks_inc);
        assert_eq!(cpu2.v[0], 0xAA);
        assert_eq!(cpu2.v[1], 0xBB);
        assert_eq!(cpu2.i, 0x302);
    }

    #[test]
    fn load_program_clamps_to_memory_capacity() {
        let mut cpu = CPU::new();
        let capacity = MEMORY_SIZE - PROGRAM_START;
        let program = vec![0xEA; capacity + 16];
        let loaded = cpu.load_program(&program);
        assert_eq!(loaded, capacity);
        assert_eq!(cpu.memory[PROGRAM_START], 0xEA);
        assert_eq!(cpu.memory[MEMORY_SIZE - 1], 0xEA);
    }
}

