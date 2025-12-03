# Shadow Swap

A unique multiplayer duel game built with Rust and Raylib where players control each other's shadows in a strategic battle of positioning and timing.

![Shadow Swap](https://img.shields.io/badge/Rust-1.91+-orange) ![Raylib](https://img.shields.io/badge/Raylib-5.5.1-blue)

## 🎮 Game Concept

**Shadow Swap** is an original multiplayer game where the core mechanic is **inverse control** - you control your opponent's shadow (or character), not your own! This creates a mind-bending strategic experience where you must think about both your position and your opponent's simultaneously.

## ✨ Features

- **Inverse Control Mechanic**: Control your opponent's shadow/character instead of your own
- **Shadow Swapping**: Instantly swap positions with your shadow to escape or reposition
- **Inverse Mode**: Periodically, control switches to directly manipulating your opponent's character
- **Strategic Trapping**: Position your shadow to trap your opponent and win
- **Real-time Multiplayer**: UDP-based networking for smooth gameplay
- **Polished UI/UX**: Clean interface with visual feedback and animations

## 🎯 How to Play

### Objective
Trap your opponent by getting them near your shadow **3 times** to win!

### Controls
- **WASD** or **Arrow Keys**: Move your opponent's shadow (or character in Inverse Mode)
- **SPACE**: Swap your character with your shadow
- **R**: Restart the game (after someone wins)

### Game Mechanics

1. **Normal Mode**: You control your opponent's shadow. Move it around to set up traps or force them into bad positions.

2. **Inverse Mode**: Every ~10 seconds, you gain direct control of your opponent's character for 5 seconds. Use this to push them into your shadow!

3. **Shadow Swap**: Press SPACE to instantly swap your character with your shadow. Use this strategically to:
   - Escape from danger
   - Reposition your shadow near the opponent
   - Create unexpected trap setups

4. **Trapping**: Your shadow has a yellow trap radius. If your opponent's character enters this radius, they get trapped and you score a point.

## 🚀 Getting Started

### Option 1: Download Pre-built Executables (Recommended)

Download the latest release from the [Releases page](https://github.com/DmarshalTU/shadow-swap/releases):

- **Windows**: `shadow-swap-windows-x86_64.tar.gz`
- **Linux**: `shadow-swap-linux-x86_64.tar.gz`
- **macOS (Intel)**: `shadow-swap-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `shadow-swap-macos-arm64.tar.gz`

Extract the archive and run the executable:
```bash
# Linux/macOS
tar -xzf shadow-swap-*.tar.gz
./shadow-swap

# Windows
# Extract the .tar.gz file (use 7-Zip or similar)
# Run shadow-swap.exe
```

### Option 2: Build from Source

#### Prerequisites

- Rust 1.91.0 or later
- Cargo (comes with Rust)
- System dependencies (see below)

#### Installation

1. Clone the repository:
```bash
git clone https://github.com/DmarshalTU/shadow-swap.git
cd shadow-swap
```

2. Install system dependencies:

**Linux:**
```bash
sudo apt-get install libasound2-dev libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libglu1-mesa-dev libxcursor-dev libxinerama-dev
```

**macOS:**
```bash
# Dependencies are usually pre-installed
```

**Windows:**
```bash
# Dependencies are usually pre-installed
```

3. Build the project:
```bash
cargo build --release
```

4. Run the game:
```bash
# Linux/macOS
./target/release/rayq

# Windows
target\release\rayq.exe
```

### Running a Multiplayer Game

1. **Host Setup**:
   - Run the game
   - Choose option `1` (Host)
   - Wait for connection on port 5555

2. **Client Setup**:
   - Run the game on another machine
   - Choose option `2` (Join)
   - Enter the host's IP address (e.g., `127.0.0.1` for localhost, or the host's local IP for LAN)

3. **Play!**
   - Both players will see the game screen
   - Use your controls to manipulate the opponent
   - First to trap the opponent 3 times wins!

## 🛠️ Technical Details

### Architecture
- **Language**: Rust
- **Graphics**: Raylib 5.5.1
- **Networking**: UDP sockets with custom protocol
- **Serialization**: Bincode for efficient message encoding

### Network Protocol
- Host-client architecture
- Host manages game state and physics
- Clients receive state updates and send input
- Messages: Player updates, ball updates, score updates, game reset

### Performance
- 60 FPS target
- Efficient UDP networking (~60 updates/second)
- Minimal latency for responsive gameplay

## 📝 License

This project is open source. Feel free to use, modify, and distribute as you wish!

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Improve documentation

## 🎨 Game Design

### Why "Shadow Swap"?
The name reflects the two core mechanics:
- **Shadow**: You control shadows, not direct characters
- **Swap**: The ability to swap with your shadow is crucial for strategy

### Unique Selling Points
1. **Inverse Control**: Unlike traditional games, you control the opponent
2. **Dual Mechanics**: Shadow control + swap creates deep strategy
3. **Inverse Mode**: Periodic power-up adds chaos and excitement
4. **Simple to Learn, Hard to Master**: Easy controls, complex strategy

## 🐛 Known Issues

- Network latency can cause slight desync (working on improvements)
- No matchmaking system (manual IP entry required)

## 🔮 Future Ideas

- [ ] Matchmaking server
- [ ] Different game modes
- [ ] Power-ups and obstacles
- [ ] Sound effects and music
- [ ] Replay system
- [ ] Tournament mode

## 📧 Contact

Have questions or suggestions? Open an issue or reach out!

---

**Enjoy the game and may the best shadow manipulator win!** 🎮✨

