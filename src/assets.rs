use shakmaty::{Color, Role};
use tiny_skia::Pixmap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardTheme {
    Amber,
    Blue,
    Pink,
}

impl BoardTheme {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "amber" | "ambar" | "miel" | "madera" => Some(BoardTheme::Amber),
            "blue" | "azul" | "glaciar" | "hielo" => Some(BoardTheme::Blue),
            "pink" | "rosa" | "pastel" => Some(BoardTheme::Pink),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BoardTheme::Amber => BoardTheme::Blue,
            BoardTheme::Blue => BoardTheme::Pink,
            BoardTheme::Pink => BoardTheme::Amber,
        }
    }
}

pub struct PieceAssets {
    pub rook: Pixmap,
    pub knight: Pixmap,
    pub bishop: Pixmap,
    pub queen: Pixmap,
    pub king: Pixmap,
    pub pawn: Pixmap,
}

impl PieceAssets {
    fn from_bytes(
        rook_b: &[u8],
        knight_b: &[u8],
        bishop_b: &[u8],
        queen_b: &[u8],
        king_b: &[u8],
        pawn_b: &[u8],
    ) -> Self {
        Self {
            rook: Pixmap::decode_png(rook_b).expect("failed to decode rook"),
            knight: Pixmap::decode_png(knight_b).expect("failed to decode knight"),
            bishop: Pixmap::decode_png(bishop_b).expect("failed to decode bishop"),
            queen: Pixmap::decode_png(queen_b).expect("failed to decode queen"),
            king: Pixmap::decode_png(king_b).expect("failed to decode king"),
            pawn: Pixmap::decode_png(pawn_b).expect("failed to decode pawn"),
        }
    }

    pub fn get(&self, role: Role) -> &Pixmap {
        match role {
            Role::Rook => &self.rook,
            Role::Knight => &self.knight,
            Role::Bishop => &self.bishop,
            Role::Queen => &self.queen,
            Role::King => &self.king,
            Role::Pawn => &self.pawn,
        }
    }
}

pub struct TileAssets {
    pub dark: Pixmap,
    pub light: Pixmap,
}

pub struct AssetManager {
    // 6 Colores de piezas
    pub pink_pieces: PieceAssets,
    pub blue_pieces: PieceAssets,
    pub lime_pieces: PieceAssets,
    pub ochre_pieces: PieceAssets,
    pub purple_pieces: PieceAssets,
    pub cyan_pieces: PieceAssets,

    // 3 Tableros de baldosas a juego
    pub amber_tiles: TileAssets,
    pub blue_tiles: TileAssets,
    pub pink_tiles: TileAssets,
}

impl AssetManager {
    pub fn load_embedded() -> Self {
        Self {
            // Set 01 (Rosa & Azul)
            pink_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_01/pink/rook.png"),
                include_bytes!("../assets/sprites/set_01/pink/knight.png"),
                include_bytes!("../assets/sprites/set_01/pink/bishop.png"),
                include_bytes!("../assets/sprites/set_01/pink/queen.png"),
                include_bytes!("../assets/sprites/set_01/pink/king.png"),
                include_bytes!("../assets/sprites/set_01/pink/pawn.png"),
            ),
            blue_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_01/blue/rook.png"),
                include_bytes!("../assets/sprites/set_01/blue/knight.png"),
                include_bytes!("../assets/sprites/set_01/blue/bishop.png"),
                include_bytes!("../assets/sprites/set_01/blue/queen.png"),
                include_bytes!("../assets/sprites/set_01/blue/king.png"),
                include_bytes!("../assets/sprites/set_01/blue/pawn.png"),
            ),

            // Set 02 (Lima & Ocre)
            lime_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_02/lime/rook.png"),
                include_bytes!("../assets/sprites/set_02/lime/knight.png"),
                include_bytes!("../assets/sprites/set_02/lime/bishop.png"),
                include_bytes!("../assets/sprites/set_02/lime/queen.png"),
                include_bytes!("../assets/sprites/set_02/lime/king.png"),
                include_bytes!("../assets/sprites/set_02/lime/pawn.png"),
            ),
            ochre_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_02/ochre/rook.png"),
                include_bytes!("../assets/sprites/set_02/ochre/knight.png"),
                include_bytes!("../assets/sprites/set_02/ochre/bishop.png"),
                include_bytes!("../assets/sprites/set_02/ochre/queen.png"),
                include_bytes!("../assets/sprites/set_02/ochre/king.png"),
                include_bytes!("../assets/sprites/set_02/ochre/pawn.png"),
            ),

            // Set 03 (Púrpura & Cian)
            purple_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_03/purple/rook.png"),
                include_bytes!("../assets/sprites/set_03/purple/knight.png"),
                include_bytes!("../assets/sprites/set_03/purple/bishop.png"),
                include_bytes!("../assets/sprites/set_03/purple/queen.png"),
                include_bytes!("../assets/sprites/set_03/purple/king.png"),
                include_bytes!("../assets/sprites/set_03/purple/pawn.png"),
            ),
            cyan_pieces: PieceAssets::from_bytes(
                include_bytes!("../assets/sprites/set_03/cyan/rook.png"),
                include_bytes!("../assets/sprites/set_03/cyan/knight.png"),
                include_bytes!("../assets/sprites/set_03/cyan/bishop.png"),
                include_bytes!("../assets/sprites/set_03/cyan/queen.png"),
                include_bytes!("../assets/sprites/set_03/cyan/king.png"),
                include_bytes!("../assets/sprites/set_03/cyan/pawn.png"),
            ),

            // Baldosas
            amber_tiles: TileAssets {
                dark: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/amber_dark.png")).unwrap(),
                light: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/amber_light.png")).unwrap(),
            },
            blue_tiles: TileAssets {
                dark: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/blue_dark.png")).unwrap(),
                light: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/blue_light.png")).unwrap(),
            },
            pink_tiles: TileAssets {
                dark: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/pink_dark.png")).unwrap(),
                light: Pixmap::decode_png(include_bytes!("../assets/sprites/tiles/pink_light.png")).unwrap(),
            },
        }
    }

    pub fn get_piece(&self, theme: BoardTheme, color: Color, role: Role) -> &Pixmap {
        match (theme, color) {
            // Ámbar: Piezas Lima vs Ocre Dorado
            (BoardTheme::Amber, Color::White) => self.lime_pieces.get(role),
            (BoardTheme::Amber, Color::Black) => self.ochre_pieces.get(role),

            // Azul Glaciar: Piezas Púrpura vs Cian Hielo
            (BoardTheme::Blue, Color::White) => self.cyan_pieces.get(role),
            (BoardTheme::Blue, Color::Black) => self.purple_pieces.get(role),

            // Rosa Pastel: Piezas Rosa vs Azul Noche
            (BoardTheme::Pink, Color::White) => self.pink_pieces.get(role),
            (BoardTheme::Pink, Color::Black) => self.blue_pieces.get(role),
        }
    }

    pub fn get_tile(&self, theme: BoardTheme, is_dark: bool) -> &Pixmap {
        match theme {
            BoardTheme::Amber => {
                if is_dark { &self.amber_tiles.dark } else { &self.amber_tiles.light }
            }
            BoardTheme::Blue => {
                if is_dark { &self.blue_tiles.dark } else { &self.blue_tiles.light }
            }
            BoardTheme::Pink => {
                if is_dark { &self.pink_tiles.dark } else { &self.pink_tiles.light }
            }
        }
    }
}
