#pragma once

#include "rx/game.h"

namespace bve {
    struct game : rx::game {
        game(void *_self,
             bool(*_on_init)(void *),
             rx::game::status(*_on_slice)(void *, rx::input::input &),
             void(*_on_resize)(void *, const rx::math::vec2z &),
             void(*_dtor)(void *));

        void *m_self;
        bool (*m_on_init)(void *);
        rx::game::status (*m_on_slice)(void *, rx::input::input &);
        void (*m_on_resize)(void *, const rx::math::vec2z &);
        void (*m_dtor)(void *);

        bool on_init() override;
        rx::game::status on_slice(rx::input::input &) override;
        void on_resize(const rx::math::vec2z &) override;
        ~game() override;
    };
}