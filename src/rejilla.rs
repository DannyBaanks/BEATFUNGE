//! La rejilla BEATfunge: un sector de disquete = 32 x 16 = 512 celdas, toroidal.

pub const ANCHO: usize = 32;
pub const ALTO: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejilla {
    pub celdas: [[u8; ANCHO]; ALTO],
}

impl Rejilla {
    /// Construye desde un texto (una linea por fila). El resto se rellena
    /// con espacios (NOP). Error si una linea excede 32 columnas o si hay
    /// mas de 16 lineas.
    pub fn de_lineas(texto: &str) -> Result<Self, String> {
        let mut celdas = [[b' '; ANCHO]; ALTO];
        let lineas: Vec<&str> = texto.lines().collect();
        if lineas.len() > ALTO {
            return Err(format!(
                "la rejilla tiene {ALTO} filas; el texto trae {}",
                lineas.len()
            ));
        }
        for (y, lin) in lineas.iter().enumerate() {
            let bytes = lin.as_bytes();
            if bytes.len() > ANCHO {
                return Err(format!(
                    "fila {y} excede {ANCHO} columnas ({} bytes)",
                    bytes.len()
                ));
            }
            celdas[y][..bytes.len()].copy_from_slice(bytes);
        }
        Ok(Rejilla { celdas })
    }

    pub fn celda(&self, x: usize, y: usize) -> u8 {
        self.celdas[y % ALTO][x % ANCHO]
    }

    /// Cualquier byte J/Z en un programa BEATfunge viola el contrato:
    /// el lenguaje no tiene saltos absolutos.
    pub fn contiene_saltos_absolutos(&self) -> bool {
        self.celdas
            .iter()
            .flatten()
            .any(|&c| c == b'J' || c == b'Z')
    }
}
