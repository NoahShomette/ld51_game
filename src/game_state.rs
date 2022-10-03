use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub enum GamePlayState {
    Menu,
    Win,
    Lose,
    Playing,
}

pub struct GameStateInfo {
    pub(crate) game_state: GamePlayState,
}

//bevy thing to allow it to be used as a resource
impl FromWorld for GameStateInfo {
    fn from_world(world: &mut World) -> Self {
        GameStateInfo {
            game_state: GamePlayState::Menu,
        }
    }
}

impl GameStateInfo {
    pub(crate) fn change_game_play_state(
        &mut self,
        play_state: GamePlayState,
        mut event_writer: &mut EventWriter<GamePlayState>,
    ) {
        match self.game_state {
            GamePlayState::Menu => {
                match play_state {
                    GamePlayState::Menu => {} //nothing
                    GamePlayState::Win => {}  //nothing shouldnt be able to go here
                    GamePlayState::Lose => {} //nothing shouldnt be able to go here
                    GamePlayState::Playing => {
                        self.game_state = GamePlayState::Playing;
                        event_writer.send(GamePlayState::Playing)
                    } //starts the game
                }
            }
            GamePlayState::Win => {
                match play_state {
                    GamePlayState::Menu => {
                        self.game_state = GamePlayState::Menu;
                        event_writer.send(GamePlayState::Menu)
                    } //go to main menu
                    GamePlayState::Win => {}  //nothing
                    GamePlayState::Lose => {} //nothing
                    GamePlayState::Playing => {
                        self.game_state = GamePlayState::Playing;
                        event_writer.send(GamePlayState::Playing)
                    } // restart game
                }
            }
            GamePlayState::Lose => {
                match play_state {
                    GamePlayState::Menu => {
                        self.game_state = GamePlayState::Menu;
                        event_writer.send(GamePlayState::Menu)
                    } //go to main menu
                    GamePlayState::Win => {}  //nothing
                    GamePlayState::Lose => {} //nothing
                    GamePlayState::Playing => {
                        self.game_state = GamePlayState::Playing;
                        event_writer.send(GamePlayState::Playing)
                    } //restart game
                }
            }
            GamePlayState::Playing => {
                match play_state {
                    GamePlayState::Menu => {
                        self.game_state = GamePlayState::Menu;
                        event_writer.send(GamePlayState::Menu)
                    } //end game and go to main menu
                    GamePlayState::Win => {
                        self.game_state = GamePlayState::Win;
                        event_writer.send(GamePlayState::Win)
                    } //game done and show win screen
                    GamePlayState::Lose => {
                        self.game_state = GamePlayState::Lose;
                        event_writer.send(GamePlayState::Lose)
                    } //game done and show lose screen
                    GamePlayState::Playing => {
                        self.game_state = GamePlayState::Playing;
                        event_writer.send(GamePlayState::Playing)
                    } // restart game
                }
            }
        }
    }
}
