use super::*;

/// Test Game construction.
#[test]
fn test_new_game() {
    let g = Game::new(&"#test_channel".to_string());
    assert!(g.channel == "#test_channel");
    assert!(match g.gun {
        Gun::Loaded => true,
        _ => false,
    });
    assert!(match g.phase {
        Phase::Inactive => true,
        _ => false,
    });
}

/// Test a Join event on an inactive game.
#[test]
fn test_join_event() {
    let g = Game::new(&"#test_channel".to_string());
    let e = GameEvent::Join("test_nick".to_string());
    let g2 = g.process(e);
    assert!(match g2.players {
        Participants::Players(_) => false,
        Participants::Joiners(ref v) => v.len() == 1 && v[0] == "test_nick",
    });
    assert!(match g2.phase {
        Phase::Starting(_) => true,
        _ => false,
    });
}

/// Test two join events.
#[test]
fn test_two_joins() {
    let g = Game::new(&"#test_channel".to_string());
    let e1 = GameEvent::Join("test_nick1".to_string());
    let e2 = GameEvent::Join("test_nick2".to_string());
    let g2 = g.process(e1);
    let g3 = g2.process(e2);
    assert!(match g3.phase {
        Phase::Starting(_) => true,
        _ => false,
    });
    match g3.players {
        Participants::Players(_) => assert!(false),
        Participants::Joiners(ref v) => {
            assert!(v.len() == 2);
            assert!(v.contains(&"test_nick1".to_string()));
            assert!(v.contains(&"test_nick2".to_string()))
        }
    }
}

/// Test an attempt to join twice with the same nick.
#[test]
fn test_join_twice() {
    let mut g = Game::new(&"#test_channel".to_string());
    let e = GameEvent::Join("test_nick".to_string());
    let e2 = GameEvent::Join("test_nick".to_string());
    g = g.process(e);
    g = g.process(e2);
    assert!(match g.players {
        Participants::Players(_) => false,
        Participants::Joiners(ref v) => v.len() == 1 && v[0] == "test_nick",
    });
}

/// Test of the leave event. Join single player and leave.
#[test]
fn test_leave() {
    // Create a Game and a join and leave Event for the same nick.
    let mut g = Game::new(&"#test_channel".to_string());
    let e = GameEvent::Join("test_nick".to_string());
    let e2 = GameEvent::Leave("test_nick".to_string());
    // Process events.
    g = g.process(e);
    g = g.process(e2);
    // Assertions:
    // When the only player of a game that is starting quits, the game goes inactive.
    assert!(match g.phase {
        Phase::Inactive => true,
        _ => false,
    });
    // And the player list is empty.
    assert!(match g.players {
        Participants::Players(_) => false,
        Participants::Joiners(ref v) => v.len() == 0,
    });
}

#[test]
// Testing two joins, followed by two leaves.
fn test_two_joins_two_leaves() {
    // Create a Game and two join and leave Events.
    let mut g = Game::new(&"#test_channel".to_string());
    let e = GameEvent::Join("test_nick".to_string());
    let e2 = GameEvent::Join("test_nick2".to_string());
    let e3 = GameEvent::Leave("test_nick".to_string());
    let e4 = GameEvent::Leave("test_nick2".to_string());
    // Process events.
    g = g.process(e);
    g = g.process(e2);
    g = g.process(e3);
    // Assertions:
    // After two joins and one leave, we're on Starting phase.
    assert!(match g.phase {
        Phase::Starting(_) => true,
        _ => false,
    });
    // And there's one player.
    assert!(match g.players {
        Participants::Players(_) => false,
        Participants::Joiners(ref v) => v.len() == 1,
    });
    // Now we process the last leave event.
    g = g.process(e4);
    // Assertions:
    // Back to the Starting condition with no players.
    assert!(match g.phase {
        Phase::Inactive => true,
        _ => false,
    });
    assert!(match g.players {
        Participants::Players(_) => false,
        Participants::Joiners(ref v) => v.len() == 0,
    });
}
