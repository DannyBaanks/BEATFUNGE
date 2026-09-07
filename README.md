# BEATfunge

**BEAT con el flujo de control geometrico de Befunge, en un sector fisico de
32 x 16 celdas.**

El proyecto conserva la semantica musical de
[BEAT](https://github.com/DannyBaanks/BEAT) y sustituye los saltos absolutos
`J`/`Z` por una rejilla toroidal. Incluye un simulador de referencia en Rust y
un nucleo NASM arrancable que ya demostro la escala 2D en QEMU.

## El Hallazgo

BEAT ejecuta un programa lineal desde el sector 4 y sus `J`/`Z` cargan
direcciones absolutas `0x06xx`; por tanto esos programas no son reubicables.
BEATfunge hace que el sector completo sea su playfield:

```text
1 sector = 512 bytes = 32 columnas x 16 filas
```

El puntero de instruccion se mueve por el plano. No necesita almacenar una
direccion absoluta para volver a un motivo o decidir una salida.

## Instrucciones

```text
> < ^ v   fijan la direccion del IP
_         reg == 0: derecha; reg != 0: izquierda
|         reg == 0: abajo;   reg != 0: arriba
#         puente: salta una celda

{ }       cinta: izquierda / derecha
+ -       registro u16 con wraparound
. ,       tocar registro / leer nota de teclado
[ ]       lazo provisional: busca pareja en linea recta toroidal
T<n>      tempo, indice 0..9 (60..240 BPM)
D<n>      duracion, semicorcheas 0..9
N<hz>     frecuencia decimal en la direccion de avance
P         pausa
R W       cinta -> registro / byte bajo del registro -> cinta
S         alterna altavoz
H / @     alto
```

La colision fundamental se resuelve a favor de la geometria: en BEAT `>` y
`<` movian la cinta; en BEATfunge son direccion del IP y `{`/`}` mueven la
cinta. `J` y `Z` no existen y el simulador rechaza cualquier rejilla que los
contenga.

## Programas Canonicos

`programs/` contiene ports de los nueve programas demostrados de BEAT:

| Programa | Lo que prueba |
|---|---|
| 01 escala | serpiente de dos filas para siete notas |
| 02 arpegio | notas y pausas en ida/vuelta |
| 03 glissando | `[`/`]` compatibles en flujo recto |
| 04 tres alturas | cinta en una ruta vertical |
| 05 motivo | contador en cinta; `_`/`|` sustituyen `Z`/`J` |
| 06 pulsos | acumulador de altura y ruta de retorno |
| 07 dos voces | el IP es scheduler espacial de multiplexado monofonico |
| 08 drone | toggle `S` y pausas |
| 09 nota y cuenta | una celda usada como altura y contador |

La prueba compara cada port contra un interprete 1D que reproduce
`BEAT.asm`, byte por byte, incluidos sus operandos absolutos originales.

```text
9/9 equivalentes. Control de flujo geometrico == control lineal+J/Z.
```

Esto significa solo que los nueve programas producen la misma secuencia de
notas, silencios, pausas, toggles y alto bajo el simulador. No demuestra salida
de audio real ni equivalencia universal de los lenguajes.

## Bare Metal

`baremetal/boot.asm` carga un motor NASM de cuatro sectores y el grid como el
sector 6. El programa conserva su forma de 32 x 16 y no contiene direcciones
absolutas.

```text
sector 1    bootloader
sectores 2-5  motor BEATfunge
sector 6    grid 32 x 16
```

El motor implementa direcciones, bifurcaciones `_`/`|`, puente `#`, `T`, `D`,
`N`, `.`, `P`, `+`, `-`, `{`, `}`, `R`, `W`, `S`, `H` y `@`. Usa una espera por
CPU para las duraciones: el test de BEAT/Kaleidoscope encontro que el sondeo de
ticks BIOS mediante `INT 1Ah` podia colapsar duraciones en esta configuracion
de captura QEMU. La calibracion exacta de tempo BEAT sigue
`NOT_DEMONSTRATED`.

La primera evidencia independiente es `01_escala.grid`: QEMU arranco la imagen
y su WAV del PC speaker midio `440, 495, 525, 585, 660, 700, 785 Hz`, dentro de
12 Hz de las siete notas esperadas. Los otros ocho grids tambien producen
WAV validos por QEMU (ver `test_all_grids.py`). Los opcodes `,`, `[` y
`]` en bare-metal siguen `NOT_DEMONSTRATED`.

## Ejecutar

```powershell
cargo test
cargo run -- equiv
cargo run -- run programs/07_dos_voces.grid --verbose
py baremetal/build.py programs/01_escala.grid
py baremetal/test_qemu.py
```

## Estado

- Simulador Rust: `PASS`
- 9/9 ports canonicos: `PASS`
- Saltos absolutos en ports 2D: `0`
- Bare-metal/QEMU, `01_escala.grid`: `PASS` (precision)
- Bare-metal/QEMU, `08_drone.grid`: `PASS` (precision)
- Bare-metal/QEMU, otros 7 grids: `PASS` (empirico — WAV producido)
- `,`, `[` y `]` en bare-metal: `NOT_DEMONSTRATED`

## Licencia

MIT.
