# Guia Rapida

```powershell
cargo run -- equiv
```

**Regla de oro:** un `PASS` del simulador no significa que el PC speaker haya
sonado. Solo `baremetal/test_qemu.py` demuestra audio real, y hoy esa evidencia
cubre exclusivamente `01_escala.grid`.

## Arrancar La Escala En QEMU

```powershell
py baremetal/build.py programs/01_escala.grid
py baremetal/test_qemu.py
```

Salida real verificada:

```text
IMAGEN: C:\Development\ISyCo Git\BEATFUNGE\build\BEATFUNGE.img
GRID:   C:\Development\ISyCo Git\BEATFUNGE\programs\01_escala.grid (512 bytes, sector 6)
LAYOUT: boot sector 1; engine sectors 2-5; grid sector 6
QEMU termino con 1 (1 indica halt via isa-debug-exit)
NOTAS: [440.0, 495.0, 525.0, 585.0, 660.0, 700.0, 785.0]
PASA: grid 2D -> PIT -> PC speaker -> WAV en QEMU
```

El test espera siete notas con una tolerancia de 12 Hz. `nasm` y
`qemu-system-i386` deben estar disponibles en `PATH`; `QEMU` puede indicar una
ruta explicita al binario de QEMU.

## Verificar Los Nueve Ports

```powershell
cargo run -- equiv
```

Salida real de la corrida inicial:

```text
 1. 01_escala         EQUIVALENTE            8            8   0 (ok)
 2. 02_arpegio        EQUIVALENTE           14           14   0 (ok)
 3. 03_glissando      EQUIVALENTE          111          111   0 (ok)
 4. 04_tres_alturas   EQUIVALENTE            5            5   0 (ok)
 5. 05_motivo         EQUIVALENTE           13           13   0 (ok)
 6. 06_pulsos         EQUIVALENTE            9            9   0 (ok)
 7. 07_dos_voces      EQUIVALENTE           17           17   0 (ok)
 8. 08_drone          EQUIVALENTE           13           13   0 (ok)
 9. 09_nota_y_cuenta  EQUIVALENTE           13           13   0 (ok)

9/9 equivalentes. Control de flujo geometrico == control lineal+J/Z.
```

## Correr Tests

```powershell
cargo test
```

Salida real:

```text
running 4 tests
test la_geometria_soporta_bifurcacion_condicional ... ok
test rejilla_rechaza_desbordes ... ok
test los_nueve_ports_son_equivalentes ... ok
test ningun_programa_usa_saltos_absolutos ... ok

test result: ok. 4 passed; 0 failed
```

## Inspeccionar Una Ruta

```powershell
cargo run -- run programs/07_dos_voces.grid --verbose
```

`--verbose` imprime cada nota solicitada, divisor PIT y ticks BIOS simulados.
Salida real:

```text
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  NOTA    262 Hz  divisor= 4554  262.0 Hz real  ticks=1
  NOTA    392 Hz  divisor= 3043  392.1 Hz real  ticks=1
  ALTO
programs/07_dos_voces.grid: 17 eventos, 16 notas/silencios, 16 ticks (~0.9s), termina
```

Los archivos `.grid` se rellenan con espacios hasta 32 x 16; una fila con mas
de 32 bytes se rechaza en lugar de cortar codigo silenciosamente.

## Trampas

- `>` y `<` ya no mueven la cinta: cambian la direccion del IP.
- `{` mueve la cinta a la izquierda; `}` a la derecha.
- `J` y `Z` son invalidos. Usa `_` o `|`.
- `S` no crea polifonia: el PC speaker es monofonico. Dos voces se representan
  por una ruta que alterna notas rapidamente.
- El motor bare-metal todavia no implementa `,`, `[` ni `]`; los nueve ports
  siguen cubiertos por el simulador Rust, pero solo `01_escala.grid` tiene
  evidencia de audio QEMU.
- La duracion bare-metal usa una espera de CPU para mantener notas observables
  en QEMU; no es todavia una calibracion exacta del tempo de BEAT.
