//! El motor 1D de BEAT, fiel a BEAT.asm. Sirve como oraculo: cualquier
//! programa BEATfunge valido debe reproducir exactamente su secuencia de
//! eventos.

use crate::{dur_ticks, Estado, Evento, TABLA_TEMPO};

const BASE_PROGRAMA: usize = 0x0600;

pub fn ejecutar_1d(programa: &[u8], entrada: &[u8]) -> Result<Vec<Evento>, String> {
    let mut st = Estado {
        entrada: entrada.to_vec(),
        ..Estado::default()
    };
    let mut ip: usize = 0;
    let mut eventos = Vec::new();
    let mut pasos: usize = 0;

    loop {
        pasos += 1;
        if pasos > 2_000_000 {
            return Err("limite de pasos (posible bucle infinito)".into());
        }
        let op = *programa.get(ip).ok_or("IP fuera del programa")?;
        ip += 1;
        match op {
            0 => {
                eventos.push(Evento::Alto);
                return Ok(eventos);
            }
            b'>' => st.cinta_derecha(),
            b'<' => st.cinta_izquierda(),
            b'+' => st.reg = st.reg.wrapping_add(1),
            b'-' => st.reg = st.reg.wrapping_sub(1),
            b'.' => st.tocar(&mut eventos),
            b',' => st.leer_tecla()?,
            b'[' => {
                if st.reg == 0 {
                    ip = buscar_pareja_adelante(programa, ip)?;
                }
            }
            b']' => {
                if st.reg != 0 {
                    ip = buscar_pareja_atras(programa, ip - 1)?;
                }
            }
            b'T' => {
                let d = *programa.get(ip).ok_or("T sin operando")?;
                ip += 1;
                if d.is_ascii_digit() {
                    st.tempo = TABLA_TEMPO[(d - b'0') as usize];
                }
            }
            b'D' => {
                let d = *programa.get(ip).ok_or("D sin operando")?;
                ip += 1;
                if d.is_ascii_digit() {
                    st.dur = (d - b'0') as u16;
                }
            }
            b'P' => st.pausa(&mut eventos),
            b'R' => st.reg = st.cinta[st.ptr] as u16,
            b'W' => st.cinta[st.ptr] = (st.reg & 0xFF) as u8,
            b'J' => {
                let lo = *programa.get(ip).ok_or("J truncado")? as usize;
                let hi = *programa.get(ip + 1).ok_or("J truncado")? as usize;
                ip = (hi << 8 | lo)
                    .checked_sub(BASE_PROGRAMA)
                    .ok_or("J a direccion < 0x0600")?;
            }
            b'Z' => {
                let lo = *programa.get(ip).ok_or("Z truncado")? as usize;
                let hi = *programa.get(ip + 1).ok_or("Z truncado")? as usize;
                ip += 2;
                if st.reg == 0 {
                    ip = (hi << 8 | lo)
                        .checked_sub(BASE_PROGRAMA)
                        .ok_or("Z a direccion < 0x0600")?;
                }
            }
            b'S' => eventos.push(Evento::AltavozToggle),
            b'N' => {
                let mut v: u16 = 0;
                loop {
                    match programa.get(ip) {
                        Some(d) if d.is_ascii_digit() => {
                            v = v.wrapping_mul(10).wrapping_add((d - b'0') as u16);
                            ip += 1;
                        }
                        _ => break,
                    }
                }
                st.reg = v;
            }
            b'H' => {
                eventos.push(Evento::Alto);
                return Ok(eventos);
            }
            _ => {} // NOP: cualquier otro byte lo ignora el motor
        }
        // mantener disponible el calculo para analisis futuros
        let _ = dur_ticks(st.tempo, st.dur);
    }
}

fn buscar_pareja_adelante(prog: &[u8], mut ip: usize) -> Result<usize, String> {
    let mut prof: usize = 1;
    loop {
        let c = *prog.get(ip).ok_or("'[' sin pareja")?;
        ip += 1;
        match c {
            0 => return Err("'[' sin pareja (NUL)".into()),
            b'[' => prof += 1,
            b']' => {
                prof -= 1;
                if prof == 0 {
                    return Ok(ip);
                }
            }
            _ => {}
        }
    }
}

fn buscar_pareja_atras(prog: &[u8], desde: usize) -> Result<usize, String> {
    let mut prof: usize = 1;
    let mut i = desde;
    loop {
        if i == 0 {
            return Err("']' sin pareja".into());
        }
        i -= 1;
        match prog[i] {
            b']' => prof += 1,
            b'[' => {
                prof -= 1;
                if prof == 0 {
                    return Ok(i + 1); // justo despues del '[' (la condicion no se re-evalua)
                }
            }
            _ => {}
        }
    }
}
