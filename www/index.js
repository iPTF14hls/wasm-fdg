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
    let fps = document.getElementById("fps");
    times.unshift(c_time);
    if (times.length > 45) {
        times.pop();
    }
    let acc = 0;
    let count = 0;
    for (let i = 1; i < times.length; i++) {
        acc += times[i-1]-times[i];
        count++;
    }
    fps.innerText = "FPS: " + Math.round(count/(acc*0.001));
}

async function loop() {
    let timer = document.getElementById("timer");
    while (true) {
        await new Promise(resolve => setTimeout(resolve, 10));
        tick();
        fps(performance.now());
    }
}

loop();
spawn_entity(500, 500, 200, 110, 100, 100,`<img width="400" src=https://upload.wikimedia.org/wikipedia/commons/thumb/9/9b/DVD_logo.svg/1200px-DVD_logo.svg.png>`, [])
const show_details = document.getElementById("status");

show_details.addEventListener("click", event => {
    alert(print_arena_statistics())
});

const info = document.getElementById("yeet");
info.textContent = print_arena_statistics();
