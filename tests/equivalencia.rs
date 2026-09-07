//! Prueba de equivalencia: los nueve programas canonicos de BEAT, portados a
//! BEATfunge, deben producir EXACTAMENTE la misma secuencia de eventos que el
//! motor 1D original (fiel a BEAT.asm).

use std::fs;
use std::path::Path;

use beatfunge::corpus;
use beatfunge::motor1d::ejecutar_1d;
use beatfunge::motor2d::ejecutar_2d;
use beatfunge::{Evento, Rejilla};

fn cargar_grid(nombre: &str) -> Rejilla {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("programs")
        .join(format!("{nombre}.grid"));
    let texto = fs::read_to_string(&ruta)
        .unwrap_or_else(|e| panic!("no pude leer {}: {e}", ruta.display()));
    Rejilla::de_lineas(&texto).expect("rejilla invalida")
}

#[test]
fn los_nueve_ports_son_equivalentes() {
    for (nombre, original) in corpus::NOMBRES.iter().zip(corpus::ORIGINALES.iter()) {
        let rej = cargar_grid(nombre);
        let ev_1d = ejecutar_1d(original, &[]).expect("el original no corre");
        let ev_2d = ejecutar_2d(&rej, &[]).expect("el port no corre");
        assert_eq!(
            ev_1d, ev_2d,
            "el port {nombre} no produce la misma secuencia de eventos que el original"
        );
        assert_eq!(
            ev_2d.last(),
            Some(&Evento::Alto),
            "el port {nombre} no termina"
        );
    }
}

#[test]
fn ningun_programa_usa_saltos_absolutos() {
    for nombre in corpus::NOMBRES {
        let rej = cargar_grid(nombre);
        assert!(
            !rej.contiene_saltos_absolutos(),
            "el programma {nombre} contiene J/Z"
        );
    }
}

#[test]
fn la_geometria_soporta_bifurcacion_condicional() {
    // `_` con registro cero sale a la derecha; con registro no-cero, a la izquierda.
    let cero = Rejilla::de_lineas(">N0_H").unwrap();
    let no_cero = Rejilla::de_lineas("v  H\n>N1|").unwrap();
    assert_eq!(ejecutar_2d(&cero, &[]).unwrap(), vec![Evento::Alto]);
    assert_eq!(ejecutar_2d(&no_cero, &[]).unwrap(), vec![Evento::Alto]);
}

#[test]
fn rejilla_rechaza_desbordes() {
    assert!(Rejilla::de_lineas(&"x".repeat(41)).is_err());
    let ancho_ok = Rejilla::de_lineas(&"x".repeat(32)).expect("32 celdas deben caber");
    assert!(ancho_ok.celda(32, 0) == ancho_ok.celda(0, 0)); // toroidal
}
