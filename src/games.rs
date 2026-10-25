#[derive(Clone, Debug)]
pub struct GameData {
    pub white: &'static str,
    pub black: &'static str,
    pub event: &'static str,
    pub year: u16,
    pub result: &'static str,
    pub termination: &'static str,
    pub moves: &'static [&'static str],
}

pub fn get_master_games() -> Vec<GameData> {
    vec![
        // 1. The Opera Game: Morphy vs Duke of Brunswick (1858)
        GameData {
            white: "Paul Morphy",
            black: "Duque de Brunswick",
            event: "The Opera Game",
            year: 1858,
            result: "1-0",
            termination: "Jaque mate",
            moves: &[
                "e4", "e5", "Nf3", "d6", "d4", "Bg4", "dxe5", "Bxf3", "Qxf3", "dxe5",
                "Bc4", "Nf6", "Qb3", "Qe7", "Nc3", "c6", "Bg5", "b5", "Nxb5", "cxb5",
                "Bxb5+", "Nbd7", "O-O-O", "Rd8", "Rxd7", "Rxd7", "Rd1", "Qe6", "Bxd7+", "Nxd7",
                "Qb8+", "Nxb8", "Rd8#"
            ],
        },
        // 2. The Immortal Game: Adolf Anderssen vs Lionel Kieseritzky (1851)
        GameData {
            white: "Adolf Anderssen",
            black: "Lionel Kieseritzky",
            event: "The Immortal Game",
            year: 1851,
            result: "1-0",
            termination: "Jaque mate",
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
            white: "Adolf Anderssen",
            black: "Jean Dufresne",
            event: "The Evergreen Game",
            year: 1852,
            result: "1-0",
            termination: "Jaque mate",
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
            white: "Richard Réti",
            black: "Savielly Tartakower",
            event: "Double Check Mate",
            year: 1910,
            result: "1-0",
            termination: "Jaque mate",
            moves: &[
                "e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Nf6", "Qd3", "e5",
                "dxe5", "Qa5+", "Bd2", "Qxe5", "O-O-O", "Nxe4", "Qd8+", "Kxd8", "Bg5+", "Kc7", "Bd8#"
            ],
        },
        // 5. The Game of the Century: Donald Byrne vs Bobby Fischer (1956)
        GameData {
            white: "Donald Byrne",
            black: "Bobby Fischer",
            event: "The Game of the Century",
            year: 1956,
            result: "0-1",
            termination: "Jaque mate",
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
            white: "Edward Lasker",
            black: "Sir George Thomas",
            event: "The King Hunt",
            year: 1912,
            result: "1-0",
            termination: "Jaque mate",
            moves: &[
                "d4", "f5", "Nf3", "e6", "Nc3", "Nf6", "Bg5", "Be7", "Bxf6", "Bxf6",
                "e4", "fxe4", "Nxe4", "b6", "Ne5", "O-O", "Bd3", "Bb7", "Qh5", "Qe7",
                "Qxh7+", "Kxh7", "Nxf6+", "Kh6", "Neg4+", "Kg5", "h4+", "Kf4", "g3+", "Kf3",
                "Be2+", "Kg2", "Rh2+", "Kg1", "Kd2#"
            ],
        },
        // 7. Akiba Rubinstein's Immortal Game vs Georg Rotlewi (1907)
        GameData {
            white: "Georg Rotlewi",
            black: "Akiba Rubinstein",
            event: "Rubinstein's Immortal",
            year: 1907,
            result: "0-1",
            termination: "Abandono",
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
            white: "Mikhail Tal",
            black: "Bent Larsen",
            event: "Tal's Attack",
            year: 1965,
            result: "1-0",
            termination: "Abandono",
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
            white: "Magnus Carlsen",
            black: "Sipke Ernst",
            event: "Young Magnus",
            year: 2004,
            result: "1-0",
            termination: "Abandono",
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
            white: "Deep Blue",
            black: "Garry Kasparov",
            event: "Match Game 6",
            year: 1997,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Nd7", "Ng5", "Ngf6",
                "Bd3", "e6", "N1f3", "h6", "Nxe6", "Qe7", "O-O", "fxe6", "Bg6+", "Kd8",
                "Bf4", "b5", "a4", "Bb7", "Re1", "Nd5", "Bg3", "Kc8", "axb5", "cxb5",
                "Qd3", "Bc6", "Bf5", "exf5", "Rxe7", "Bxe7", "c4"
            ],
        },
        // 11. Emanuel Lasker vs Johann Bauer (1889) - Double Bishop Sacrifice
        GameData {
            white: "Emanuel Lasker",
            black: "Johann Bauer",
            event: "Double Bishop Sacrifice",
            year: 1889,
            result: "1-0",
            termination: "Abandono",
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
            white: "Nigel Short",
            black: "Jan Timman",
            event: "The King March",
            year: 1991,
            result: "1-0",
            termination: "Abandono",
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
        // 13. Garry Kasparov vs Veselin Topalov (1999) - Kasparov's Immortal
        GameData {
            white: "Garry Kasparov",
            black: "Veselin Topalov",
            event: "Kasparov's Immortal",
            year: 1999,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "d6", "d4", "Nf6", "Nc3", "g6", "Be3", "Bg7", "Qd2", "c6",
                "f3", "b5", "Nge2", "Nbd7", "Bh6", "Bxh6", "Qxh6", "Bb7", "a3", "e5",
                "O-O-O", "Qe7", "Kb1", "a6", "Nc1", "O-O-O", "Nb3", "exd4", "Rxd4", "c5",
                "Rd1", "Nb6", "g3", "Kb8", "Na5", "Ba8", "Bh3", "d5", "Qf4+", "Ka7",
                "Rhe1", "d4", "Nd5", "Nbxd5", "exd5", "Qd6", "Rxd4", "cxd4", "Re7+", "Kb6",
                "Qxd4+", "Kxa5", "b4+", "Ka4", "Qc3", "Qxd5", "Ra7", "Bb7", "Rxb7", "Qc4",
                "Qxf6", "Kxa3", "Qxa6+", "Kxb4", "c3+", "Kxc3", "Qa1+", "Kd2", "Qb2+", "Kd1",
                "Bf1", "Rd2", "Rd7", "Rxd7", "Bxc4", "bxc4", "Qxh8", "Rd3", "Qa8", "c3",
                "Qa4+", "Ke1", "f4", "f5", "Kc1", "Rd2", "Qa7"
            ],
        },
        // 14. Stefan Levitsky vs Frank Marshall (1912) - The Gold Coins Game
        GameData {
            white: "Stefan Levitsky",
            black: "Frank Marshall",
            event: "The Gold Coins Game",
            year: 1912,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "d4", "e6", "e4", "d5", "Nc3", "c5", "Nf3", "Nc6", "exd5", "exd5",
                "Be2", "Nf6", "O-O", "Be7", "Bg5", "O-O", "dxc5", "Be6", "Nd4", "Bxc5",
                "Nxe6", "fxe6", "Bg4", "Qd6", "Bh3", "Rae8", "Qd2", "Bb4", "Bxf6", "Rxf6",
                "Rad1", "Qc5", "Qe2", "Bxc3", "bxc3", "Qxc3", "Rxd5", "Nd4", "Qh5", "Ref8",
                "Re5", "Rh6", "Qg5", "Rxh3", "Rc5", "Qg3"
            ],
        },
        // 15. Wei Yi vs Lazaro Bruzon (2015) - Modern Immortal
        GameData {
            white: "Wei Yi",
            black: "Lázaro Bruzón",
            event: "Modern Immortal",
            year: 2015,
            result: "1-0",
            termination: "Jaque mate",
            moves: &[
                "e4", "c5", "Nf3", "e6", "Nc3", "a6", "Be2", "Nc6", "d4", "cxd4",
                "Nxd4", "Qc7", "O-O", "Nf6", "Be3", "Be7", "f4", "d6", "Kh1", "O-O",
                "Qe1", "Nxd4", "Bxd4", "b5", "Qg3", "Bb7", "a3", "Rad8", "Rae1", "Rd7",
                "Bd3", "Qd8", "Qh3", "g6", "f5", "e5", "Be3", "Re8", "fxg6", "hxg6",
                "Nd5", "Nxd5", "Rxf7", "Kxf7", "Qh7+", "Ke6", "exd5+", "Kxd5", "Be4+", "Kxe4",
                "Qf7", "Bf6", "Bd2+", "Kd4", "Be3+", "Ke4", "Qb3", "Kf5", "Rf1+", "Kg4",
                "Qd3", "Bxg2+", "Kxg2", "Qa8+", "Kg1", "Bg5", "Qe2+", "Kh4", "Bf2+", "Kh3",
                "Be1", "Rf8", "Qd3+", "Kg4", "Qg3+", "Kh5", "Qh3+", "Bh4", "Qxh4#"
            ],
        },
        // 16. Rashid Nezhmetdinov vs Oleg Chernikov (1962) - Queen Sacrifice
        GameData {
            white: "Rashid Nezhmetdinov",
            black: "Oleg Chernikov",
            event: "Nezhmetdinov's Queen Sac",
            year: 1962,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4", "g6", "Nc3", "Bg7",
                "Be3", "Nf6", "Bc4", "O-O", "Bb3", "Ng4", "Qxg4", "Nxd4", "Qh4", "Qa5",
                "O-O", "Bf6", "Qxf6", "Ne2+", "Nxe2", "exf6", "Nc3", "Re8", "Nd5", "Re6",
                "Bd4", "Kg7", "Rad1", "d6", "Rd3", "Bd7", "Rf3", "Bb5", "Bc3", "Qd8",
                "Nxf6", "Be2", "Nxh7+", "Kg8", "Rh3", "Re5", "f4", "Bxf1", "Kxf1", "Rc8",
                "Bd4", "b5", "Ng5", "Rc7", "Bxf7+", "Rxf7", "Rh8+", "Kxh8", "Nxf7+", "Kh7",
                "Nxd8", "Rxe4", "Nc6", "Rxf4+", "Ke2"
            ],
        },
        // 17. Mikhail Botvinnik vs José Raúl Capablanca (1938) - AVRO Deflection
        GameData {
            white: "Mikhail Botvinnik",
            black: "José Raúl Capablanca",
            event: "AVRO Deflection",
            year: 1938,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "d4", "Nf6", "c4", "e6", "Nc3", "Bb4", "e3", "d5", "a3", "Bxc3+",
                "bxc3", "c5", "cxd5", "exd5", "Bd3", "O-O", "Ne2", "b6", "O-O", "Ba6",
                "Bxa6", "Nxa6", "Bb2", "Qd7", "a4", "Rfe8", "Qd3", "c4", "Qc2", "Nb8",
                "Rae1", "Nc6", "Ng3", "Na5", "f3", "Nb3", "e4", "Qxa4", "e5", "Nd7",
                "Qf2", "g6", "f4", "f5", "exf6", "Nxf6", "f5", "Rxe1", "Rxe1", "Re8",
                "Re6", "Rxe6", "fxe6", "Kg7", "Qf4", "Qe8", "Qe5", "Qe7", "Ba3", "Qxa3",
                "Nh5+", "gxh5", "Qg5+", "Kf8", "Qxf6+", "Kg8", "e7", "Qc1+", "Kf2", "Qc2+",
                "Kg3", "Qd3+", "Kh4", "Qe4+", "Kxh5", "Qe2+", "Kh4", "Qe4+", "g4", "Qe1+",
                "Kh5"
            ],
        },
        // 18. Louis Paulsen vs Paul Morphy (1857) - Morphy's Queen Sac
        GameData {
            white: "Louis Paulsen",
            black: "Paul Morphy",
            event: "Morphy's Queen Sac",
            year: 1857,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "e4", "e5", "Nf3", "Nc6", "Nc3", "Nf6", "Bb5", "Bc5", "O-O", "O-O",
                "Nxe5", "Re8", "Nxc6", "dxc6", "Bc4", "b5", "Be2", "Nxe4", "Nxe4", "Rxe4",
                "Bf3", "Re6", "c3", "Qd3", "b4", "Bb6", "a4", "bxa4", "Qxa4", "Bd7",
                "Ra2", "Rae8", "Qa6", "Qxf3", "gxf3", "Rg6+", "Kh1", "Bh3", "Rd1", "Bg2+",
                "Kg1", "Bxf3+", "Kf1", "Bg2+", "Kg1", "Bh3+", "Kh1", "Bxf2", "Qf1", "Bxf1",
                "Rxf1", "Re2", "Ra1", "Rh6", "d4", "Be3"
            ],
        },
        // 19. Robert Byrne vs Bobby Fischer (1963) - The Brilliance Prize
        GameData {
            white: "Robert Byrne",
            black: "Bobby Fischer",
            event: "The Brilliance Prize",
            year: 1963,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "d4", "Nf6", "c4", "g6", "g3", "c6", "Bg2", "d5", "cxd5", "cxd5",
                "Nc3", "Bg7", "e3", "O-O", "Nge2", "Nc6", "O-O", "b6", "b3", "Ba6",
                "Ba3", "Re8", "Qd2", "e5", "dxe5", "Nxe5", "Rfd1", "Nd3", "Qc2", "Nxf2",
                "Kxf2", "Ng4+", "Kg1", "Nxe3", "Qd2", "Nxg2", "Kxg2", "d4", "Nxd4", "Bb7+",
                "Kf1", "Qd7"
            ],
        },
        // 20. Levon Aronian vs Viswanathan Anand (2013) - Anand's Immortal
        GameData {
            white: "Levon Aronian",
            black: "Viswanathan Anand",
            event: "Anand's Immortal",
            year: 2013,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "d4", "d5", "c4", "c6", "Nf3", "Nf6", "Nc3", "e6", "e3", "Nbd7",
                "Bd3", "dxc4", "Bxc4", "b5", "Bd3", "Bd6", "O-O", "O-O", "Qc2", "Bb7",
                "a3", "Rc8", "Ng5", "c5", "Nxh7", "Ng4", "f4", "cxd4", "exd4", "Bc5",
                "Be2", "Nde5", "Bxg4", "Bxd4+", "Kh1", "Nxg4", "Nxf8", "f5", "Ng6", "Qf6",
                "h3", "Qxg6", "Qe2", "Qh5", "Qd3", "Be3"
            ],
        },
        // 21. Mikhail Botvinnik vs Mikhail Tal (1960) - Tal's Knight Sacrifice
        GameData {
            white: "Mikhail Botvinnik",
            black: "Mikhail Tal",
            event: "Tal's Knight Sacrifice",
            year: 1960,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "c4", "Nf6", "Nf3", "g6", "g3", "Bg7", "Bg2", "O-O", "d4", "d6",
                "Nc3", "Nbd7", "O-O", "e5", "e4", "c6", "h3", "Qb6", "d5", "cxd5",
                "cxd5", "Nc5", "Ne1", "Bd7", "Nd3", "Nxd3", "Qxd3", "Rfc8", "Rb1", "Nh5",
                "Be3", "Qb4", "Qe2", "Rc4", "Rfc1", "Rac8", "Kh2", "f5", "exf5", "Bxf5",
                "Ra1", "Nf4", "gxf4", "exf4", "Bd2", "Qxb2", "Rab1", "f3", "Rxb2", "fxe2",
                "Rb3", "Rd4", "Be1", "Be5+", "Kg1", "Bf4", "Nxe2", "Rxc1", "Nxd4", "Rxe1+",
                "Bf1", "Be4", "Ne2", "Be5", "f4", "Bf6", "Rxb7", "Bxd5", "Rc7", "Bxa2",
                "Rxa7", "Bc4", "Ra8+", "Kf7", "Ra7+", "Ke6", "Ra3", "d5", "Kf2", "Bh4+",
                "Kg2", "Kd6", "Ng3", "Bxg3", "Bxc4", "dxc4", "Kxg3", "Kd5", "Ra7", "c3",
                "Rc7", "Kd4", "Rd7+", "Ke4", "Re7+", "Kd3", "Rd7+", "Ke2", "Rc7", "Kd2",
                "Rd7+", "Kc1", "Rxh7", "c2", "Rc7", "Re3+", "Kg4", "Kd2", "h4", "Rc3",
                "Rd7+", "Ke2", "Re7+", "Kf2", "Re1", "Kxe1", "f5", "c1=Q", "fxg6"
            ],
        },
        // 22. Veselin Topalov vs Alexei Shirov (1998) - Shirov's Bishop Sacrifice
        GameData {
            white: "Veselin Topalov",
            black: "Alexei Shirov",
            event: "Shirov's Bishop Sac",
            year: 1998,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "d4", "Nf6", "c4", "g6", "Nc3", "d5", "cxd5", "Nxd5", "e4", "Nxc3",
                "bxc3", "Bg7", "Bb5+", "c6", "Ba4", "O-O", "Ne2", "Nd7", "O-O", "e5",
                "f3", "Qe7", "Be3", "Rd8", "Qc2", "Nb6", "Bb3", "Be6", "Rad1", "Nc4",
                "Bc1", "b5", "f4", "exd4", "Nxd4", "Bg4", "Rde1", "Qc5", "Kh1", "a5",
                "h3", "Bd7", "a4", "bxa4", "Ba2", "Be8", "e5", "Nb6", "f5", "Nd5",
                "Bd2", "Nb4", "Qxa4", "Nxa2", "Qxa2", "Bxe5", "fxg6", "hxg6", "Bg5", "Rd5",
                "Re3", "Qd6", "Qe2", "Bd7", "c4", "Bxd4", "cxd5", "Bxe3", "Qxe3", "Re8",
                "Qc3", "Qxd5", "Bh6", "Re5", "Rf3", "Qc5", "Qa1", "Bf5", "Re3", "f6",
                "Rxe5", "Qxe5", "Qa2+", "Qd5", "Qxd5+", "cxd5", "Bd2", "a4", "Bc3", "Kf7",
                "h4", "Ke6", "Kg1", "Bh3", "gxh3", "Kf5", "Kf2", "Ke4", "Bxf6", "d4",
                "Be7", "Kd3", "Bc5", "Kc4", "Be7", "Kb3"
            ],
        },
        // 23. Magnus Carlsen vs Sergey Karjakin (2016) - World Championship Tiebreak
        GameData {
            white: "Magnus Carlsen",
            black: "Sergey Karjakin",
            event: "Carlsen's Queen Sac",
            year: 2016,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "f3", "e5",
                "Nb3", "Be7", "c4", "a5", "Be3", "a4", "Nc1", "O-O", "Nc3", "Qa5",
                "Qd2", "Na6", "Be2", "Nc5", "O-O", "Bd7", "Rb1", "Rfc8", "b4", "axb3",
                "axb3", "Qd8", "Nd3", "Ne6", "Nb4", "Bc6", "Rfd1", "h5", "Bf1", "h4",
                "Qf2", "Nd7", "g3", "Ra3", "Bh3", "Rca8", "Nc2", "R3a6", "Nb4", "Ra5",
                "Nc2", "b6", "Rd2", "Qc7", "Rbd1", "Bf8", "gxh4", "Nf4", "Bxf4", "exf4",
                "Bxd7", "Qxd7", "Nb4", "Ra3", "Nxc6", "Qxc6", "Nb5", "Rxb3", "Nd4", "Qxc4",
                "Nxb3", "Qxb3", "Qe2", "Be7", "Kg2", "Qe6", "h5", "Ra3", "Rd3", "Ra2",
                "R3d2", "Ra3", "Rd3", "Ra7", "Rd5", "Rc7", "Qd2", "Qf6", "Rf5", "Qh4",
                "Rc1", "Ra7", "Qxf4", "Ra2+", "Kh1", "Qf2", "Rc8+", "Kh7", "Qh6+"
            ],
        },
        // 24. Daniil Dubov vs Sergey Karjakin (2020) - Dubov's Queen Sac
        GameData {
            white: "Daniil Dubov",
            black: "Sergey Karjakin",
            event: "Dubov's Fireworks",
            year: 2020,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "c3", "Nf6", "d4", "exd4",
                "b4", "Bb6", "e5", "Ne4", "Bd5", "Nxc3", "Nxc3", "dxc3", "Bg5", "Ne7",
                "O-O", "h6", "Bh4", "O-O", "Re1", "Qe8", "Bb3", "a5", "Bf6", "a4",
                "Bc4", "Ng6", "Qd3", "d5", "exd6", "Be6", "Qxg6", "fxg6", "Rxe6", "Qf7",
                "Bxc3", "Kh8", "Re4", "Qf5", "Re7", "Rg8", "Bxg8", "Rxg8", "dxc7", "Qc2",
                "Be5", "Bxf2+", "Kh1", "Bb6", "h3", "Kh7", "Re1", "a3", "Kh2", "g5",
                "Nd4", "Qc4", "Nf5", "Qxb4", "Rc1", "Kg6", "Rxg7+", "Kxf5", "Rxg8", "Bxc7",
                "Bxc7", "Qb2", "Rc5+", "Ke4", "Rd8"
            ],
        },
        // 25. Lev Polugaevsky vs Rashid Nezhmetdinov (1958) - Nezhmetdinov's Immortal
        GameData {
            white: "Lev Polugaevsky",
            black: "Rashid Nezhmetdinov",
            event: "Nezhmetdinov's Immortal",
            year: 1958,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "d4", "Nf6", "c4", "d6", "Nc3", "e5", "e4", "exd4", "Qxd4", "Nc6",
                "Qd2", "g6", "b3", "Bg7", "Bb2", "O-O", "Bd3", "Ng4", "Nge2", "Qh4",
                "Ng3", "Nge5", "O-O", "f5", "f3", "Bh6", "Qd1", "f4", "Nge2", "g5",
                "Nd5", "g4", "g3", "fxg3", "hxg3", "Qh3", "f4", "Be6", "Bc2", "Rf7",
                "Kf2", "Qh2+", "Ke3", "Bxd5", "cxd5", "Nb4", "Rh1", "Rxf4", "Rxh2", "Rf3+",
                "Kd4", "Bg7", "a4", "c5+", "dxc6", "bxc6", "Bd3", "Nexd3+", "Kc4", "d5+",
                "exd5", "cxd5+", "Kb5", "Rb8+", "Ka5", "Nc6+"
            ],
        },
        // 26. Tigran Petrosian vs Ludek Pachman (1961) - Petrosian's Queen Sac
        GameData {
            white: "Tigran Petrosian",
            black: "Ludek Pachman",
            event: "Petrosian's Queen Sac",
            year: 1961,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "Nf3", "c5", "g3", "Nc6", "Bg2", "g6", "O-O", "Bg7", "d3", "e6",
                "e4", "Nge7", "Re1", "O-O", "e5", "d6", "exd6", "Qxd6", "Nbd2", "Qc7",
                "Nb3", "Nd4", "Bf4", "Qb6", "Ne5", "Nxb3", "Nc4", "Qb5", "axb3", "a5",
                "Bd6", "Bf6", "Qf3", "Kg7", "Re4", "Rd8", "Qxf6+", "Kxf6", "Be5+", "Kg5",
                "Bg7"
            ],
        },
        // 27. Judit Polgar vs Ferenc Berkes (2003) - Polgar's Firestorm
        GameData {
            white: "Judit Polgár",
            black: "Ferenc Berkes",
            event: "Polgar's Firestorm",
            year: 2003,
            result: "1-0",
            termination: "Abandono",
            moves: &[
                "e4", "e6", "d4", "d5", "Nc3", "Nf6", "Bg5", "dxe4", "Nxe4", "Be7",
                "Bxf6", "Bxf6", "Nf3", "O-O", "Qd2", "Nd7", "O-O-O", "Be7", "Bd3", "b6",
                "Neg5", "h6", "Bh7+", "Kh8", "Be4", "hxg5", "g4", "Rb8", "h4", "g6",
                "hxg5+", "Kg7", "Qf4", "Bb7", "Rh7+", "Kxh7", "Qh2+", "Kg8", "Rh1", "Bxg5+",
                "Nxg5", "Qxg5+", "f4", "Qxf4+", "Qxf4", "Bxe4", "Qxe4"
            ],
        },
        // 28. Richard Reti vs Alexander Alekhine (1925) - Alekhine's Masterpiece
        GameData {
            white: "Richard Réti",
            black: "Alexander Alekhine",
            event: "Alekhine's Masterpiece",
            year: 1925,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "g3", "e5", "Nf3", "e4", "Nd4", "d5", "d3", "exd3", "Qxd3", "Nf6",
                "Bg2", "Bb4+", "Bd2", "Bxd2+", "Nxd2", "O-O", "c4", "Na6", "cxd5", "Nb4",
                "Qc4", "Nbxd5", "N2b3", "c6", "O-O", "Re8", "Rfd1", "Bg4", "Rd2", "Qc8",
                "Nc5", "Bh3", "Bf3", "Bg4", "Bg2", "Bh3", "Bf3", "Bg4", "Bh1", "h5",
                "b4", "a6", "Rc1", "h4", "a4", "hxg3", "hxg3", "Qc7", "b5", "axb5",
                "axb5", "Re3", "Nf3", "cxb5", "Qxb5", "Nc3", "Qxb7", "Qxb7", "Nxb7", "Nxe2+",
                "Kh2", "Ne4", "Rc4", "Nxf2", "Bg2", "Be6", "Rcc2", "Ng4+", "Kh3", "Ne5+",
                "Kh2", "Rxf3", "Rxe2", "Ng4+", "Kh3", "Ne3+", "Kh2", "Nxc2", "Bxf3", "Nd4"
            ],
        },
        // 29. Anatoly Karpov vs Garry Kasparov (1985) - World Championship Game 16
        GameData {
            white: "Anatoly Karpov",
            black: "Garry Kasparov",
            event: "The Octopus Knight",
            year: 1985,
            result: "0-1",
            termination: "Abandono",
            moves: &[
                "e4", "c5", "Nf3", "e6", "d4", "cxd4", "Nxd4", "Nc6", "Nb5", "d6",
                "c4", "Nf6", "N1c3", "a6", "Na3", "d5", "cxd5", "exd5", "exd5", "Nb4",
                "Be2", "Bc5", "O-O", "O-O", "Bf3", "Bf5", "Bg5", "Re8", "Qd2", "b5",
                "Rad1", "Nd3", "Nab1", "h6", "Bh4", "b4", "Na4", "Bd6", "Bg3", "Rc8",
                "b3", "g5", "Bxd6", "Qxd6", "g3", "Nd7", "Bg2", "Qf6", "a3", "a5",
                "axb4", "axb4", "Qa2", "Bg6", "d6", "g4", "Qd2", "Kg7", "f3", "Qxd6",
                "fxg4", "Qd4+", "Kh1", "Nf6", "Rf4", "Ne4", "Qxd3", "Nf2+", "Rxf2", "Bxd3",
                "Rfd2", "Qe3", "Rxd3", "Rc1", "Nb2", "Qf2", "Nd2", "Rxd1+", "Nxd1", "Re1+"
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Chess, Position};
    use shakmaty::san::San;

    #[test]
    fn test_all_games_legal_moves() {
        for (i, g) in get_master_games().iter().enumerate() {
            let mut pos = Chess::default();
            for (m_idx, &m_str) in g.moves.iter().enumerate() {
                let san: San = m_str.parse().expect(&format!("Game {} move {} failed to parse: {}", i, m_idx, m_str));
                let m = san.to_move(&pos).expect(&format!("Game {} ('{}') move {} ('{}') is illegal!", i, g.event, m_idx + 1, m_str));
                pos.play_unchecked(m);
            }
        }
    }
}
