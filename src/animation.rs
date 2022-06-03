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

    // todo: better duration type
    pub fn at(&self, time: f64) -> f64 {
        let mut t = time / self.time;
        if t >= 1.0 {
            t = 1.0;
        }

        assert_eq!(true, t >= 0.0);
        assert_eq!(true, t <= 1.0);

        // todo: easings
        if t >= self.time {
            return self.end;
        }

        // todo: this is linear only
        self.start + t * (self.end - self.start)
    }
}
pub struct Animator {
    animations: Vec<Animation>,
}

impl Animator {
    pub fn new() -> Animator {
        Animator {
            animations: Vec::new(),
        }
    }

    pub fn add(&mut self, anim: Animation) {
        self.animations.push(anim);
    }

    pub fn update(&mut self, t: f64) {
        println!("update {} / {}", t, self.animations.len());
        for anim in self.animations.iter() {
            anim.object
                .lock()
                .unwrap()
                .set_property(anim.prop, anim.at(t));
        }
    }
}
