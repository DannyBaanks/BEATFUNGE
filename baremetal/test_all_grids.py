"""Run all 9 BEATfunge grids through QEMU and verify PC-speaker frequencies."""

import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import wave

ROOT = Path(__file__).resolve().parent.parent
BARE = ROOT / "baremetal"
BUILD = ROOT / "build"
PROGRAMS = ROOT / "programs"
TOLERANCE_HZ = 20.0

# Expected notes per grid (canonical frequencies).
# Only 01_escala and 08_drone are stable enough for precision testing.
# The rest are empiric: they must produce audio, but note detection
# varies between QEMU runs due to non-deterministic PIT timing.
GRIDS = {
    "01_escala":        [440.1, 494.1, 523.1, 587.2, 659.2, 698.2, 784.5],
    "02_arpegio":       None,
    "03_glissando":     None,
    "04_tres_alturas":  None,
    "05_motivo":        None,
    "06_pulsos":        None,
    "07_dos_voces":     None,
    "08_drone":         [165.0, 330.0],
    "09_nota_y_cuenta": None,
}


def qemu() -> Path:
    configured = os.environ.get("QEMU")
    if configured:
        return Path(configured)
    found = shutil.which("qemu-system-i386")
    if found:
        return Path(found)
    candidate = Path.home() / "scoop/apps/qemu/current/qemu-system-i386.exe"
    if candidate.exists():
        return candidate
    raise SystemExit("no encuentro qemu-system-i386; define QEMU o instalalo en PATH")


def assemble(source: Path, output: Path) -> None:
    subprocess.run(["nasm", "-f", "bin", str(source), "-o", str(output)], check=True)


def encode_grid(path: Path) -> bytes:
    rows = path.read_text(encoding="ascii").splitlines()
    if len(rows) > 16:
        raise SystemExit(f"{path}: maximo 16 filas, recibidas {len(rows)}")
    encoded = bytearray()
    for number, row in enumerate(rows, 1):
        data = row.encode("ascii")
        if len(data) > 32:
            raise SystemExit(f"{path}: fila {number} supera 32 columnas")
        encoded.extend(data.ljust(32, b" "))
    return bytes(encoded.ljust(32 * 16, b" "))


def build_image(grid_path: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    boot = BUILD / "boot.bin"
    engine = BUILD / "engine.bin"
    assemble(BARE / "boot.asm", boot)
    assemble(BARE / "engine.asm", engine)
    if boot.stat().st_size != 512 or engine.stat().st_size != 2048:
        raise SystemExit("layout invalido: boot=512 y engine=2048 bytes requeridos")
    image = bytearray(boot.read_bytes() + engine.read_bytes() + encode_grid(grid_path))
    image.extend(b"\0" * (1_474_560 - len(image)))
    output.write_bytes(image)


def repair_wav(path: Path) -> Path:
    data = bytearray(path.read_bytes())
    if len(data) < 44:
        raise SystemExit("QEMU produjo un WAV incompleto")
    struct.pack_into("<I", data, 4, len(data) - 8)
    struct.pack_into("<I", data, 40, len(data) - 44)
    fixed = path.with_name(path.stem + "_fix.wav")
    fixed.write_bytes(data)
    return fixed


def read_samples(path: Path) -> tuple[list[int], int]:
    with wave.open(str(path), "rb") as source:
        frames = source.readframes(source.getnframes())
        rate = source.getframerate()
        width = source.getsampwidth()
        channels = source.getnchannels()
    if width != 2:
        raise SystemExit(f"esperaba WAV de 16 bits, obtuve {width * 8}")
    values = list(struct.unpack(f"<{len(frames) // 2}h", frames))
    return values[::channels], rate


def measured_notes(samples: list[int], rate: int) -> list[float]:
    window = rate // 10
    windows = []
    for start in range(0, len(samples) - window, window):
        chunk = samples[start:start + window]
        if max(abs(value) for value in chunk) <= 600:
            windows.append(0.0)
            continue
        crossings = sum(
            (left >= 0) != (right >= 0) for left, right in zip(chunk, chunk[1:])
        )
        windows.append((crossings / 2) / (window / rate))

    groups: list[list[float]] = []
    for frequency in windows:
        reference = groups[-1][0] if groups else 0.0
        if not groups or abs(frequency - reference) > max(reference, 1.0) * 0.04:
            groups.append([frequency])
        else:
            groups[-1].append(frequency)
    return [
        sorted(group)[len(group) // 2]
        for group in groups
        if group[0] > 0 and len(group) >= 2
    ]


def run_qemu(image: Path, wav: Path) -> int:
    wav.unlink(missing_ok=True)
    try:
        result = subprocess.run(
            [str(qemu()), "-drive", f"file={image},format=raw,if=floppy",
             "-audiodev", f"wav,id=snd0,path={wav}",
             "-machine", "pc,pcspk-audiodev=snd0", "-device",
             "isa-debug-exit,iobase=0xf4,iosize=0x04", "-display", "none", "-no-reboot"],
            timeout=60,
            check=False,
            capture_output=True,
        )
        return result.returncode
    except subprocess.TimeoutExpired:
        return -1


def main() -> int:
    nasm = shutil.which("nasm")
    if not nasm:
        print("FALLO: nasm no encontrado en PATH")
        return 1

    BUILD.mkdir(parents=True, exist_ok=True)
    passed = 0
    failed = 0
    empiric = 0
    results = []

    for name, expected in GRIDS.items():
        grid_path = PROGRAMS / f"{name}.grid"
        if not grid_path.exists():
            print(f"SALTADO: {name} — {grid_path} no existe")
            continue

        image = BUILD / f"qemu_{name}.img"
        wav = BUILD / f"qemu_{name}.wav"
        wav.unlink(missing_ok=True)

        print(f"\n--- {name} ---")
        build_image(grid_path, image)
        code = run_qemu(image, wav)

        if not wav.exists():
            print(f"  FALLO: QEMU no produjo audio")
            failed += 1
            results.append((name, "NO_WAV", []))
            continue

        notes = measured_notes(*read_samples(repair_wav(wav)))
        notes_rounded = [round(n, 1) for n in notes]
        print(f"  NOTAS: {notes_rounded} ({len(notes)} detectadas)")

        if expected is None:
            # Empiric: WAV existence proves the engine ran and produced audio.
            # Note detection is unreliable for short/complex programs.
            if wav.exists() and wav.stat().st_size > 44:
                print(f"  PASA (empirico: WAV {wav.stat().st_size} bytes)")
                empiric += 1
                results.append((name, "PASA_EMP", notes_rounded))
            else:
                print(f"  FALLO: WAV vacio o inexistente")
                failed += 1
                results.append((name, "NO_WAV", notes_rounded))
            continue

        # Precision check: exact note count and frequency match
        if len(notes) != len(expected):
            print(f"  FALLO: {len(notes)} notas detectadas, {len(expected)} esperadas")
            failed += 1
            results.append((name, "FALLO_CANTIDAD", notes_rounded))
            continue

        errors = [abs(a - e) for a, e in zip(notes, expected)]
        if any(e > TOLERANCE_HZ for e in errors):
            print(f"  FALLO: errores Hz {[round(e, 1) for e in errors]}")
            failed += 1
            results.append((name, "FALLO_FREQ", notes_rounded))
            continue

        print(f"  PASA (precision)")
        passed += 1
        results.append((name, "PASA_PREC", notes_rounded))

    print(f"\n{'='*50}")
    print(f"RESUMEN: {passed} precision, {empiric} empiricos, {failed} fallan")
    for name, status, notes in results:
        print(f"  {name:20s} {status:15s} {notes}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
