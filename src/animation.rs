use std::sync::Arc;
use std::sync::Mutex;

use crate::scene;

#[derive(Copy, Clone)]
pub enum AnimationProperty {
    X,
    Y,
    Z,
    RADIUS,
}

pub enum Easing {
    LINEAR,
}

pub struct Animation {
    object: Arc<Mutex<dyn scene::Hitable>>,
    prop: AnimationProperty,
    start: f64,
    time: f64, // todo: duration and some std::duration?
    end: f64,
    ease: Easing,
}

impl Animation {
    pub fn new(
        object: Arc<Mutex<dyn scene::Hitable>>,
        prop: AnimationProperty,
        start: f64,
        end: f64,
        time: f64,
        ease: Easing,
    ) -> Animation {
        Animation {
            object,
            prop,
            start,
            end,
            time,
            ease,
        }
    }

    pub fn at(&self, t: f64) -> f64 {
        assert_eq!(true, t <= 1.0);
        assert_eq!(true, t >= 0.0);

        // todo
        return t;
    }
}
pub struct Animator {
    delta: f64,
    animations: Vec<Animation>,
}

impl Animator {
    pub fn new() -> Animator {
        Animator {
            delta: 0.0,
            animations: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.delta += dt;

        for anim in self.animations.iter() {
            anim.object
                .lock()
                .unwrap()
                .set_property(anim.prop, anim.at(self.delta));
        }
    }

    pub fn reset(&mut self) {
        self.delta = 0.0
    }
}
