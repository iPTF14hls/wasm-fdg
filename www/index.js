import {init, update_mouse_position, spawn_entity, print_arena_statistics, tick} from "force-directed-graph";

const ARENA = document.getElementById("arena");

init();

ARENA.addEventListener("mousemove", event => {
    update_mouse_position(event.offsetX, event.offsetY);
})

async function loop() {
    let timer = document.getElementById("timer");
    while (true) {
        await new Promise(resolve => setTimeout(resolve, 16));
        tick();
    }
}

loop();
spawn_entity("Hello, world!", ["Yeet"])
const show_details = document.getElementById("status");

show_details.addEventListener("click", event => {
    alert(print_arena_statistics())
});

const info = document.getElementById("yeet");
info.textContent = print_arena_statistics();