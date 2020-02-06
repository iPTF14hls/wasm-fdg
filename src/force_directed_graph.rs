use specs::{
    world::EntitiesRes, Builder, Component, Dispatcher, DispatcherBuilder, Entities, Join,
    NullStorage, Read, ReadStorage, System, VecStorage, World, WorldExt, WriteStorage,
};
use lazy_static::lazy_static;
use log::info;
#[derive(Default)]
pub struct DeltaTime(pub f64);

#[derive(Default)]
pub struct MousePos(pub (f64, f64));

//All units in Pixels
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position {
    pub x: f64,
    pub y: f64,
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
pub struct Collider { pub rad: f64 }

pub fn initialize_world() -> World {
    let mut world = World::new();
    world.insert(DeltaTime(0.));
    world.insert(MousePos((0., 0.)));
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<DomElement>();
    world.register::<MouseAttract>();
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
                    let (dx, dy) = (pos.x - rect.width() / 2., pos.y - rect.height());
                    if elem.set_attribute("style", &format!("position: absolute;top:{}px;left:{}px", dy, dx)).is_err() {
                        eat_delete(ent);
                    };
                }
                None => eat_delete(ent),
            }
        }
    }
}

struct FollowMouse;

impl<'a> System<'a> for FollowMouse {
    type SystemData = (ReadStorage<'a, Position>, WriteStorage<'a, Velocity>, ReadStorage<'a,MouseAttract>, Read<'a, MousePos>);

    fn run(&mut self, (poses, mut vels, attracts, mpos): Self::SystemData) {
        let (mx, my) = mpos.0;
        const MAX_ACC: f64 = 10.;
        for (pos, mut vel, _) in (&poses, &mut vels, &attracts).join() {
            let (ox, oy) = (mx - pos.x, my - pos.y);
            let mag = smooth_step((ox.powi(2) + oy.powi(2)).sqrt()/MAX_ACC)*MAX_ACC;
            let ang = oy.atan2(ox);
            vel.xv += ang.cos()*mag;
            vel.yv += ang.sin()*mag;
        }
    }
}

/*
impl<'a> System<'a> for FollowMouse {
    type SystemData = (WriteStorage<'a, Position>, Read<'a, MousePos>);

    fn run(&mut self, (mut poses, mpos): Self::SystemData) {
        let (mx, my) = mpos.0;
        const MAX_ACC: f64 = 1.;
        for mut pos in (&mut poses).join() {

            pos.x = mx;
            pos.y = my;
        }
    }
}
*/

struct Friction;

impl<'a> System<'a> for Friction {
    type SystemData = WriteStorage<'a, Velocity>;

    fn run(&mut self, mut vels: Self::SystemData) {
        const FRICTION: f64 = 0.95;
        for mut vel in (&mut vels).join() {
            vel.xv *= FRICTION;
            vel.yv *= FRICTION;
        }
    }
}


fn smooth_step(x: f64) -> f64 {
    let cx = x.clamp(0., 1.);
    3.*cx.powi(2) -2.*cx.powi(3)
}

pub fn execute_systems(world:&World) {
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
        let mut system = VelocityApply;
        system.run_now(world);
    }    
    {
        let mut system = ApplyPosition;
        system.run_now(world);
    }
    
}

pub fn initialize_dispatcher() -> Dispatcher<'static, 'static> {
    DispatcherBuilder::new()
    .with(FollowMouse, "follow_mouse", &[])
    .with(VelocityApply, "velocity_apply", &["follow_mouse"])
    .with(ApplyPosition, "apply_position", &["velocity_apply"])
    .build()
}
