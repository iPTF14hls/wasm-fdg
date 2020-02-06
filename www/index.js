import { init, update_mouse_position, spawn_entity, print_arena_statistics, tick } from "force-directed-graph";

const ARENA = document.getElementById("arena");

/*
function update_size() {

let rect = ARENA.getBoundingClientRect();
//update_arena_size(rect.width, rect.height);
}

update_size();
object.addEventListener("resize", event => {
    update_size();
});

*/

init();

ARENA.addEventListener("mousemove", event => {
    update_mouse_position(event.clientX, event.clientY);
})

async function loop() {
    let timer = document.getElementById("timer");
    while (true) {
        await new Promise(resolve => setTimeout(resolve, 16));
        tick();
    }
}

loop();
spawn_entity(`<img width="100" src=https://upload.wikimedia.org/wikipedia/commons/thumb/9/9b/DVD_logo.svg/1200px-DVD_logo.svg.png>`, ["greeeeeen"])
const show_details = document.getElementById("status");

show_details.addEventListener("click", event => {
    alert(print_arena_statistics())
});

const info = document.getElementById("yeet");
info.textContent = print_arena_statistics();