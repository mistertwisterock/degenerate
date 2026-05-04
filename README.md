degenerate
==========

A playground for math-based generative image and animation creation using arithmetic primitives, epicycles, spirals, procedural noise, and audio analysis.

## Overview

**degenerate** is a Rust-based tool for creating abstract generative art. It generates images and animations by:
- Applying 19 different mathematical equations to create visual patterns
- Analyzing audio files (WAV) using FFT and RMS for audio-reactive visuals
- Using procedural noise functions (OpenSimplex, HybridMulti, Billow)
- Rendering with Cairo for high-quality vector graphics output
- Supporting displacement mapping from input images

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd degenerate

# Build (requires Rust 1.80+ and system dependencies)
sudo apt-get install libglib2.0-dev libcairo2-dev pkg-config
cargo build --release

# Run
cargo run --release -- [OPTIONS] [SOUNDFILE]
```

## Usage

### Basic Examples

```bash
# Generate a single 4000x4000 image with default settings
cargo run --release

# Create a custom-sized image with 5000 iterations
cargo run --release -- --width 1920 --height 1080 -i 5000

# Generate with specific equations (f1=10, f2=5)
cargo run --release -- --width 800 --height 600 --f1 10 --f2 5 -i 2000

# Use different rendering methods
cargo run --release -- -M line -i 1000      # Lines between points
cargo run --release -- -M curve -i 1000     # Bezier curves
cargo run --release -- -M arc -i 1000       # Circular arcs
cargo run --release -- -M dot -s 2.0 -i 1000  # Dots with size 2.0

# Set custom output location
cargo run --release -- -o ./output --filename artwork_ -i 3000
```

### Audio-Reactive Animation

```bash
# Generate 100 frames synchronized to audio.wav at 25 fps
cargo run --release -- audio.wav --fps 25 -f 100 -o ./frames

# Create full animation from audio file
cargo run --release -- music.wav --width 1920 --height 1080 -M curve

# Start from frame 50 and generate next 100 frames
cargo run --release -- audio.wav --start 50 -f 100
```

### Displacement Mapping

```bash
# Use image.png as displacement source
cargo run --release -- --image input.png --scale-image 1.5 -f 120

# Combine with audio
cargo run --release -- audio.wav --image logo.png -f 200
```

## Parameters

### Positional Arguments

| Argument | Description | Default |
|----------|-------------|--------|
| `SOUNDFILE` | Path to WAV audio file for audio-reactive generation | `""` (no audio) |

### Output Options

| Flag | Description | Default |
|------|-------------|--------|
| `--width <WIDTH>` | Image width in pixels | `4000` |
| `--height <HEIGHT>` | Image height in pixels | `4000` |
| `-o, --outdir <OUTDIR>` | Output directory for generated frames | `/tmp` |
| `--filename <FILENAME>` | Base filename for output (frame number appended) | `frame_` |

### Generation Parameters

| Flag | Description | Default |
|------|-------------|--------|
| `-i, --iterations <ITERATIONS>` | Number of point pairs to generate per frame (0=auto) | `0` |
| `-M, --method <METHOD>` | Rendering method: `dot`, `line`, `curve`, `arc` | `dot` |
| `-s, --size <SIZE>` | Point size multiplier (for dot method, 0=auto) | `0` |
| `--combine-dots` | Combine point pairs into single dots | `false` |
| `-r, --radius <RADIUS>` | Base radius for pattern generation (0=auto: width/2) | `0` |
| `-e, --expansion <EXPANSION>` | Radius expansion factor per frame | `1.0` |
| `-g, --grow <GROW>` | Growth parameter (deprecated) | `0` |
| `-t <T>` | Time scale multiplier | `1.0` |
| `-m <M>` | Modulation parameter for exponential transfer | `0.2` |

### Equation Selection

| Flag | Description | Default |
|------|-------------|--------|
| `--f1 <F1>` | Force equation index (0-18) for first point (0=auto) | `0` |
| `--f2 <F2>` | Force equation index (0-18) for second point (0=auto) | `0` |

**Available equations (0-18):**
- `0`: Basic sine/cosine circle
- `1`: Golden ratio spiral
- `2`: Logarithmic spiral
- `3`: Noise-modulated harmonics
- `4`: Billow noise with logistic map
- `5`: Multi-layered noise combination
- `6`: Fractal iterations with fallback
- `7`: Hyperbolic transformations
- `8`: FFT-reactive displacement
- `9`: Complex power functions
- `10`: "Totenschiff" - layered trigonometric
- `11`: FFT frequency visualization
- `12`: Logarithmic displacement
- `13`: Arc tangent fractals
- `14`: Pure noise composition
- `15`: Popcorn attractor
- `16`: Quantum-inspired equation
- `17`: Cubic root transformation
- `18`: Radial distortion

### Animation Options

| Flag | Description | Default |
|------|-------------|--------|
| `-f, --frames <FRAMES>` | Number of frames to generate (0=auto from audio) | `0` |
| `--fps <FPS>` | Frames per second for audio synchronization | `25` |
| `--start <START>` | Starting frame number | `0` |

### Image Displacement

| Flag | Description | Default |
|------|-------------|--------|
| `--image <IMAGE>` | Path to image file for displacement mapping | `""` (none) |
| `--scale-image <SCALE_IMAGE>` | Scale factor for input image | `1` |

### Debugging

| Flag | Description | Default |
|------|-------------|--------|
| `-d, --debug` | Enable debug output (prints point coordinates) | `false` |
| `-h, --help` | Print help information | |
| `-V, --version` | Print version | |

## Codebase Structure

```
degenerate/
├── src/
│   ├── main.rs         # Entry point, frame generation, Cairo rendering
│   ├── lib.rs          # Utilities: FFT, normalization, RMS, audio loading
│   ├── args.rs         # CLI argument parsing with clap
│   ├── ghostweb.rs     # Core: 19 mathematical equations, state machine
│   ├── feed.rs         # Data structures: Point, Feed
│   └── render.rs       # Render configuration structure
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

### Core Modules

#### `main.rs`
- Multi-frame animation loop with progress bar
- Cairo context setup and rendering
- Displacement mapping between generated and image points
- Draw functions for different rendering methods (dot, line, curve, arc)

#### `ghostweb.rs`
- State machine for iterative point generation
- 19 mathematical equation functions (`equation_000` to `equation_018`)
- Noise generators: OpenSimplex, HybridMulti, Billow
- Uses constants: π, e, φ (golden ratio), √2
- Automatic equation selection based on audio samples or FFT bins
- Image-to-points conversion for displacement mapping

#### `lib.rs`
- **FFT**: Fast Fourier Transform using rustfft
- **RMS**: Root Mean Square calculation for audio energy
- **Normalization**: Sample normalization to -1..1 range
- **Audio loading**: WAV file parsing with hound
- **Frame saving**: PNG export via Cairo

#### `args.rs`
- CLI argument definitions using clap derive macros
- Method enum: Arc, Curve, Dot, Line
- Custom parser for method selection

#### `feed.rs`
- `Point`: 3D coordinate (x, y, z)
- `Feed`: Pair of points (p1, p2) with radius

#### `render.rs`
- `RenderConfig`: Encapsulates rendering parameters
- Constructed from Args for each frame

### Dependencies

- **cairo-rs** (0.15.11): Vector graphics rendering and PNG export
- **rustfft** (6.0.1): Fast Fourier Transform for audio analysis
- **noise** (0.7.0): Procedural noise functions
- **hound** (3.4.0): WAV audio file reading
- **image** (0.24.2): Image loading and processing
- **clap** (4.5.4): Command-line argument parsing
- **rand** (0.8.3): Random number generation
- **pbr** (1.0.4): Progress bar display
- **png** (0.17.5): PNG format support

## How It Works

1. **Initialization**: Parse arguments and load optional audio/image files
2. **Audio Processing**: If provided, WAV file is split into blocks matching FPS
3. **Frame Loop**: For each frame:
   - Extract audio block and compute FFT/RMS
   - Initialize state machine with iteration counter
   - For each iteration:
     - Advance state (update time, counters, noise state)
     - Select equations (automatic or forced via --f1/--f2)
     - Compute p1 using equation_f1(state, params, prev_p1, prev_p2)
     - Compute p2 using equation_f2(state, params, prev_p2, prev_p1)
     - Store Feed{p1, p2, radius}
   - Optionally apply displacement from image points
   - Render to Cairo surface using selected method
   - Save PNG frame
4. **Output**: Numbered PNG files in output directory

## Mathematical Concepts

- **Epicycles**: Circular motion combinations create complex paths
- **Spirals**: Logarithmic and Archimedean spirals from polar equations
- **Attractors**: Strange attractors like Popcorn create chaotic patterns
- **Noise Fields**: Perlin/Simplex noise adds organic variation
- **FFT Mapping**: Audio frequencies map to visual parameters
- **Logistic Map**: Chaotic dynamics via xₙ₊₁ = r·xₙ·(1-xₙ)
- **Golden Ratio (φ)**: φ = 1.618... creates aesthetically pleasing proportions

## Examples of Output

### Single Frame (Default)
```bash
cargo run --release -- --width 800 --height 600 -i 1000
# → /tmp/frame_000000.png (abstract dot pattern)
```

### Animation Sequence
```bash
cargo run --release -- audio.wav -f 300 --fps 30 -o ./animation
# → ./animation/frame_000000.png ... frame_000299.png
# Convert to video:
# ffmpeg -r 30 -i ./animation/frame_%06d.png -c:v libx264 output.mp4
```

### High-Resolution Artwork
```bash
cargo run --release -- --width 8000 --height 8000 -i 50000 --f1 10 --f2 6 -M curve -r 3000
# → /tmp/frame_000000.png (detailed generative art piece)
```

## Tips

- **Iterations**: More iterations = denser patterns (try 10000+ for detailed work)
- **Equations**: Experiment with different f1/f2 combinations (each has unique character)
- **Audio**: Use music with strong beats for dramatic visual changes
- **Methods**: `curve` creates flowing organic forms, `line` creates geometric webs
- **Radius**: Larger radius spreads pattern across more of the canvas
- **Time scale (-t)**: Values > 1 speed up animation, < 1 slow it down

## License

See repository for license information.

## Author

Ulrich Berthold <u@sansculotte.net>
