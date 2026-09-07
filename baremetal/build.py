"""Build a BEATfunge floppy image with one 32 x 16 program sector."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
BARE = ROOT / "baremetal"
BUILD = ROOT / "build"
GRID_SIZE = 32 * 16
FLOPPY_SIZE = 1_474_560


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
    return bytes(encoded.ljust(GRID_SIZE, b" "))


def main() -> None:
    grid = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "programs/01_escala.grid"
    output = Path(sys.argv[2]) if len(sys.argv) > 2 else BUILD / "BEATFUNGE.img"
    if not grid.is_absolute():
        grid = (ROOT / grid).resolve()
    if not output.is_absolute():
        output = (ROOT / output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)

    boot = BUILD / "boot.bin"
    engine = BUILD / "engine.bin"
    assemble(BARE / "boot.asm", boot)
    assemble(BARE / "engine.asm", engine)
    if boot.stat().st_size != 512 or engine.stat().st_size != 2048:
        raise SystemExit("layout invalido: boot=512 bytes y engine=2048 bytes requeridos")

    image = bytearray(boot.read_bytes() + engine.read_bytes() + encode_grid(grid))
    image.extend(b"\0" * (FLOPPY_SIZE - len(image)))
    output.write_bytes(image)
    print(f"IMAGEN: {output}")
    print(f"GRID:   {grid} ({GRID_SIZE} bytes, sector 6)")
    print("LAYOUT: boot sector 1; engine sectors 2-5; grid sector 6")


if __name__ == "__main__":
    main()
