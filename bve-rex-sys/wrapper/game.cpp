#include "game.hpp"
namespace bve {
    game::game(void *_self,
               bool(*_on_init)(void *),
               rx::game::status(*_on_slice)(void *, rx::input::input &),
               void(*_on_resize)(void *, const rx::math::vec2z &),
               void(*_dtor)(void *))
            : m_self(_self),
              m_on_init(_on_init),
              m_on_slice(_on_slice),
              m_on_resize(_on_resize),
              m_dtor(_dtor) {}

    bool game::on_init() {
        return this->m_on_init(this->m_self);
    }

    rx::game::status game::on_slice(rx::input::input &_input) {
        return this->m_on_slice(this->m_self, _input);
    }

    void game::on_resize(const rx::math::vec2z &_resolution) {
        this->m_on_resize(this->m_self, _resolution);
    }

    game::~game() {
        this->m_dtor(this->m_self);
    }
}