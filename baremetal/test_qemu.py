"""Boot the canonical scale and verify its PC-speaker frequencies in QEMU."""

import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import wave

ROOT = Path(__file__).resolve().parent.parent
IMAGE = ROOT / "build" / "BEATFUNGE.img"
EXPECTED = [440.1, 494.1, 523.1, 587.2, 659.2, 698.2, 784.5]
TOLERANCE_HZ = 12.0


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


def main() -> int:
    image = Path(sys.argv[1]) if len(sys.argv) > 1 else IMAGE
    wav = ROOT / "build" / "qemu_scale.wav"
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
        print(f"QEMU termino con {result.returncode} (1 indica halt via isa-debug-exit)")
        if result.stderr:
            print(result.stderr.decode(errors="replace"))
    except subprocess.TimeoutExpired:
        pass  # The guest correctly remains halted after H.

    if not wav.exists():
        print("FALLO: QEMU no produjo audio")
        return 1
    notes = measured_notes(*read_samples(repair_wav(wav)))
    print(f"NOTAS: {[round(note, 1) for note in notes]}")
    if len(notes) != len(EXPECTED):
        print(f"FALLO: detectadas {len(notes)} notas; esperadas {len(EXPECTED)}")
        return 1
    errors = [abs(actual - expected) for actual, expected in zip(notes, EXPECTED)]
    if any(error > TOLERANCE_HZ for error in errors):
        print(f"FALLO: errores Hz {[round(error, 1) for error in errors]}")
        return 1
    print("PASA: grid 2D -> PIT -> PC speaker -> WAV en QEMU")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
