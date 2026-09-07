# Guia Rapida

```powershell
cargo run -- equiv
```

**Regla de oro:** un `PASS` del simulador no significa que el PC speaker haya
sonado. Hasta que exista un motor arrancable y una corrida QEMU, el audio real
es `NOT_DEMONSTRATED`.

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
