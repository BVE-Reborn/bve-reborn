use bve_rex_sys::*;

struct GameBuilder;

impl bve_rex::game::GameBuilder for GameBuilder {
    type Built = Game;

    fn build(&mut self, _interface: &mut rx::render::frontend::interface) -> Box<Self::Built> {
        Box::new(Game)
    }
}

struct Game;

impl bve_rex::game::Game for Game {
    fn on_init(&mut self) -> bool {
        true
    }

    fn on_slice(&mut self, _input: &mut rx::input::input) -> rx::game_status {
        rx::game_status::k_shutdown
    }

    fn on_resize(&mut self, _dimensions: &rx::math::vec2z) {}
}

fn main() {
    bve_rex::game::start(Box::new(GameBuilder))
}
