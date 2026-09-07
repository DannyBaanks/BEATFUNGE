//! Los nueve programas canonicos de BEAT, byte a byte desde BEAT.asm.
//! Son el oraculo: BEATfunge debe producir la misma secuencia de eventos.
//! Los \x06/\x07/\x0D/\x1E/\x30 son los operandos de salto absoluto J/Z
//! (little-endian, direcciones 0x06xx del sector 4).

pub const PROG1: &[u8] = b"T4D9N440.N494.N523.N587.N659.N698.N784.H\x00";

pub const PROG2: &[u8] = b"T3D4N262.PN330.PN392.PN523.PN392.PN330.PN262.H\x00";

pub const PROG3: &[u8] = b"T9D0N440[.----]H\x00";

pub const PROG4: &[u8] = b"T3D5N200W>N150W>N120W<<R.>R.>R.<<R.H\x00";

pub const PROG5: &[u8] = b"T5D3N4WRZ \x06N330.N392.N523.R-WJ\x07\x06H\x00";

pub const PROG6: &[u8] = b"T7D2N8W>N60W<RZ0\x06>R++++++++++++++++++++W.<R-WJ\x0D\x06H\x00";

pub const PROG7: &[u8] =
    b"T9D0N262.N392.N262.N392.N262.N392.N262.N392.N262.N392.N262.N392.N262.N392.N262.N392.H\x00";

pub const PROG8: &[u8] = b"T5D4N165.SPPPPPPPPSN330.H\x00";

pub const PROG9: &[u8] = b"T4D4N120WRZ\x1E\x06R.R----------WJ\x09\x06H\x00";

pub const ORIGINALES: [&[u8]; 9] = [
    PROG1, PROG2, PROG3, PROG4, PROG5, PROG6, PROG7, PROG8, PROG9,
];

pub const NOMBRES: [&str; 9] = [
    "01_escala",
    "02_arpegio",
    "03_glissando",
    "04_tres_alturas",
    "05_motivo",
    "06_pulsos",
    "07_dos_voces",
    "08_drone",
    "09_nota_y_cuenta",
];
