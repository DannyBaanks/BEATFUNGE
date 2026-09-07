//! El motor 2D de BEATfunge: el mismo universo BEAT, control geometrico.
//!
//! Colision resuelta: en BEAT `>` y `<` movian la cinta. En BEATfunge
//! `> < ^ v` son la geometria del IP; la cinta se mueve con `{` y `}`.
//!
//! Mapa completo:
//! ```text
//!   > < ^ v  : fijan la direccion del puntero de instruccion
//!   _        : reg == 0 -> derecha; reg != 0 -> izquierda
//!   |        : reg == 0 -> abajo;  reg != 0 -> arriba
//!   #        : puente (salta la siguiente celda en la direccion actual)
//!   { }      : puntero de cinta izquierda / derecha
//!   + -      : registro +/- 1
//!   . ,      : tocar registro / entrada de teclado
//!   [ ]      : lazo (v0.1: busqueda en linea recta y toroidal en la
//!              direccion actual; previsto ser reemplazado por geometria)
//!   T n      : tempo (tabla, 0..9)
//!   D n      : duracion (0..9)
//!   N <hz>   : parse decimal en la direccion de avance
//!   P        : pausa
//!   R W      : cinta -> registro / registro -> cinta (byte bajo)
//!   S        : toggle del altavoz
//!   H / @    : alto
//! ```
//!
//! Borrados: J y Z (saltos absolutos). Su papel lo toman `_` y `|`.

use crate::rejilla::{Rejilla, ALTO, ANCHO};
use crate::{Estado, Evento, TABLA_TEMPO};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dir {
    Derecha,
    Izquierda,
    Arriba,
    Abajo,
}

fn paso(x: usize, y: usize, dir: Dir) -> (usize, usize) {
    match dir {
        Dir::Derecha => ((x + 1) % ANCHO, y),
        Dir::Izquierda => ((x + ANCHO - 1) % ANCHO, y),
        Dir::Arriba => (x, (y + ALTO - 1) % ALTO),
        Dir::Abajo => (x, (y + 1) % ALTO),
    }
}

pub fn ejecutar_2d(rej: &Rejilla, entrada: &[u8]) -> Result<Vec<Evento>, String> {
    if rej.contiene_saltos_absolutos() {
        return Err("el programa contiene J o Z: BEATfunge no tiene saltos absolutos".into());
    }
    let mut st = Estado {
        entrada: entrada.to_vec(),
        ..Estado::default()
    };
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut dir = Dir::Derecha;
    let mut eventos = Vec::new();
    let mut pasos: usize = 0;

    loop {
        pasos += 1;
        if pasos > 2_000_000 {
            return Err("limite de pasos (posible bucle infinito)".into());
        }
        let op = rej.celda(x, y);
        let mut avanzar = true;
        match op {
            b'>' => dir = Dir::Derecha,
            b'<' => dir = Dir::Izquierda,
            b'^' => dir = Dir::Arriba,
            b'v' => dir = Dir::Abajo,
            b'_' => {
                dir = if st.reg == 0 {
                    Dir::Derecha
                } else {
                    Dir::Izquierda
                };
            }
            b'|' => {
                dir = if st.reg == 0 { Dir::Abajo } else { Dir::Arriba };
            }
            b'#' => {
                let (nx, ny) = paso(x, y, dir);
                x = nx;
                y = ny;
            }
            b'{' => st.cinta_izquierda(),
            b'}' => st.cinta_derecha(),
            b'+' => st.reg = st.reg.wrapping_add(1),
            b'-' => st.reg = st.reg.wrapping_sub(1),
            b'.' => st.tocar(&mut eventos),
            b',' => st.leer_tecla()?,
            b'T' => {
                let (ox, oy) = paso(x, y, dir);
                let d = rej.celda(ox, oy);
                if d.is_ascii_digit() {
                    st.tempo = TABLA_TEMPO[(d - b'0') as usize];
                }
                x = ox;
                y = oy;
            }
            b'D' => {
                let (ox, oy) = paso(x, y, dir);
                let d = rej.celda(ox, oy);
                if d.is_ascii_digit() {
                    st.dur = (d - b'0') as u16;
                }
                x = ox;
                y = oy;
            }
            b'N' => {
                let mut v: u16 = 0;
                let (mut cx, mut cy) = paso(x, y, dir);
                loop {
                    let d = rej.celda(cx, cy);
                    if d.is_ascii_digit() {
                        v = v.wrapping_mul(10).wrapping_add((d - b'0') as u16);
                        let (nx, ny) = paso(cx, cy, dir);
                        cx = nx;
                        cy = ny;
                    } else {
                        break;
                    }
                }
                // como en el metal: el byte no-digito SE EJECUTA a continuacion
                x = cx;
                y = cy;
                avanzar = false;
                st.reg = v;
            }
            b'P' => st.pausa(&mut eventos),
            b'R' => st.reg = st.cinta[st.ptr] as u16,
            b'W' => st.cinta[st.ptr] = (st.reg & 0xFF) as u8,
            b'S' => eventos.push(Evento::AltavozToggle),
            b'H' | b'@' => {
                eventos.push(Evento::Alto);
                return Ok(eventos);
            }
            b'[' => {
                if st.reg == 0 {
                    let (bx, by) = pareja_adelante(rej, x, y, dir)?;
                    x = bx;
                    y = by;
                }
            }
            b']' => {
                if st.reg != 0 {
                    let (bx, by) = pareja_atras(rej, x, y, dir)?;
                    x = bx;
                    y = by;
                }
            }
            _ => {} // NOP: espacios, digitos sueltos, otros bytes
        }
        if avanzar {
            let (nx, ny) = paso(x, y, dir);
            x = nx;
            y = ny;
        }
    }
}

/// Busca el ']' que empareja a un '[' viajando en linea recta toroidal.
fn pareja_adelante(rej: &Rejilla, x: usize, y: usize, dir: Dir) -> Result<(usize, usize), String> {
    let (mut cx, mut cy) = paso(x, y, dir);
    let mut prof: usize = 1;
    for _ in 0..(ANCHO * ALTO) {
        match rej.celda(cx, cy) {
            b'[' => prof += 1,
            b']' => {
                prof -= 1;
                if prof == 0 {
                    return Ok((cx, cy));
                }
            }
            _ => {}
        }
        let (nx, ny) = paso(cx, cy, dir);
        cx = nx;
        cy = ny;
    }
    Err("'[' sin pareja en la rejilla".into())
}

/// Busca el '[' que empareja a un ']' viajando en linea recta en reversa.
fn pareja_atras(rej: &Rejilla, x: usize, y: usize, dir: Dir) -> Result<(usize, usize), String> {
    let rev = match dir {
        Dir::Derecha => Dir::Izquierda,
        Dir::Izquierda => Dir::Derecha,
        Dir::Arriba => Dir::Abajo,
        Dir::Abajo => Dir::Arriba,
    };
    let (mut cx, mut cy) = paso(x, y, rev);
    let mut prof: usize = 1;
    for _ in 0..(ANCHO * ALTO) {
        match rej.celda(cx, cy) {
            b']' => prof += 1,
            b'[' => {
                prof -= 1;
                if prof == 0 {
                    return Ok((cx, cy));
                }
            }
            _ => {}
        }
        let (nx, ny) = paso(cx, cy, rev);
        cx = nx;
        cy = ny;
    }
    Err("']' sin pareja en la rejilla".into())
}
