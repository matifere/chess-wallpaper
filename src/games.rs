pub struct GameData {
    pub name: &'static str,
    pub moves: &'static [&'static str],
}

pub fn get_master_games() -> Vec<GameData> {
    vec![
        // 1. The Opera Game: Morphy vs Duke of Brunswick (1858)
        GameData {
            name: "Paul Morphy vs Duke of Brunswick (1858) - The Opera Game",
            moves: &[
                "e4", "e5", "Nf3", "d6", "d4", "Bg4", "dxe5", "Bxf3", "Qxf3", "dxe5",
                "Bc4", "Nf6", "Qb3", "Qe7", "Nc3", "c6", "Bg5", "b5", "Nxb5", "cxb5",
                "Bxb5+", "Nbd7", "O-O-O", "Rd8", "Rxd7", "Rxd7", "Rd1", "Qe6", "Bxd7+", "Nxd7",
                "Qb8+", "Nxb8", "Rd8#"
            ],
        },
        // 2. The Immortal Game: Adolf Anderssen vs Lionel Kieseritzky (1851)
        GameData {
            name: "Adolf Anderssen vs Lionel Kieseritzky (1851) - The Immortal Game",
            moves: &[
                "e4", "e5", "f4", "exf4", "Bc4", "Qh4+", "Kf1", "b5", "Bxb5", "Nf6",
                "Nf3", "Qh6", "d3", "Nh5", "Nh4", "Qg5", "Nf5", "c6", "g4", "Nf6",
                "Rg1", "cxb5", "h4", "Qg6", "h5", "Qg5", "Qf3", "Ng8", "Bxf4", "Qf6",
                "Nc3", "Bc5", "Nd5", "Qxb2", "Bd6", "Bxg1", "e5", "Qxa1+", "Ke2", "Na6",
                "Nxg7+", "Kd8", "Qf6+", "Nxf6", "Be7#"
            ],
        },
        // 3. The Evergreen Game: Adolf Anderssen vs Jean Dufresne (1852)
        GameData {
            name: "Adolf Anderssen vs Jean Dufresne (1852) - The Evergreen Game",
            moves: &[
                "e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "b4", "Bxb4", "c3", "Ba5",
                "d4", "exd4", "O-O", "d3", "Qb3", "Qf6", "e5", "Qg6", "Re1", "Nge7",
                "Ba3", "b5", "Qxb5", "Rb8", "Qa4", "Bb6", "Nbd2", "Bb7", "Ne4", "Qf5",
                "Bxd3", "Qh5", "Nf6+", "gxf6", "exf6", "Rg8", "Rad1", "Qxf3", "Rxe7+", "Nxe7",
                "Qxd7+", "Kxd7", "Bf5+", "Ke8", "Bd7+", "Kf8", "Bxe7#"
            ],
        },
        // 4. Reti vs Tartakower: Double Check Mate (1910)
        GameData {
            name: "Richard Reti vs Savielly Tartakower (1910) - Double Check Mate",
            moves: &[
                "e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Nf6", "Qd3", "e5",
                "dxe5", "Qa5+", "Bd2", "Qxe5", "O-O-O", "Nxe4", "Qd8+", "Kxd8", "Bg5+", "Kc7", "Bd8#"
            ],
        },
        // 5. The Game of the Century: Donald Byrne vs Bobby Fischer (1956)
        GameData {
            name: "Donald Byrne vs Bobby Fischer (1956) - The Game of the Century",
            moves: &[
                "Nf3", "Nf6", "c4", "g6", "Nc3", "Bg7", "d4", "O-O", "Bf4", "d5",
                "Qb3", "dxc4", "Qxc4", "c6", "e4", "Nbd7", "Rd1", "Nb6", "Qc5", "Bg4",
                "Bg5", "Na4", "Qa3", "Nxc3", "bxc3", "Nxe4", "Bxe7", "Qb6", "Bc4", "Nxc3",
                "Bc5", "Rfe8+", "Kf1", "Be6", "Bxb6", "Bxc4+", "Kg1", "Ne2+", "Kf1", "Nxd4+",
                "Kg1", "Ne2+", "Kf1", "Nc3+", "Kg1", "axb6", "Qb4", "Ra4", "Qxb6", "Nxd1",
                "h3", "Rxa2", "Kh2", "Nxf2", "Re1", "Rxe1", "Qd8+", "Bf8", "Nxe1", "Bd5",
                "Nf3", "Ne4", "Qb8", "b5", "h4", "h5", "Ne5", "Kg7", "Kg1", "Bc5+",
                "Kf1", "Ng3+", "Ke1", "Bb4+", "Kd1", "Bb3+", "Kc1", "Ne2+", "Kb1", "Nc3+",
                "Kc1", "Rc2#"
            ],
        },
        // 6. Edward Lasker vs Sir George Thomas (1912) - The King Hunt
        GameData {
            name: "Edward Lasker vs George Thomas (1912) - The King Hunt",
            moves: &[
                "d4", "f5", "Nf3", "e6", "Nc3", "Nf6", "Bg5", "Be7", "Bxf6", "Bxf6",
                "e4", "fxe4", "Nxe4", "b6", "Ne5", "O-O", "Bd3", "Bb7", "Qh5", "Qe7",
                "Qxh7+", "Kxh7", "Nxf6+", "Kh6", "Neg4+", "Kg5", "h4+", "Kf4", "g3+", "Kf3",
                "Be2+", "Kg2", "Rh2+", "Kg1", "Kd2#"
            ],
        },
        // 7. Akiba Rubinstein's Immortal Game vs Georg Rotlewi (1907)
        GameData {
            name: "Georg Rotlewi vs Akiba Rubinstein (1907) - Rubinstein's Immortal",
            moves: &[
                "d4", "d5", "Nf3", "e6", "e3", "c5", "c4", "Nc6", "Nc3", "Nf6",
                "dxc5", "Bxc5", "a3", "a6", "b4", "Bd6", "Bb2", "O-O", "Qd2", "Qe7",
                "Bd3", "dxc4", "Bxc4", "b5", "Bd3", "Rd8", "Qe2", "Bb7", "O-O", "Ne5",
                "Nxe5", "Bxe5", "f4", "Bc7", "e4", "Rac8", "e5", "Bb6+", "Kh1", "Ng4",
                "Be4", "Qh4", "g3", "Rxc3", "gxh4", "Rd2", "Qxd2", "Bxe4+", "Qg2", "Rh3"
            ],
        },
        // 8. Mikhail Tal vs Bent Larsen (1965) - Tal's Knight Sacrifice
        GameData {
            name: "Mikhail Tal vs Bent Larsen (1965) - Tal's Attack",
            moves: &[
                "e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4", "e6", "Nc3", "d6",
                "Be3", "Nf6", "f4", "Be7", "Qf3", "O-O", "O-O-O", "Qc7", "Ndb5", "Qb8",
                "g4", "a6", "Nd4", "Nxd4", "Bxd4", "b5", "g5", "Nd7", "Bd3", "b4",
                "Nd5", "exd5", "exd5", "f5", "gxf6", "Nxf6", "Rhg1", "Rf7", "Rde1", "Qb7",
                "Re6", "Bxe6", "dxe6", "Rff8", "Bxf6", "Bxf6", "Qh5", "g6", "Bxg6", "Kh8",
                "Bf7", "Rxf7", "exf7", "Rf8", "Qf5", "Qe7", "Kd1", "Qxf7", "Re1", "Bxb2"
            ],
        },
        // 9. Young Magnus Carlsen vs Sipke Ernst (2004)
        GameData {
            name: "Magnus Carlsen vs Sipke Ernst (2004) - Young Magnus",
            moves: &[
                "e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Bf5", "Ng3", "Bg6",
                "h4", "h6", "Nf3", "Nd7", "h5", "Bh7", "Bd3", "Bxd3", "Qxd3", "e6",
                "Bf4", "Ngf6", "O-O-O", "Be7", "Ne4", "Nxe4", "Qxe4", "Nf6", "Qe2", "Qd5",
                "Kb1", "Qe4", "Be3", "Ng4", "Nd2", "Qf5", "f3", "Nxe3", "Qxe3", "O-O",
                "g4", "Qa5", "f4", "Rad8", "Nf3", "Rd5", "Ne5", "Rfd8", "g5", "hxg5",
                "fxg5", "c5", "g6", "f6", "h6", "cxd4", "hxg7"
            ],
        },
        // 10. Deep Blue vs Garry Kasparov (1997) - Game 6
        GameData {
            name: "Deep Blue vs Garry Kasparov (1997) - Game 6",
            moves: &[
                "e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Nd7", "Ng5", "Ngf6",
                "Bd3", "e6", "N1f3", "h6", "Nxe6", "Qe7", "O-O", "fxe6", "Bg6+", "Kd8",
                "Bf4", "b5", "a4", "Bb7", "Re1", "Nd5", "Bg3", "Kc8", "axb5", "cxb5",
                "Qd3", "Bc6", "Bf5", "exf5", "Rxe7", "Bxe7", "c4"
            ],
        },
        // 11. Emanuel Lasker vs Johann Bauer (1889) - Double Bishop Sacrifice
        GameData {
            name: "Emanuel Lasker vs Johann Bauer (1889) - Double Bishop Sacrifice",
            moves: &[
                "f4", "d5", "e3", "Nf6", "b3", "e6", "Bb2", "Be7", "Bd3", "b6",
                "Nf3", "Bb7", "Nc3", "Nbd7", "O-O", "O-O", "Ne2", "c5", "Ng3", "Qc7",
                "Ne5", "Nxe5", "Bxe5", "Qc6", "Qe2", "a6", "Nh5", "Nxh5", "Bxh7+", "Kxh7",
                "Qxh5+", "Kg8", "Bxg7", "Kxg7", "Qg4+", "Kh7", "Rf3", "e5", "Rh3+", "Qh6",
                "Rxh6+", "Kxh6", "Qd7", "Bf6", "Qxb7", "Kg7", "Rf1", "Rab8", "Qxd5", "Rfd8",
                "Qe4", "exf4", "Rxf4", "Rd6", "Rg4+", "Kf8", "Qf4"
            ],
        },
        // 12. Nigel Short vs Jan Timman (1991) - The King March
        GameData {
            name: "Nigel Short vs Jan Timman (1991) - The King March",
            moves: &[
                "e4", "Nf6", "e5", "Nd5", "d4", "d6", "Nf3", "g6", "Bc4", "Nb6",
                "Bb3", "Bg7", "Qe2", "Nc6", "O-O", "O-O", "h3", "a5", "a4", "dxe5",
                "dxe5", "Nd4", "Nxd4", "Qxd4", "Re1", "e6", "Nd2", "Nd5", "Nf3", "Qc5",
                "Qe4", "Qb4", "Bc4", "Nb6", "b3", "Nxc4", "bxc4", "Re8", "Rd1", "Qc5",
                "Qh4", "b6", "Be3", "Qc6", "Bh6", "Bh8", "Rd8", "Bb7", "Rad1", "Bg7",
                "R8d7", "Rf8", "Bxg7", "Kxg7", "R1d4", "Rae8", "Qf6+", "Kg8", "h4", "h5",
                "Kh2", "Rc8", "Kg3", "Rce8", "Kf4", "Bc8", "Kg5"
            ],
        },
    ]
}
