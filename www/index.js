import { init, update_mouse_position, spawn_entity, print_arena_statistics, tick, update_arena_size } from "force-directed-graph";

const ARENA = document.getElementById("arena");

init();

function update_size() {
    let box = ARENA.getBoundingClientRect();
    console.log(box.width+" "+box.height);
    update_arena_size(box.width, box.height);
}

update_size();
window.addEventListener("resize", evemt => {
    update_size();
}) 

ARENA.addEventListener("mousemove", event => {
    update_mouse_position(event.clientX, event.clientY);
})

let times = [];
times.unshift(performance.now());


async function fps(c_time) {
    if (c_time) {
        times.unshift(c_time);
        if (times.length > 45) {
            times.pop();
        }
    }
    let acc = 0;
    let count = 0;
    for (let i = 1; i < times.length; i++) {
        acc += times[i-1]-times[i];
        count++;
    }
    return count/(acc*0.001);
}

async function set_fps(c_time) {
    let f = fps(c_time);
    let display = document.getElementById("fps");
    display.innerText = "FPS: " + Math.round(await f);
}

async function game_loop(times) {
    for (let i = 0; i < times; i++) {
        await new Promise(resolve => setTimeout(resolve, 1));
        tick();
        set_fps(performance.now());
    }
}

let entities = 0;

async function loop() {
    let display = document.getElementById("bench");
    let pfps = Number.POSITIVE_INFINITY;
    console.log("yeet");
    while (pfps > 25) {
        await game_loop(50);
        pfps = await fps(undefined);
        console.log("Print fps" + pfps);
        let args = {
            "pos": {
                "x": 500,
                "y": 500
            },
            "vel": {
                "vx": 100,
                "vy": 100
            },
            "colds": {
                "w": 50,
                "h": 20
            },
            "html":`<img width="100" src=https://upload.wikimedia.org/wikipedia/commons/thumb/9/9b/DVD_logo.svg/1200px-DVD_logo.svg.png>`
        };
        spawn_entity(args, []);
        entities++;
        display.innerText = "Entities: "+entities;
    }
    
    display.innerText = "Done. Total Entities: "+entities;
    console.log("Broke");
    await game_loop(Number.POSITIVE_INFINITY);

}

loop();
const show_details = document.getElementById("status");

show_details.addEventListener("click", event => {
    alert(print_arena_statistics())
});

const info = document.getElementById("yeet");
info.textContent = print_arena_statistics();
