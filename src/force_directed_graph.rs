use log::info;
use specs::{
    world::EntitiesRes, Component, Entities, Join, NullStorage, Read, ReadStorage, System,
    VecStorage, World, WorldExt, WriteStorage,
};
#[derive(Default)]
pub struct DeltaTime(pub f64);

#[derive(Default)]
pub struct MousePos(pub (f64, f64));

#[derive(Default)]
pub struct ArenaSize(pub (f64, f64));

pub struct PosDiff {
    pub dx: f64,
    pub dy: f64,
}

impl PosDiff {
    fn dist_squared(&self) -> f64 {
        self.dx.powi(2) + self.dy.powi(2)
    }

    fn dist(&self) -> f64 {
        self.dist_squared().sqrt()
    }

    fn angle(&self) -> f64 {
        self.dy.atan2(self.dx)
    }
}

//All units in Pixels
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    fn diff(&self, other: &Self) -> PosDiff {
        PosDiff {
            dx: self.x - other.x,
            dy: self.y - other.y,
        }
    }
}

//All units are in Pixels per seconds (pps)
#[derive(Component, Debug, Default)]
#[storage(VecStorage)]
pub struct Velocity {
    pub xv: f64,
    pub yv: f64,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct DomElement {
    pub id: String,
}

#[derive(Component, Default, Debug)]
#[storage(NullStorage)]
pub struct MouseAttract;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider {
    pub w: f64,
    pub h: f64,
}

pub fn initialize_world() -> World {
    let mut world = World::new();
    world.insert(DeltaTime::default());
    world.insert(MousePos::default());
    world.insert(ArenaSize::default());
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<DomElement>();
    world.register::<MouseAttract>();
    world.register::<Collider>();
    world.register::<Repel>();
    world
}

struct VelocityApply;

impl<'a> System<'a> for VelocityApply {
    type SystemData = (
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, (mut pdat, vdat, d_time): Self::SystemData) {
        for (pos, vel) in (&mut pdat, &vdat).join() {
            //The 0.001 converts pixels per second to pixels per milisecond
            //Then d_time turns pixels per milisecond to pixels
            //After we add that to the position then we are done
            pos.x += (vel.xv * 0.001) * d_time.0;
            pos.y += (vel.yv * 0.001) * d_time.0;
        }
    }
}

struct ApplyPosition;

impl<'a> System<'a> for ApplyPosition {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, DomElement>,
        Entities<'a>,
        Read<'a, EntitiesRes>,
    );

    fn run(&mut self, (poses, doms, ents, entres): Self::SystemData) {
        use crate::utils::document;

        let doc = document();

        let eat_delete = |ent| {
            if entres.delete(ent).is_err() {
                //Honestly we couldn't give less of a shit about this.
                //So this branch leads into a comment which will be optumized out.
                //The reason why we don't care is because the deletion can only
                //fail if it's been deleted already.
                //So either way the thing is deleted and we're now just twittling our thumbs until
                //the world can collect the garbage.
            }
        };

        for (ent, pos, dom) in (&*ents, &poses, &doms).join() {
            match doc.get_element_by_id(&dom.id) {
                Some(elem) => {
                    let rect = elem.get_bounding_client_rect();
                    let (dx, dy) = (pos.x - rect.width() / 2., pos.y - rect.height() / 2.);
                    if elem
                        .set_attribute(
                            "style",
                            &format!("position: absolute;top:{}px;left:{}px", dy, dx),
                        )
                        .is_err()
                    {
                        eat_delete(ent);
                    };
                }
                None => eat_delete(ent),
            }
        }
    }
}

struct FollowMouse;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Repel {
    pub charge: f64,
}

impl<'a> System<'a> for FollowMouse {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, MouseAttract>,
        Read<'a, MousePos>,
    );

    fn run(&mut self, (poses, mut vels, attracts, mpos): Self::SystemData) {
        let (mx, my) = mpos.0;
        const MAX_ACC: f64 = 10.;
        for (pos, mut vel, _) in (&poses, &mut vels, &attracts).join() {
            let (ox, oy) = (mx - pos.x, my - pos.y);
            let mag = smooth_step((ox.powi(2) + oy.powi(2)).sqrt() / MAX_ACC) * MAX_ACC;
            let ang = oy.atan2(ox);
            vel.xv += ang.cos() * mag;
            vel.yv += ang.sin() * mag;
        }
    }
}

struct Wall;

impl<'a> System<'a> for Wall {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Collider>,
        Read<'a, ArenaSize>,
        Entities<'a>,
    );

    fn run(&mut self, (mut poses, mut vels, cldrs, size, ents): Self::SystemData) {
        let (w, h) = size.0;
        const FRICTION: f64 = 0.8;
        for (ent, mut pos, cld) in (&*ents, &mut poses, &cldrs).join() {
            let (px, py) = (pos.x, pos.y);
            let vop = vels.get_mut(ent);
            if w - cld.w < 0. || h - cld.h < 0. {
                return;
            }
            pos.x = pos.x.clamp(cld.w, w - cld.w);
            pos.y = pos.y.clamp(cld.h, h - cld.h);
            if let Some(mut vel) = vop {
                let (dx, dy) = (px - pos.x, py - pos.y);
                if dx != 0.0 {
                    vel.xv *= -FRICTION;
                }

                if dy != 0. {
                    vel.yv *= -FRICTION;
                }
            }
        }
    }
}

struct Friction;

impl<'a> System<'a> for Friction {
    type SystemData = WriteStorage<'a, Velocity>;

    fn run(&mut self, mut vels: Self::SystemData) {
        const FRICTION: f64 = 0.995;
        for mut vel in (&mut vels).join() {
            vel.xv *= FRICTION;
            vel.yv *= FRICTION;
        }
    }
}

struct CoulombRepulsion;

impl<'a> System<'a> for CoulombRepulsion {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Repel>,
    );

    fn run(&mut self, (poses, mut vels, charges): Self::SystemData) {
        const CONST: f64 = 1.;
        for (p1, mut vel, c1) in (&poses, &mut vels, &charges).join() {
            for (p2, c2) in (&poses, &charges).join() {
                let diff = p1.diff(p2);
                let dist = diff.dist_squared();
                if dist < 0.01 {
                    continue;
                }
                let acc = CONST * (c1.charge * c2.charge) / dist;
                let ang = diff.angle();
                vel.xv += ang.cos() * acc;
                vel.yv += ang.sin() * acc;
            }
        }
    }
}

fn smooth_step(x: f64) -> f64 {
    let cx = x.clamp(0., 1.);
    3. * cx.powi(2) - 2. * cx.powi(3)
}

pub fn execute_systems(world: &World) {
    use specs::RunNow;

    {
        let mut system = FollowMouse;
        system.run_now(world);
    }
    {
        let mut system = Friction;
        system.run_now(world);
    }

    {
        let mut system = CoulombRepulsion;
        system.run_now(world);
    }
    {
        let mut system = VelocityApply;
        system.run_now(world);
    }
    {
        let mut system = Wall;
        system.run_now(world);
    }
    {
        let mut system = ApplyPosition;
        system.run_now(world);
    }
}
