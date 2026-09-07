//! BEATfunge — simulador de referencia.
//!
//! Dos interpretes en un solo crate:
//!
//! * [`ejecutar_1d`]: el BEAT lineal original, fiel a `BEAT.asm` (motor de
//!   arranque bare-metal, sector 4 = programa, saltos `J`/`Z` absolutos desde
//!   `0x0600`).
//! * [`ejecutar_2d`]: BEATfunge, el mismo universo musical/computacional con
//!   control de flujo proyectado a una rejilla toroidal de 32 x 16 = 512
//!   celdas (un sector de disquete exacto).
//!
//! La prueba de equivalencia compara la SECUENCIA DE EVENTOS de ambos
//! interpretes sobre los nueve programas canonicos: mismas notas, mismas
//! frecuencias pedidas, mismos ticks, mismo alto.

pub mod corpus;

pub mod motor1d;
pub mod motor2d;
pub mod rejilla;

pub use rejilla::Rejilla;

/// Tabla de tempos del motor bare-metal (indice 0-9 -> BPM).
pub const TABLA_TEMPO: [u16; 10] = [60, 80, 100, 120, 140, 160, 180, 200, 220, 240];

/// Tabla de notas del teclado para la instruccion `,` (Do4 .. Si4).
pub const TABLA_NOTAS: [u16; 12] = [262, 277, 294, 311, 330, 349, 370, 392, 415, 440, 466, 494];

/// Frecuencia del reloj del PIT 8253.
pub const RELOJ_PIT: u32 = 1_193_182;

/// Unidad de tiempo: ticks del BIOS (18.2065 Hz).
/// ticks = (1092 / BPM) * D / 4, con un minimo de 1. Igual que en el metal:
/// divisiones enteras, en este orden.
pub fn dur_ticks(tempo: u16, dur: u16) -> u16 {
    let v = (1092 / tempo) * dur / 4;
    if v == 0 {
        1
    } else {
        v
    }
}

/// Evento observable de una ejecucion. La comparacion de equivalencia se
/// hace sobre esta secuencia: NO sobre el texto del programa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evento {
    /// `.` con frecuencia ~>= 19 Hz: el divisor que recibiria el 8253 y los
    /// ticks que duraria la nota.
    Nota { hz: u16, divisor: u16, ticks: u16 },
    /// `.` con frecuencia < 19 Hz: el metal no la puede tocar (o la division
    /// desbordaria) y solo espera.
    Silencio { ticks: u16 },
    /// `P`: pausa.
    Pausa { ticks: u16 },
    /// `S`: alterna el altavoz (XOR sobre los dos bits de 61h).
    AltavozToggle,
    /// `H` (o fin del programa / `@`): alto con el altavoz apagado.
    Alto,
}

/// Estado interno comun a los dos motores.
pub struct Estado {
    pub reg: u16,
    pub tempo: u16,
    pub dur: u16,
    pub cinta: [u8; 2048],
    pub ptr: usize,
    pub entrada: Vec<u8>,
    pub entrada_pos: usize,
}

impl Default for Estado {
    fn default() -> Self {
        Estado {
            reg: 0,
            tempo: 120, // como el motor: TEMPO=120, DUR=4 al arrancar
            dur: 4,
            cinta: [0; 2048],
            ptr: 0,
            entrada: Vec::new(),
            entrada_pos: 0,
        }
    }
}

impl Estado {
    pub fn tocar(&self, eventos: &mut Vec<Evento>) {
        let ticks = dur_ticks(self.tempo, self.dur);
        if self.reg < 19 {
            eventos.push(Evento::Silencio { ticks });
        } else {
            eventos.push(Evento::Nota {
                hz: self.reg,
                divisor: (RELOJ_PIT / self.reg as u32) as u16,
                ticks,
            });
        }
    }

    pub fn pausa(&self, eventos: &mut Vec<Evento>) {
        eventos.push(Evento::Pausa {
            ticks: dur_ticks(self.tempo, self.dur),
        });
    }

    /// `,`: tecla 0..9 (o 10..12 por el rango del metal) -> nota de tabla.
    /// El motor original lee teclas hasta recibir una en rango; el simulador
    /// conserva el quirk del metal para el indice 12 (lee el primer BPM
    /// por desbordamiento adyacente de tabla, i.e. 60) para mantenerse fiel.
    pub fn leer_tecla(&mut self) -> Result<(), String> {
        loop {
            let c = *self
                .entrada
                .get(self.entrada_pos)
                .ok_or("entrada agotada")?;
            self.entrada_pos += 1;
            let d = c.wrapping_sub(b'0');
            if d <= 12 {
                self.reg = if d == 12 { 60 } else { TABLA_NOTAS[d as usize] };
                return Ok(());
            }
        }
    }

    pub fn cinta_izquierda(&mut self) {
        self.ptr = (self.ptr + 2048 - 1) % 2048;
    }
    pub fn cinta_derecha(&mut self) {
        self.ptr = (self.ptr + 1) % 2048;
    }
}
