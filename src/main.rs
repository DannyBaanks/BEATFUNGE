//! beatfunge — CLI minimo: corre un grid, lista el catalogo, o verifica la
//! equivalencia de los nueve programas canonicos contra el oraculo 1D.

use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

use beatfunge::corpus;
use beatfunge::motor1d::ejecutar_1d;
use beatfunge::motor2d::ejecutar_2d;
use beatfunge::{Evento, Rejilla};

fn fmt_evento(e: &Evento) -> String {
    match e {
        Evento::Nota { hz, divisor, ticks } => {
            format!(
                "NOTA   {:>4} Hz  divisor={:>5}  {:.1} Hz real  ticks={}",
                hz,
                divisor,
                1_193_182.0 / *divisor as f64,
                ticks
            )
        }
        Evento::Silencio { ticks } => format!("SILENCIO (<19 Hz)      ticks={}", ticks),
        Evento::Pausa { ticks } => format!("PAUSA                  ticks={}", ticks),
        Evento::AltavozToggle => "TOGGLE-SPEAKER".to_string(),
        Evento::Alto => "ALTO".to_string(),
    }
}

fn resumen(eventos: &[Evento]) -> (usize, u32) {
    let notas = eventos
        .iter()
        .filter(|e| matches!(e, Evento::Nota { .. } | Evento::Silencio { .. }))
        .count();
    let ticks: u32 = eventos
        .iter()
        .map(|e| match e {
            Evento::Nota { ticks, .. } => *ticks as u32,
            Evento::Silencio { ticks } | Evento::Pausa { ticks } => *ticks as u32,
            _ => 0,
        })
        .sum();
    (notas, ticks)
}

fn correr(ruta: &str, verboso: bool) -> Result<Vec<Evento>, String> {
    let texto = fs::read_to_string(ruta).map_err(|e| format!("no pude leer {ruta}: {e}"))?;
    let rej = Rejilla::de_lineas(&texto)?;
    let ev = ejecutar_2d(&rej, &[])?;

    if verboso {
        for e in &ev {
            println!("  {}", fmt_evento(e));
        }
    }
    let (notas, ticks) = resumen(&ev);
    let dur_s = ticks as f64 / 18.2065;
    println!(
        "{}: {} eventos, {} notas/silencios, {} ticks (~{:.1}s), {}",
        ruta,
        ev.len(),
        notas,
        ticks,
        dur_s,
        if matches!(ev.last(), Some(Evento::Alto)) {
            "termina"
        } else {
            "NO TERMINA"
        }
    );
    Ok(ev)
}

fn equivalencia() -> i32 {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("programs");
    let mut fallos = 0;
    println!(
        "{:<20} {:>10} {:>12} {:>12} {:>8}",
        "programa", "veredicto", "eventos 1d", "eventos 2d", "J/Z en 2d"
    );
    for (i, (nombre, orig)) in corpus::NOMBRES
        .iter()
        .zip(corpus::ORIGINALES.iter())
        .enumerate()
    {
        let ruta = base.join(format!("{nombre}.grid"));
        let texto = fs::read_to_string(&ruta).expect(&format!("falta {}", ruta.display()));
        let rej = Rejilla::de_lineas(&texto).expect("rejilla invalida");

        let ev1 = ejecutar_1d(orig, &[]).expect("el original 1D no corre");
        let ev2 = ejecutar_2d(&rej, &[]).expect("el port 2D no corre");

        let sin_saltos = !rej.contiene_saltos_absolutos();
        let ok = ev1 == ev2 && sin_saltos;
        if !ok {
            fallos += 1;
        }
        println!(
            "{:>2}. {:<17} {:>10} {:>12} {:>12} {:>8}",
            i + 1,
            nombre,
            if ok { "EQUIVALENTE" } else { "DIFERENTE" },
            ev1.len(),
            ev2.len(),
            if sin_saltos { "0 (ok)" } else { "SI (!)" }
        );
    }
    if fallos > 0 {
        println!("\n{} programa(s) NO equivalentes", fallos);
        1
    } else {
        println!("\n9/9 equivalentes. Control de flujo geometrico == control lineal+J/Z.");
        0
    }
}

fn ayuda() {
    println!("beatfunge — simulador de referencia de BEATfunge (32x16 toroidal)");
    println!();
    println!("Uso:");
    println!("  beatfunge run <archivo.grid> [--verbose]");
    println!("  beatfunge canon");
    println!("  beatfunge equiv");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        ayuda();
        return;
    }
    let code = match args[0].as_str() {
        "run" => match args.get(1) {
            Some(ruta) => {
                let verboso = args.iter().any(|a| a == "--verbose" || a == "-v");
                match correr(ruta, verboso) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("error: {e}");
                        1
                    }
                }
            }
            None => {
                ayuda();
                1
            }
        },
        "canon" => {
            let mut rc = 0;
            for nombre in corpus::NOMBRES {
                let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("programs")
                    .join(format!("{nombre}.grid"));
                if let Err(e) = correr(ruta.to_str().unwrap(), false) {
                    eprintln!("{e}");
                    rc = 1;
                }
            }
            rc
        }
        "equiv" => equivalencia(),
        _ => {
            ayuda();
            1
        }
    };
    exit(code);
}
