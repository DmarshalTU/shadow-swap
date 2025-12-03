use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, Instant};

const SCREEN_WIDTH: i32 = 1200;
const SCREEN_HEIGHT: i32 = 800;
const PLAYER_SIZE: f32 = 20.0;
const SHADOW_SIZE: f32 = 18.0;
const PLAYER_SPEED: f32 = 200.0;
const INVERSE_DURATION: f32 = 5.0; // seconds
const INVERSE_COOLDOWN: f32 = 10.0; // seconds between inversions
const PORT: u16 = 5555;
const TRAP_RADIUS: f32 = 50.0;
const WIN_SCORE: i32 = 3; // First to get trapped 3 times loses

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl From<Vec2> for Vector2 {
    fn from(v: Vec2) -> Self {
        Vector2::new(v.x, v.y)
    }
}

impl From<Vector2> for Vec2 {
    fn from(v: Vector2) -> Self {
        Vec2 { x: v.x, y: v.y }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Player {
    id: u8,
    pos: Vec2,
    shadow_pos: Vec2,
    score: i32,
    is_trapped: bool,
}

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    PlayerUpdate(Player),
    InverseControl { active: bool, time_left: f32 },
    TrapEvent { player_id: u8 },
    GameReset,
}

struct GameState {
    players: [Player; 2],
    is_host: bool,
    player_id: u8,
    last_send: Instant,
    socket: Option<UdpSocket>,
    client_addr: Option<SocketAddr>,
    inverse_active: bool,
    inverse_timer: f32,
    inverse_cooldown: f32,
    trap_flash_timer: [f32; 2], // Visual feedback when trapped
    game_time: f32, // For visual effects
    // Interpolation state
    last_network_update: [Instant; 2], // Last time we received update for each player
    network_players: [Player; 2], // Networked player state
}

impl GameState {
    fn new(is_host: bool) -> Self {
        let player_id = if is_host { 0 } else { 1 };
        GameState {
            players: [
                Player {
                    id: 0,
                    pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 },
                    shadow_pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 + 100.0 },
                    score: 0,
                    is_trapped: false,
                },
                Player {
                    id: 1,
                    pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 },
                    shadow_pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 - 100.0 },
                    score: 0,
                    is_trapped: false,
                },
            ],
            is_host,
            player_id,
            last_send: Instant::now(),
            socket: None,
            client_addr: None,
            inverse_active: false,
            inverse_timer: 0.0,
            inverse_cooldown: 0.0,
            trap_flash_timer: [0.0, 0.0],
            game_time: 0.0,
            last_network_update: [Instant::now(), Instant::now()],
            network_players: [
                Player {
                    id: 0,
                    pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 },
                    shadow_pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 + 100.0 },
                    score: 0,
                    is_trapped: false,
                },
                Player {
                    id: 1,
                    pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 },
                    shadow_pos: Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 - 100.0 },
                    score: 0,
                    is_trapped: false,
                },
            ],
        }
    }

    fn connect(&mut self, addr: &str) -> Result<(), String> {
        let socket = if self.is_host {
            UdpSocket::bind(format!("0.0.0.0:{}", PORT)).map_err(|e| e.to_string())?
        } else {
            let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
            sock.connect(addr).map_err(|e| e.to_string())?;
            sock
        };
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        self.socket = Some(socket);
        Ok(())
    }

    fn send_message(&mut self, msg: Message) {
        if let Some(ref socket) = self.socket {
            if let Ok(data) = bincode::serialize(&msg) {
                if self.is_host {
                    if let Some(addr) = self.client_addr {
                        let _ = socket.send_to(&data, addr);
                    }
                } else {
                    let _ = socket.send(&data);
                }
            }
        }
    }

    fn receive_messages(&mut self) {
        let mut should_reset = false;
        if let Some(ref socket) = self.socket {
            let mut buf = [0u8; 1024];
            while let Ok((size, peer_addr)) = socket.recv_from(&mut buf) {
                if self.is_host && self.client_addr.is_none() {
                    self.client_addr = Some(peer_addr);
                    println!("Client connected from: {}", peer_addr);
                }
                
                if let Ok(msg) = bincode::deserialize::<Message>(&buf[..size]) {
                    match msg {
                        Message::PlayerUpdate(player) => {
                            // Update network state and timestamp
                            let pid = player.id as usize;
                            self.network_players[pid] = player;
                            self.last_network_update[pid] = Instant::now();
                            
                            // Immediately update for non-controlled players (smooth interpolation)
                            if pid != self.player_id as usize {
                                self.players[pid] = player;
                            }
                        }
                        Message::InverseControl { active, time_left } => {
                            self.inverse_active = active;
                            self.inverse_timer = time_left;
                        }
                        Message::TrapEvent { player_id } => {
                            let pid = player_id as usize;
                            self.players[pid].is_trapped = true;
                            self.players[pid].score += 1;
                            self.trap_flash_timer[pid] = 1.0;
                        }
                        Message::GameReset => {
                            should_reset = true;
                        }
                    }
                }
            }
        }
        if should_reset {
            self.reset_game();
        }
    }

    fn update_inverse_timer(&mut self, dt: f32) {
        if !self.is_host {
            return;
        }

        if self.inverse_active {
            self.inverse_timer -= dt;
            if self.inverse_timer <= 0.0 {
                self.inverse_active = false;
                self.inverse_cooldown = INVERSE_COOLDOWN;
                self.send_message(Message::InverseControl { active: false, time_left: 0.0 });
            }
        } else {
            self.inverse_cooldown -= dt;
            if self.inverse_cooldown <= 0.0 {
                self.inverse_active = true;
                self.inverse_timer = INVERSE_DURATION;
                self.inverse_cooldown = INVERSE_COOLDOWN;
                self.send_message(Message::InverseControl { active: true, time_left: INVERSE_DURATION });
            }
        }
    }

    fn update_player(&mut self, input: Vector2, dt: f32) {
        let other_id = (1 - self.player_id as usize) as usize;
        
        // Determine what we're controlling
        let controlling_shadow = !self.inverse_active;
        
        if controlling_shadow {
            // Control other player's shadow (client-side prediction)
            let target = &mut self.players[other_id].shadow_pos;
            target.x += input.x * PLAYER_SPEED * dt;
            target.y += input.y * PLAYER_SPEED * dt;
            
            // Keep shadow in bounds
            target.x = target.x.max(PLAYER_SIZE).min(SCREEN_WIDTH as f32 - PLAYER_SIZE);
            target.y = target.y.max(PLAYER_SIZE).min(SCREEN_HEIGHT as f32 - PLAYER_SIZE);
            
            // Also update network state for sending
            self.network_players[other_id].shadow_pos = *target;
        } else {
            // Control other player's actual character (INVERSE MODE!)
            let target = &mut self.players[other_id].pos;
            target.x += input.x * PLAYER_SPEED * dt;
            target.y += input.y * PLAYER_SPEED * dt;
            
            // Keep in bounds
            target.x = target.x.max(PLAYER_SIZE).min(SCREEN_WIDTH as f32 - PLAYER_SIZE);
            target.y = target.y.max(PLAYER_SIZE).min(SCREEN_HEIGHT as f32 - PLAYER_SIZE);
            
            // Also update network state for sending
            self.network_players[other_id].pos = *target;
        }
    }
    
    fn interpolate_players(&mut self, dt: f32) {
        // Smooth interpolation for network updates
        const INTERPOLATION_SPEED: f32 = 10.0; // How fast to catch up to network state
        
        for i in 0..2 {
            if i == self.player_id as usize {
                continue; // Don't interpolate our own character (we control it)
            }
            
            // Check if we have recent network updates
            let time_since_update = self.last_network_update[i].elapsed().as_secs_f32();
            if time_since_update > 0.1 {
                // No recent updates, use network state directly
                self.players[i] = self.network_players[i];
            } else {
                // Interpolate towards network state
                let network = &self.network_players[i];
                let current = &mut self.players[i];
                
                // Interpolate position
                let dx = network.pos.x - current.pos.x;
                let dy = network.pos.y - current.pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.1 {
                    let move_dist = INTERPOLATION_SPEED * dt;
                    if dist > move_dist {
                        current.pos.x += (dx / dist) * move_dist;
                        current.pos.y += (dy / dist) * move_dist;
                    } else {
                        current.pos = network.pos;
                    }
                }
                
                // Interpolate shadow position
                let sdx = network.shadow_pos.x - current.shadow_pos.x;
                let sdy = network.shadow_pos.y - current.shadow_pos.y;
                let sdist = (sdx * sdx + sdy * sdy).sqrt();
                if sdist > 0.1 {
                    let move_dist = INTERPOLATION_SPEED * dt;
                    if sdist > move_dist {
                        current.shadow_pos.x += (sdx / sdist) * move_dist;
                        current.shadow_pos.y += (sdy / sdist) * move_dist;
                    } else {
                        current.shadow_pos = network.shadow_pos;
                    }
                }
                
                // Sync other properties immediately
                current.score = network.score;
                current.is_trapped = network.is_trapped;
            }
        }
    }

    fn swap_with_shadow(&mut self) {
        let player = &mut self.players[self.player_id as usize];
        std::mem::swap(&mut player.pos, &mut player.shadow_pos);
        // Also update network state
        let network_player = &mut self.network_players[self.player_id as usize];
        std::mem::swap(&mut network_player.pos, &mut network_player.shadow_pos);
    }

    fn reset_game(&mut self) {
        // Reset player positions
        self.players[0].pos = Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 };
        self.players[0].shadow_pos = Vec2 { x: SCREEN_WIDTH as f32 * 0.3, y: SCREEN_HEIGHT as f32 / 2.0 + 100.0 };
        self.players[0].score = 0;
        self.players[0].is_trapped = false;
        
        self.players[1].pos = Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 };
        self.players[1].shadow_pos = Vec2 { x: SCREEN_WIDTH as f32 * 0.7, y: SCREEN_HEIGHT as f32 / 2.0 - 100.0 };
        self.players[1].score = 0;
        self.players[1].is_trapped = false;
        
        // Reset timers
        self.inverse_active = false;
        self.inverse_timer = 0.0;
        self.inverse_cooldown = 0.0;
        self.trap_flash_timer = [0.0, 0.0];
        // Note: game_time is not reset to keep visual effects smooth
    }

    fn check_traps(&mut self, dt: f32) {
        if !self.is_host {
            return;
        }

        // Update flash timers
        for i in 0..2 {
            if self.trap_flash_timer[i] > 0.0 {
                self.trap_flash_timer[i] -= dt;
            }
        }

        for i in 0..2 {
            let other_id = 1 - i;
            let player_pos = self.players[i].pos;
            let other_shadow_pos = self.players[other_id].shadow_pos;
            
            // Calculate distance once
            let dx = player_pos.x - other_shadow_pos.x;
            let dy = player_pos.y - other_shadow_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();
            
            // Check if player is near other player's shadow (trapped!)
            if dist < TRAP_RADIUS && !self.players[i].is_trapped {
                self.players[i].is_trapped = true;
                self.players[i].score += 1; // Positive score = times trapped (bad!)
                self.trap_flash_timer[i] = 1.0; // Flash for 1 second
                self.send_message(Message::TrapEvent { player_id: i as u8 });
            }
            
            // Reset trap after a moment
            if self.players[i].is_trapped && dist > TRAP_RADIUS * 2.0 {
                self.players[i].is_trapped = false;
            }
        }
    }
}

fn get_input(rl: &RaylibHandle) -> Vector2 {
    let mut input = Vector2::zero();
    
    if rl.is_key_down(KeyboardKey::KEY_D) || rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        input.x += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) || rl.is_key_down(KeyboardKey::KEY_LEFT) {
        input.x -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
        input.y -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) || rl.is_key_down(KeyboardKey::KEY_DOWN) {
        input.y += 1.0;
    }
    
    if input.length_sqr() > 0.0 {
        let len = input.length();
        Vector2::new(input.x / len, input.y / len)
    } else {
        input
    }
}

fn main() {
    println!("=== SHADOW SWAP ===");
    println!("1. Host (wait for connection)");
    println!("2. Join (connect to host)");
    print!("Choose (1/2): ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let is_host = input.trim() == "1";

    let mut game = GameState::new(is_host);

    if is_host {
        println!("\nWaiting for connection on port {}...", PORT);
        game.connect("").unwrap();
        println!("Server started! Waiting for player to connect...");
        println!("(Share your IP address with the other player)");
        std::thread::sleep(Duration::from_secs(1));
    } else {
        println!("\nEnter host IP address:");
        println!("  - For local network: Enter the host's local IP (e.g., 192.168.1.31)");
        println!("  - For same computer: Enter 127.0.0.1");
        print!("\nHost IP: ");
        let mut addr = String::new();
        std::io::stdin().read_line(&mut addr).unwrap();
        let addr = format!("{}:{}", addr.trim(), PORT);
        println!("\nConnecting to {}...", addr);
        game.connect(&addr).unwrap();
        println!("Connected! Starting game...");
    }

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Shadow Swap - Multiplayer Duel")
        .build();

    rl.set_target_fps(60);
    let mut last_frame = Instant::now();

    while !rl.window_should_close() {
        let dt = last_frame.elapsed().as_secs_f32();
        last_frame = Instant::now();

        // Network receive (do this first for lowest latency)
        game.receive_messages();

        // Update game time for visual effects
        game.game_time += dt;

        // Update inverse timer (host only)
        game.update_inverse_timer(dt);
        
        // Interpolate network updates for smooth movement
        game.interpolate_players(dt);

        // Get input
        let input = get_input(&rl);
        
        // Update player (controls other player's shadow/character)
        if input.length_sqr() > 0.0 {
            game.update_player(input, dt);
        }

        // Swap with shadow (SPACE key)
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            game.swap_with_shadow();
        }
        
        // Keep our own network state in sync
        game.network_players[game.player_id as usize] = game.players[game.player_id as usize];

        // Restart game (R key) - only when game is over
        let is_game_over = game.players[0].score >= WIN_SCORE || game.players[1].score >= WIN_SCORE;
        if rl.is_key_pressed(KeyboardKey::KEY_R) && is_game_over {
            game.reset_game();
            game.send_message(Message::GameReset);
        }

        // Check traps (host only)
        game.check_traps(dt);

        // Send updates more frequently for better sync (every 8ms = ~125fps)
        if game.last_send.elapsed().as_millis() >= 8 {
            // Always send our own player update (use network state which includes our controlled changes)
            game.send_message(Message::PlayerUpdate(game.network_players[game.player_id as usize]));
            
            // If we're controlling the opponent's shadow/character, send their update too
            let other_id = (1 - game.player_id as usize) as usize;
            game.send_message(Message::PlayerUpdate(game.network_players[other_id]));
            
            if game.is_host {
                game.send_message(Message::InverseControl { 
                    active: game.inverse_active, 
                    time_left: game.inverse_timer 
                });
            }
            game.last_send = Instant::now();
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);
        // Dark gradient background
        d.clear_background(Color::new(10, 10, 20, 255));
        
        // Draw subtle background pattern
        for y in (0..SCREEN_HEIGHT).step_by(100) {
            d.draw_line(0, y, SCREEN_WIDTH, y, Color::new(20, 20, 30, 50));
        }
        for x in (0..SCREEN_WIDTH).step_by(100) {
            d.draw_line(x, 0, x, SCREEN_HEIGHT, Color::new(20, 20, 30, 50));
        }

        // Draw center divider line
        d.draw_line(SCREEN_WIDTH / 2, 0, SCREEN_WIDTH / 2, SCREEN_HEIGHT, Color::new(100, 100, 120, 80));

        // Draw players and shadows
        for (i, player) in game.players.iter().enumerate() {
            let player_color = if i == 0 { Color::GREEN } else { Color::RED };
            let shadow_color = if i == 0 { 
                Color::new(0, 150, 0, 150) 
            } else { 
                Color::new(150, 0, 0, 150) 
            };

            let player_pos = Vector2::new(player.pos.x, player.pos.y);
            let shadow_pos = Vector2::new(player.shadow_pos.x, player.shadow_pos.y);

            // Draw shadow (semi-transparent, slightly smaller)
            d.draw_circle_v(shadow_pos, SHADOW_SIZE, shadow_color);
            d.draw_circle_lines(
                shadow_pos.x as i32,
                shadow_pos.y as i32,
                SHADOW_SIZE,
                Color::new(shadow_color.r, shadow_color.g, shadow_color.b, 200),
            );

            // Draw connection line from player to shadow (with glow effect)
            let line_color = Color::new(player_color.r, player_color.g, player_color.b, 120);
            d.draw_line_ex(player_pos, shadow_pos, 3.0, line_color);
            d.draw_line_ex(player_pos, shadow_pos, 1.5, Color::new(255, 255, 255, 80));

            // Draw player with glow effect
            let alpha = if player.is_trapped { 150 } else { 255 };
            // Outer glow
            d.draw_circle_v(player_pos, PLAYER_SIZE + 3.0, Color::new(player_color.r, player_color.g, player_color.b, alpha / 3));
            // Main circle
            d.draw_circle_v(player_pos, PLAYER_SIZE, Color::new(player_color.r, player_color.g, player_color.b, alpha));
            // Inner highlight
            d.draw_circle_v(player_pos, PLAYER_SIZE * 0.6, Color::new(255, 255, 255, alpha / 2));
            // Border
            d.draw_circle_lines(
                player_pos.x as i32,
                player_pos.y as i32,
                PLAYER_SIZE,
                Color::new(255, 255, 255, alpha),
            );

            // Draw trap radius around shadow (more visible)
            if i != game.player_id as usize {
                // Pulsing effect using game time
                let pulse = (game.game_time * 2.0).sin().abs();
                let alpha = (100.0 + pulse * 100.0) as u8;
                d.draw_circle_lines(
                    shadow_pos.x as i32,
                    shadow_pos.y as i32,
                    TRAP_RADIUS,
                    Color::new(255, 255, 0, alpha),
                );
                // Inner warning circle
                d.draw_circle_lines(
                    shadow_pos.x as i32,
                    shadow_pos.y as i32,
                    TRAP_RADIUS * 0.7,
                    Color::new(255, 200, 0, alpha / 2),
                );
            }
            
            // Flash effect when trapped
            if game.trap_flash_timer[i] > 0.0 {
                let flash_alpha = (game.trap_flash_timer[i] * 200.0) as u8;
                d.draw_circle_v(player_pos, PLAYER_SIZE + 10.0, Color::new(255, 0, 0, flash_alpha));
            }
        }

        // Draw UI with better styling - organized layout
        let player_color = if game.player_id == 0 { Color::GREEN } else { Color::RED };
        let is_game_over = game.players[0].score >= WIN_SCORE || game.players[1].score >= WIN_SCORE;
        
        // Title bar background
        d.draw_rectangle(0, 0, SCREEN_WIDTH, 140, Color::new(0, 0, 0, 200));
        
        // Game title (top center)
        d.draw_text(
            "SHADOW SWAP",
            SCREEN_WIDTH / 2 - 120,
            8,
            32,
            Color::new(200, 200, 255, 255),
        );
        
        // Left side: Player info
        d.draw_text(
            &format!("Player {} (YOU)", game.player_id + 1),
            20,
            45,
            26,
            player_color,
        );
        
        // Scores with proper spacing
        let my_score = game.players[game.player_id as usize].score;
        let other_score = game.players[1 - game.player_id as usize].score;
        d.draw_text(
            &format!("Trapped: {} / {}", my_score, WIN_SCORE),
            20,
            72,
            22,
            Color::WHITE,
        );
        d.draw_text(
            &format!("Opponent: {} / {}", other_score, WIN_SCORE),
            20,
            95,
            22,
            Color::GRAY,
        );

        // Right side: Mode indicator
        let inverse_text = if game.inverse_active {
            format!("⚡ INVERSE MODE! ⚡ ({:.1}s)", game.inverse_timer.max(0.0))
        } else {
            format!("Shadow Control ({:.1}s)", game.inverse_cooldown.max(0.0))
        };
        let inverse_color = if game.inverse_active { 
            Color::new(255, 255, 0, 255) 
        } else { 
            Color::new(200, 200, 200, 255) 
        };
        
        // Background for mode indicator
        if game.inverse_active {
            let bg_alpha = ((game.inverse_timer * 3.0).sin().abs() * 50.0 + 30.0) as u8;
            d.draw_rectangle(
                SCREEN_WIDTH - 380,
                70,
                360,
                35,
                Color::new(255, 255, 0, bg_alpha),
            );
        }
        
        // Mode text (right aligned)
        d.draw_text(
            &inverse_text,
            SCREEN_WIDTH - 370,
            75,
            24,
            inverse_color,
        );

        // Draw instructions in a panel
        let instructions_y = SCREEN_HEIGHT - 110;
        d.draw_rectangle(10, instructions_y - 10, SCREEN_WIDTH - 20, 105, Color::new(0, 0, 0, 150));
        d.draw_rectangle_lines(10, instructions_y - 10, SCREEN_WIDTH - 20, 105, Color::new(100, 100, 100, 200));
        
        d.draw_text(
            "CONTROLS:",
            20,
            instructions_y,
            20,
            Color::new(255, 255, 200, 255),
        );
        d.draw_text(
            "WASD/Arrows → Move opponent's shadow/character",
            20,
            instructions_y + 25,
            18,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            "SPACE → Swap YOUR position with YOUR shadow",
            20,
            instructions_y + 45,
            18,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            &format!("GOAL → Trap opponent {} times to win!", WIN_SCORE),
            20,
            instructions_y + 65,
            18,
            Color::YELLOW,
        );
        d.draw_text(
            "R → Restart (after game ends)",
            20,
            instructions_y + 85,
            16,
            Color::new(150, 150, 150, 255),
        );
        
        // Show restart instruction (only when game is over)
        if is_game_over {
            d.draw_text(
                "Press R to restart the game",
                SCREEN_WIDTH / 2 - 120,
                SCREEN_HEIGHT / 2 + 100,
                25,
                Color::YELLOW,
            );
        }

        // Draw win condition with better visuals
        if game.players[0].score >= WIN_SCORE {
            // Semi-transparent overlay
            d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 180));
            d.draw_text(
                "PLAYER 2 WINS!",
                SCREEN_WIDTH / 2 - 180,
                SCREEN_HEIGHT / 2 - 40,
                60,
                Color::RED,
            );
            d.draw_text(
                "Player 1 was trapped too many times!",
                SCREEN_WIDTH / 2 - 220,
                SCREEN_HEIGHT / 2 + 30,
                28,
                Color::WHITE,
            );
        } else if game.players[1].score >= WIN_SCORE {
            // Semi-transparent overlay
            d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 180));
            d.draw_text(
                "PLAYER 1 WINS!",
                SCREEN_WIDTH / 2 - 180,
                SCREEN_HEIGHT / 2 - 40,
                60,
                Color::GREEN,
            );
            d.draw_text(
                "Player 2 was trapped too many times!",
                SCREEN_WIDTH / 2 - 220,
                SCREEN_HEIGHT / 2 + 30,
                28,
                Color::WHITE,
            );
        }

        // FPS counter (top right, above instructions)
        d.draw_fps(SCREEN_WIDTH - 100, 115);
    }
}
