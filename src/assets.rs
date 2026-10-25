use shakmaty::{Color, Role};
use tiny_skia::Pixmap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardStyle {
    Blue,
    Green,
    Walnut,
    Glass,
    IcySea,
    Neon,
    Tournament,
    DarkWood,
    Marble,
    Bubblegum,
    EightBit,
}

impl BoardStyle {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "blue" | "azul" => Some(BoardStyle::Blue),
            "green" | "verde" => Some(BoardStyle::Green),
            "walnut" | "wood" | "madera" => Some(BoardStyle::Walnut),
            "glass" | "vidrio" | "cristal" => Some(BoardStyle::Glass),
            "icy_sea" | "icy" | "glaciar" | "hielo" => Some(BoardStyle::IcySea),
            "neon" | "cyber" => Some(BoardStyle::Neon),
            "tournament" | "torneo" => Some(BoardStyle::Tournament),
            "dark_wood" | "madera_oscura" => Some(BoardStyle::DarkWood),
            "marble" | "marmol" => Some(BoardStyle::Marble),
            "bubblegum" | "rosa" | "pink" => Some(BoardStyle::Bubblegum),
            "8_bit" | "8bit" | "retro" => Some(BoardStyle::EightBit),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BoardStyle::Blue => BoardStyle::Green,
            BoardStyle::Green => BoardStyle::Walnut,
            BoardStyle::Walnut => BoardStyle::Glass,
            BoardStyle::Glass => BoardStyle::IcySea,
            BoardStyle::IcySea => BoardStyle::Neon,
            BoardStyle::Neon => BoardStyle::Tournament,
            BoardStyle::Tournament => BoardStyle::DarkWood,
            BoardStyle::DarkWood => BoardStyle::Marble,
            BoardStyle::Marble => BoardStyle::Bubblegum,
            BoardStyle::Bubblegum => BoardStyle::EightBit,
            BoardStyle::EightBit => BoardStyle::Blue,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceStyle {
    Neo,
    Classic,
    Wood,
    Glass,
    IcySea,
    Neon,
    Tournament,
    Alpha,
    Bubblegum,
    EightBit,
}

impl PieceStyle {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "neo" | "default" => Some(PieceStyle::Neo),
            "classic" | "clasico" => Some(PieceStyle::Classic),
            "wood" | "madera" => Some(PieceStyle::Wood),
            "glass" | "vidrio" => Some(PieceStyle::Glass),
            "icy_sea" | "icy" | "hielo" => Some(PieceStyle::IcySea),
            "neon" => Some(PieceStyle::Neon),
            "tournament" | "torneo" => Some(PieceStyle::Tournament),
            "alpha" => Some(PieceStyle::Alpha),
            "bubblegum" | "rosa" | "pink" => Some(PieceStyle::Bubblegum),
            "8_bit" | "8bit" | "retro" => Some(PieceStyle::EightBit),
            _ => None,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PieceStyle::Neo => PieceStyle::Classic,
            PieceStyle::Classic => PieceStyle::Wood,
            PieceStyle::Wood => PieceStyle::Glass,
            PieceStyle::Glass => PieceStyle::IcySea,
            PieceStyle::IcySea => PieceStyle::Neon,
            PieceStyle::Neon => PieceStyle::Tournament,
            PieceStyle::Tournament => PieceStyle::Alpha,
            PieceStyle::Alpha => PieceStyle::Bubblegum,
            PieceStyle::Bubblegum => PieceStyle::EightBit,
            PieceStyle::EightBit => PieceStyle::Neo,
        }
    }
}

pub struct PieceSet {
    pub wk: Pixmap,
    pub wq: Pixmap,
    pub wr: Pixmap,
    pub wb: Pixmap,
    pub wn: Pixmap,
    pub wp: Pixmap,

    pub bk: Pixmap,
    pub bq: Pixmap,
    pub br: Pixmap,
    pub bb: Pixmap,
    pub bn: Pixmap,
    pub bp: Pixmap,
}

impl PieceSet {
    pub fn get(&self, color: Color, role: Role) -> &Pixmap {
        match (color, role) {
            (Color::White, Role::King) => &self.wk,
            (Color::White, Role::Queen) => &self.wq,
            (Color::White, Role::Rook) => &self.wr,
            (Color::White, Role::Bishop) => &self.wb,
            (Color::White, Role::Knight) => &self.wn,
            (Color::White, Role::Pawn) => &self.wp,

            (Color::Black, Role::King) => &self.bk,
            (Color::Black, Role::Queen) => &self.bq,
            (Color::Black, Role::Rook) => &self.br,
            (Color::Black, Role::Bishop) => &self.bb,
            (Color::Black, Role::Knight) => &self.bn,
            (Color::Black, Role::Pawn) => &self.bp,
        }
    }
}

macro_rules! load_pieces {
    ($path:literal) => {
        PieceSet {
            wk: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wk.png"))).unwrap(),
            wq: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wq.png"))).unwrap(),
            wr: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wr.png"))).unwrap(),
            wb: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wb.png"))).unwrap(),
            wn: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wn.png"))).unwrap(),
            wp: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/wp.png"))).unwrap(),

            bk: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/bk.png"))).unwrap(),
            bq: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/bq.png"))).unwrap(),
            br: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/br.png"))).unwrap(),
            bb: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/bb.png"))).unwrap(),
            bn: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/bn.png"))).unwrap(),
            bp: Pixmap::decode_png(include_bytes!(concat!("../assets/pieces/", $path, "/bp.png"))).unwrap(),
        }
    };
}

macro_rules! load_board {
    ($path:literal) => {
        Pixmap::decode_png(include_bytes!(concat!("../assets/boards/", $path, ".png"))).unwrap()
    };
}

pub struct AssetManager {
    // Tableros oficiales de Chess.com
    pub board_blue: Pixmap,
    pub board_green: Pixmap,
    pub board_walnut: Pixmap,
    pub board_glass: Pixmap,
    pub board_icy_sea: Pixmap,
    pub board_neon: Pixmap,
    pub board_tournament: Pixmap,
    pub board_dark_wood: Pixmap,
    pub board_marble: Pixmap,
    pub board_bubblegum: Pixmap,
    pub board_8_bit: Pixmap,

    // Sets de piezas oficiales de Chess.com
    pub pieces_neo: PieceSet,
    pub pieces_classic: PieceSet,
    pub pieces_wood: PieceSet,
    pub pieces_glass: PieceSet,
    pub pieces_icy_sea: PieceSet,
    pub pieces_neon: PieceSet,
    pub pieces_tournament: PieceSet,
    pub pieces_alpha: PieceSet,
    pub pieces_bubblegum: PieceSet,
    pub pieces_8_bit: PieceSet,
}

impl AssetManager {
    pub fn load_embedded() -> Self {
        Self {
            board_blue: load_board!("blue"),
            board_green: load_board!("green"),
            board_walnut: load_board!("walnut"),
            board_glass: load_board!("glass"),
            board_icy_sea: load_board!("icy_sea"),
            board_neon: load_board!("neon"),
            board_tournament: load_board!("tournament"),
            board_dark_wood: load_board!("dark_wood"),
            board_marble: load_board!("marble"),
            board_bubblegum: load_board!("bubblegum"),
            board_8_bit: load_board!("8_bit"),

            pieces_neo: load_pieces!("neo"),
            pieces_classic: load_pieces!("classic"),
            pieces_wood: load_pieces!("wood"),
            pieces_glass: load_pieces!("glass"),
            pieces_icy_sea: load_pieces!("icy_sea"),
            pieces_neon: load_pieces!("neon"),
            pieces_tournament: load_pieces!("tournament"),
            pieces_alpha: load_pieces!("alpha"),
            pieces_bubblegum: load_pieces!("bubblegum"),
            pieces_8_bit: load_pieces!("8_bit"),
        }
    }

    pub fn get_board(&self, style: BoardStyle) -> &Pixmap {
        match style {
            BoardStyle::Blue => &self.board_blue,
            BoardStyle::Green => &self.board_green,
            BoardStyle::Walnut => &self.board_walnut,
            BoardStyle::Glass => &self.board_glass,
            BoardStyle::IcySea => &self.board_icy_sea,
            BoardStyle::Neon => &self.board_neon,
            BoardStyle::Tournament => &self.board_tournament,
            BoardStyle::DarkWood => &self.board_dark_wood,
            BoardStyle::Marble => &self.board_marble,
            BoardStyle::Bubblegum => &self.board_bubblegum,
            BoardStyle::EightBit => &self.board_8_bit,
        }
    }

    pub fn get_piece(&self, style: PieceStyle, color: Color, role: Role) -> &Pixmap {
        let set = match style {
            PieceStyle::Neo => &self.pieces_neo,
            PieceStyle::Classic => &self.pieces_classic,
            PieceStyle::Wood => &self.pieces_wood,
            PieceStyle::Glass => &self.pieces_glass,
            PieceStyle::IcySea => &self.pieces_icy_sea,
            PieceStyle::Neon => &self.pieces_neon,
            PieceStyle::Tournament => &self.pieces_tournament,
            PieceStyle::Alpha => &self.pieces_alpha,
            PieceStyle::Bubblegum => &self.pieces_bubblegum,
            PieceStyle::EightBit => &self.pieces_8_bit,
        };
        set.get(color, role)
    }
}
