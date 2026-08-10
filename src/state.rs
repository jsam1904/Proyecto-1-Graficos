#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    Welcome,
    LevelSelect,
    Playing,
    Success,
}