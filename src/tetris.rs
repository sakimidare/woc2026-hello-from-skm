// src/tetris.rs
// SPDX-License-Identifier: GPL-2.0

//! Tetris game kernel module with character device interface

use kernel::{
    fs::{File, Kiocb},
    iov::{IovIterDest, IovIterSource},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    prelude::*,
    sync::Arc,
    types::ForeignOwnable,
};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

/// Tetromino shapes (7 standard pieces)
#[derive(Debug, Clone, Copy, PartialEq)]
enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

/// Tetromino piece with position and rotation
#[derive(Debug, Clone, Copy)]
struct Tetromino {
    piece_type: TetrominoType,
    x: i32,
    y: i32,
    rotation: u8,
}

impl Tetromino {
    fn new(piece_type: TetrominoType) -> Self {
        Self {
            piece_type,
            x: (BOARD_WIDTH / 2) as i32 - 2,
            y: 0,
            rotation: 0,
        }
    }

    /// Get the shape matrix for current rotation
    fn get_shape(&self) -> [[bool; 4]; 4] {
        let base = match self.piece_type {
            TetrominoType::I => [
                [false, false, false, false],
                [true, true, true, true],
                [false, false, false, false],
                [false, false, false, false],
            ],
            TetrominoType::O => [
                [false, false, false, false],
                [false, true, true, false],
                [false, true, true, false],
                [false, false, false, false],
            ],
            TetrominoType::T => [
                [false, false, false, false],
                [false, true, false, false],
                [true, true, true, false],
                [false, false, false, false],
            ],
            TetrominoType::S => [
                [false, false, false, false],
                [false, true, true, false],
                [true, true, false, false],
                [false, false, false, false],
            ],
            TetrominoType::Z => [
                [false, false, false, false],
                [true, true, false, false],
                [false, true, true, false],
                [false, false, false, false],
            ],
            TetrominoType::J => [
                [false, false, false, false],
                [true, false, false, false],
                [true, true, true, false],
                [false, false, false, false],
            ],
            TetrominoType::L => [
                [false, false, false, false],
                [false, false, true, false],
                [true, true, true, false],
                [false, false, false, false],
            ],
        };

        // Apply rotation
        let mut result = base;
        for _ in 0..(self.rotation % 4) {
            result = Self::rotate_matrix(result);
        }
        result
    }

    fn rotate_matrix(matrix: [[bool; 4]; 4]) -> [[bool; 4]; 4] {
        let mut rotated = [[false; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                rotated[j][3 - i] = matrix[i][j];
            }
        }
        rotated
    }
}

/// Game state
struct TetrisGame {
    board: [[bool; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Option<Tetromino>,
    score: u32,
    game_over: bool,
    next_piece_type: TetrominoType,
}

impl TetrisGame {
    fn new() -> Self {
        Self {
            board: [[false; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            score: 0,
            game_over: false,
            next_piece_type: TetrominoType::I,
        }
    }

    fn reset(&mut self) {
        self.board = [[false; BOARD_WIDTH]; BOARD_HEIGHT];
        self.current_piece = None;
        self.score = 0;
        self.game_over = false;
        self.spawn_piece();
    }

    fn spawn_piece(&mut self) {
        if self.game_over {
            return;
        }

        let new_piece = Tetromino::new(self.next_piece_type);

        // Check if we can spawn
        if self.check_collision(&new_piece) {
            self.game_over = true;
            return;
        }

        self.current_piece = Some(new_piece);
        self.next_piece_type = self.get_random_piece_type();
    }

    fn get_random_piece_type(&self) -> TetrominoType {
        // Simple pseudo-random based on current state
        let random = (self.score + self.board[0][0] as u32) % 7;
        match random {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::J,
            _ => TetrominoType::L,
        }
    }

    fn check_collision(&self, piece: &Tetromino) -> bool {
        let shape = piece.get_shape();

        for i in 0..4 {
            for j in 0..4 {
                if shape[i][j] {
                    let board_x = piece.x + j as i32;
                    let board_y = piece.y + i as i32;

                    // Check boundaries
                    if board_x < 0 || board_x >= BOARD_WIDTH as i32 {
                        return true;
                    }
                    if board_y < 0 || board_y >= BOARD_HEIGHT as i32 {
                        return true;
                    }

                    // Check collision with placed pieces
                    if self.board[board_y as usize][board_x as usize] {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn move_left(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece {
            piece.x -= 1;
            if !self.check_collision(&piece) {
                self.current_piece = Some(piece);
                return true;
            }
        }
        false
    }

    fn move_right(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece {
            piece.x += 1;
            if !self.check_collision(&piece) {
                self.current_piece = Some(piece);
                return true;
            }
        }
        false
    }

    fn move_down(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece {
            piece.y += 1;
            if !self.check_collision(&piece) {
                self.current_piece = Some(piece);
                return true;
            } else {
                // Lock piece
                self.lock_piece();
                return false;
            }
        }
        false
    }

    fn rotate(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece {
            piece.rotation = (piece.rotation + 1) % 4;
            if !self.check_collision(&piece) {
                self.current_piece = Some(piece);
                return true;
            }
        }
        false
    }

    fn hard_drop(&mut self) {
        while self.move_down() {}
    }

    fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece {
            let shape = piece.get_shape();

            for i in 0..4 {
                for j in 0..4 {
                    if shape[i][j] {
                        let board_x = piece.x + j as i32;
                        let board_y = piece.y + i as i32;

                        if board_y >= 0
                            && board_y < BOARD_HEIGHT as i32
                            && board_x >= 0
                            && board_x < BOARD_WIDTH as i32
                        {
                            self.board[board_y as usize][board_x as usize] = true;
                        }
                    }
                }
            }

            self.current_piece = None;
            self.clear_lines();
            self.spawn_piece();
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;

        for y in (0..BOARD_HEIGHT).rev() {
            let mut line_full = true;
            for x in 0..BOARD_WIDTH {
                if !self.board[y][x] {
                    line_full = false;
                    break;
                }
            }

            if line_full {
                lines_cleared += 1;
                // Move all lines above down
                for yy in (1..=y).rev() {
                    self.board[yy] = self.board[yy - 1];
                }
                self.board[0] = [false; BOARD_WIDTH];
            }
        }

        if lines_cleared > 0 {
            self.score += match lines_cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                _ => 800,
            };
        }
    }

    fn render_to_buffer(&self, buffer: &mut [u8]) -> usize {
        let mut pos = 0;

        // Create a temporary board with current piece
        let mut display_board = self.board;

        if let Some(piece) = self.current_piece {
            let shape = piece.get_shape();
            for i in 0..4 {
                for j in 0..4 {
                    if shape[i][j] {
                        let board_x = piece.x + j as i32;
                        let board_y = piece.y + i as i32;
                        if board_y >= 0
                            && board_y < BOARD_HEIGHT as i32
                            && board_x >= 0
                            && board_x < BOARD_WIDTH as i32
                        {
                            display_board[board_y as usize][board_x as usize] = true;
                        }
                    }
                }
            }
        }

        // Build output
        let top_border = b"\xE2\x95\x94"; // ╔
        let horizontal = b"\xE2\x95\x90"; // ═
        let top_right = b"\xE2\x95\x97\n"; // ╗\n

        // Top border
        for &byte in top_border {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }
        for _ in 0..BOARD_WIDTH {
            for &byte in horizontal {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
            for &byte in horizontal {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
        }
        for &byte in top_right {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }

        let left_border = b"\xE2\x95\x91"; // ║
        let right_border = b"\xE2\x95\x91\n"; // ║\n
        let filled = b"\xE2\x96\x88\xE2\x96\x88"; // ██
        let empty = b"  ";

        // Board rows
        for row in &display_board {
            for &byte in left_border {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
            for &cell in row {
                let bytes: &[u8] = if cell { filled } else { empty };
                for &byte in bytes {
                    if pos < buffer.len() {
                        buffer[pos] = byte;
                        pos += 1;
                    }
                }
            }
            for &byte in right_border {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
        }

        // Bottom border
        let bottom_left = b"\xE2\x95\x9A"; // ╚
        let bottom_right = b"\xE2\x95\x9D\n"; // ╝\n

        for &byte in bottom_left {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }
        for _ in 0..BOARD_WIDTH {
            for &byte in horizontal {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
            for &byte in horizontal {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
        }
        for &byte in bottom_right {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }

        // Score
        let score_text = b"Score: ";
        for &byte in score_text {
            if pos < buffer.len() {
                buffer[pos] = byte;
                pos += 1;
            }
        }

        // Simple integer to ASCII conversion
        let mut score = self.score;
        let mut digits = [0u8; 10];
        let mut digit_count = 0;

        if score == 0 {
            digits[0] = b'0';
            digit_count = 1;
        } else {
            while score > 0 && digit_count < 10 {
                digits[digit_count] = (score % 10) as u8 + b'0';
                score /= 10;
                digit_count += 1;
            }
        }

        for i in (0..digit_count).rev() {
            if pos < buffer.len() {
                buffer[pos] = digits[i];
                pos += 1;
            }
        }

        if pos < buffer.len() {
            buffer[pos] = b'\n';
            pos += 1;
        }

        // Game over message
        if self.game_over {
            let game_over_text = b"GAME OVER!\n";
            for &byte in game_over_text {
                if pos < buffer.len() {
                    buffer[pos] = byte;
                    pos += 1;
                }
            }
        }

        pos
    }
}

/// Device state
pub(crate) struct TetrisDevice {
    inner: Arc<TetrisDeviceInner>,
}

#[pin_data]
struct TetrisDeviceInner {
    #[pin]
    game: kernel::sync::Mutex<TetrisGame>,
}

impl TetrisDevice {
    fn new() -> Result<Arc<Self>> {
        let inner = Arc::pin_init(
            pin_init!(TetrisDeviceInner {
                game <- kernel::new_mutex!(TetrisGame::new()),
            }),
            GFP_KERNEL,
        )?;

        // Initialize the game
        inner.game.lock().spawn_piece();

        Ok(Arc::new(Self { inner }, GFP_KERNEL)?)
    }
}

#[vtable]
impl MiscDevice for TetrisDevice {
    type Ptr = Arc<TetrisDevice>;

    fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<Self::Ptr> {
        TetrisDevice::new()
    }

    fn read_iter(kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterDest<'_>) -> Result<usize> {
        let device = kiocb.file();
        let game = device.inner.game.lock();

        // Allocate a buffer to render the game state
        let mut buffer = kernel::alloc::KVec::new();
        buffer.resize(2048, 0, GFP_KERNEL)?;

        let len = game.render_to_buffer(&mut buffer);

        // Copy to user space
        let bytes_to_copy = core::cmp::min(len, iov.len());
        let mut copied = 0;

        while copied < bytes_to_copy {
            let chunk_size = core::cmp::min(bytes_to_copy - copied, buffer.len() - copied);
            if chunk_size == 0 {
                break;
            }

            // Use copy_to_iter equivalent
            let slice = &buffer[copied..copied + chunk_size];
            let n = iov.copy_to_iter(slice);
            if n == 0 {
                break;
            }
            copied += n;
        }

        Ok(copied)
    }

    fn write_iter(kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterSource<'_>) -> Result<usize> {
        let device = kiocb.file();
        let mut buffer = [0u8; 1];
        let len = iov.copy_from_iter(&mut buffer);

        if len > 0 {
            let mut game = device.inner.game.lock();
            match buffer[0] {
                b'a' | b'A' => {
                    game.move_left();
                }
                b'd' | b'D' => {
                    game.move_right();
                }
                b's' | b'S' => {
                    game.move_down();
                }
                b'w' | b'W' => {
                    game.rotate();
                }
                b' ' => {
                    game.hard_drop();
                }
                b'r' | b'R' => {
                    game.reset();
                }
                _ => {}
            }
        }

        Ok(len)
    }

    fn ioctl(
        device: <Self::Ptr as ForeignOwnable>::Borrowed<'_>,
        _file: &File,
        cmd: u32,
        _arg: usize,
    ) -> Result<isize> {
        let mut game = device.inner.game.lock();

        match cmd {
            0 => {
                game.move_left();
            }
            1 => {
                game.move_right();
            }
            2 => {
                game.move_down();
            }
            3 => {
                game.rotate();
            }
            4 => {
                game.hard_drop();
            }
            5 => {
                game.reset();
            }
            _ => return Err(EINVAL),
        }

        Ok(0)
    }
}

pub(crate) fn register_tetris_device(
) -> Result<Pin<kernel::alloc::KBox<MiscDeviceRegistration<TetrisDevice>>>> {
    kernel::alloc::KBox::pin_init(
        MiscDeviceRegistration::register(MiscDeviceOptions { name: c"tetris" }),
        GFP_KERNEL,
    )
}
