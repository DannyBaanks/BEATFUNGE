# BEATfunge

**BEAT con el flujo de control geometrico de Befunge, en un sector fisico de
32 x 16 celdas.**

El proyecto arranca como un simulador de referencia en Rust. Conserva la
semantica musical de [BEAT](https://github.com/DannyBaanks/BEAT) y sustituye
los saltos absolutos `J`/`Z` por una rejilla toroidal. No es todavia un motor
bare-metal ni ha sido arrancado en QEMU: eso sigue `NOT_DEMONSTRATED`.

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

## Ejecutar

```powershell
cargo test
cargo run -- equiv
cargo run -- run programs/07_dos_voces.grid --verbose
```

## Estado

- Simulador Rust: `PASS`
- 9/9 ports canonicos: `PASS`
- Saltos absolutos en ports 2D: `0`
- Motor bare-metal / sector arrancable / QEMU: `NOT_DEMONSTRATED`

## Licencia

MIT.
